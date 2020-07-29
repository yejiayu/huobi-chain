#[cfg(test)]
mod tests;
mod types;

use asset::types::GetBalancePayload;
use asset::Assets;
use binding_macro::{cycles, genesis, service, write};
use derive_more::Display;
use governance::Governance;
use protocol::{
    traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap},
    types::{Address, ServiceContext, SignedTransaction},
};
use serde::Serialize;

use types::{AddressList, Genesis, NewAdmin, StatusList, Validate};

macro_rules! require_admin {
    ($service:expr, $ctx:expr) => {
        let admin = if let Some(tmp) = $service
            .sdk
            .get_value::<_, Address>(&ADMISSION_CONTROL_ADMIN_KEY.to_owned())
        {
            tmp
        } else {
            return ServiceError::NonAuthorized.into();
        };

        if admin != $ctx.get_caller() {
            return ServiceError::NonAuthorized.into();
        }
    };
}

macro_rules! require_valid_payload {
    ($payload:expr) => {
        if let Err(e) = $payload.validate() {
            return e.into();
        }
    };
}

macro_rules! sub_cycles {
    ($ctx:expr, $cycles:expr) => {
        if !$ctx.sub_cycles($cycles) {
            return ServiceError::OutOfCycles.into();
        }
    };
}

pub trait AdmissionControl {
    fn is_allowed(&self, ctx: &ServiceContext, payload: SignedTransaction) -> bool;
}

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "No authorized")]
    NonAuthorized,

    #[display(fmt = "Codec {}", _0)]
    Codec(serde_json::Error),

    #[display(fmt = "Out of cycles")]
    OutOfCycles,

    #[display(fmt = "Blocked transaction")]
    BlockedTx,

    #[display(fmt = "Bad payload {}", _0)]
    BadPayload(&'static str),

    #[display(fmt = "Balance lower than fee")]
    BalanceLowerThanFee,

    #[display(fmt = "Can not get admin address")]
    CannotGetAdmin,
}

impl ServiceError {
    pub fn code(&self) -> u64 {
        match self {
            ServiceError::NonAuthorized => 1000,
            ServiceError::Codec(_) => 1001,
            ServiceError::OutOfCycles => 1002,
            ServiceError::BlockedTx => 1003,
            ServiceError::BadPayload(_) => 1004,
            ServiceError::BalanceLowerThanFee => 1005,
            ServiceError::CannotGetAdmin => 1006,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}

const ADMISSION_CONTROL_ADMIN_KEY: &str = "admission_control_admin";

pub struct AdmissionControlService<A, G, SDK> {
    sdk:       SDK,
    deny_list: Box<dyn StoreMap<Address, bool>>,

    asset:      A,
    governance: G,
}

impl<A, G, SDK> AdmissionControl for AdmissionControlService<A, G, SDK>
where
    A: Assets,
    G: Governance,
    SDK: ServiceSDK + 'static,
{
    fn is_allowed(&self, ctx: &ServiceContext, payload: SignedTransaction) -> bool {
        (!self.is_permitted(ctx.clone(), payload.clone()).is_error())
            && (!self.is_valid(ctx.clone(), payload).is_error())
    }
}

#[service]
impl<A, G, SDK> AdmissionControlService<A, G, SDK>
where
    A: Assets,
    G: Governance,
    SDK: ServiceSDK + 'static,
{
    pub fn new(mut sdk: SDK, asset: A, governance: G) -> Self {
        let deny_list = sdk.alloc_or_recover_map("admission_control_deny_list");

        AdmissionControlService {
            sdk,
            deny_list,
            asset,
            governance,
        }
    }

    // # Panic invalid genesis
    #[genesis]
    fn init_genesis(&mut self, payload: Genesis) {
        if let Err(e) = payload.validate() {
            panic!(e.to_string());
        }

        self.sdk
            .set_value(ADMISSION_CONTROL_ADMIN_KEY.to_owned(), payload.admin);

        for addr in payload.deny_list {
            self.deny_list.insert(addr, true);
        }
    }

    #[cycles(10_000)]
    #[read]
    fn get_admin(&self, ctx: ServiceContext) -> ServiceResponse<Address> {
        if let Some(admin) = self.sdk.get_value(&ADMISSION_CONTROL_ADMIN_KEY.to_owned()) {
            ServiceResponse::from_succeed(admin)
        } else {
            ServiceError::CannotGetAdmin.into()
        }
    }

    #[cycles(10_000)]
    #[read]
    fn is_permitted(&self, ctx: ServiceContext, payload: SignedTransaction) -> ServiceResponse<()> {
        if self.deny_list.contains(&payload.raw.sender) {
            ServiceError::BlockedTx.into()
        } else {
            ServiceResponse::from_succeed(())
        }
    }

    #[cycles(10_000)]
    #[read]
    fn is_valid(&self, ctx: ServiceContext, payload: SignedTransaction) -> ServiceResponse<()> {
        // Check sender balance for failure fee
        let sender_balance = match self.get_native_balance(&ctx, &payload.raw.sender) {
            Ok(b) => b,
            Err(e) => return e,
        };

        let failure_fee = match self.governance.get_info(&ctx) {
            Ok(info) => info.tx_failure_fee,
            Err(e) => return e,
        };

        if sender_balance < failure_fee {
            ServiceError::BalanceLowerThanFee.into()
        } else {
            ServiceResponse::from_succeed(())
        }
    }

    #[cycles(500_00)]
    #[read]
    fn status(&self, _ctx: ServiceContext, payload: AddressList) -> ServiceResponse<StatusList> {
        let result = payload
            .addrs
            .iter()
            .map(|address| !self.deny_list.contains(address))
            .collect::<Vec<bool>>();

        ServiceResponse::from_succeed(StatusList { status: result })
    }

    #[cycles(210_00)]
    #[write]
    fn change_admin(&mut self, ctx: ServiceContext, payload: NewAdmin) -> ServiceResponse<()> {
        require_admin!(self, ctx);
        require_valid_payload!(&payload);

        self.sdk.set_value(
            ADMISSION_CONTROL_ADMIN_KEY.to_owned(),
            payload.new_admin.clone(),
        );

        Self::emit_event(&ctx, "ChangeAdmin".to_owned(), payload)
    }

    #[write]
    fn forbid(&mut self, ctx: ServiceContext, payload: AddressList) -> ServiceResponse<()> {
        require_admin!(self, ctx);
        sub_cycles!(ctx, payload.addrs.len() as u64 * 10_000);

        for addr in payload.addrs.iter() {
            self.deny_list.insert(addr.to_owned(), true);
        }

        Self::emit_event(&ctx, "Forbid".to_owned(), payload)
    }

    #[write]
    fn permit(&mut self, ctx: ServiceContext, payload: AddressList) -> ServiceResponse<()> {
        require_admin!(self, ctx);
        sub_cycles!(ctx, payload.addrs.len() as u64 * 10_000);

        for addr in payload.addrs.iter() {
            self.deny_list.remove(addr);
        }

        Self::emit_event(&ctx, "Permit".to_owned(), payload)
    }

    fn get_native_balance(
        &self,
        ctx: &ServiceContext,
        caller: &Address,
    ) -> Result<u64, ServiceResponse<()>> {
        let asset_id = self.asset.native_asset(&ctx)?.id;
        let native_account = self.asset.balance(&ctx, GetBalancePayload {
            asset_id,
            user: caller.clone(),
        })?;

        Ok(native_account.balance)
    }

    fn emit_event<T: Serialize>(
        ctx: &ServiceContext,
        name: String,
        event: T,
    ) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::Codec(err).into(),
            Ok(json) => {
                ctx.emit_event(name, json);
                ServiceResponse::from_succeed(())
            }
        }
    }

    // #[cfg(test)]
    // fn admin(&self) -> Address {
    //     self.sdk
    //         .get_value(&ADMISSION_CONTROL_ADMIN_KEY.to_owned())
    //         .expect("admin not found")
    // }
}
