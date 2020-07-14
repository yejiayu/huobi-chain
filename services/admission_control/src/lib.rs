#[cfg(test)]
mod tests;
mod types;

use binding_macro::{cycles, genesis, service, write};
use derive_more::Display;
use protocol::{
    traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap},
    types::{Address, ServiceContext, SignedTransaction},
};
use serde::{Deserialize, Serialize};

use types::{AddressList, Genesis, NewAdmin, StatusList, Validate};

macro_rules! require_admin {
    ($service:expr, $ctx:expr) => {{
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
    }};
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
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}

const ADMISSION_CONTROL_ADMIN_KEY: &str = "admission_control_admin";

#[derive(Debug)]
struct ServicePayload<P: Serialize> {
    name:    &'static str,
    method:  &'static str,
    payload: P,
}

pub struct AdmissionControlService<SDK> {
    sdk:       SDK,
    deny_list: Box<dyn StoreMap<Address, bool>>,
}

#[service]
impl<SDK: ServiceSDK + 'static> AdmissionControlService<SDK> {
    pub fn new(mut sdk: SDK) -> Self {
        let deny_list = sdk.alloc_or_recover_map("admission_control_deny_list");

        AdmissionControlService { sdk, deny_list }
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

        let failure_fee: u64 = match self.sdk_read(&ctx, ServicePayload {
            name:    "governance",
            method:  "get_tx_failure_fee",
            payload: "",
        }) {
            Ok(fee) => fee,
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
        #[derive(Debug, Deserialize)]
        struct Asset {
            pub id: protocol::types::Hash,
        }

        #[derive(Debug, Deserialize)]
        struct AssetAccount {
            pub balance: u64,
        }

        let native_asset: Asset = self.sdk_read(ctx, ServicePayload {
            name:    "asset",
            method:  "get_native_asset",
            payload: "",
        })?;
        let asset_id = native_asset.id;

        let native_account: AssetAccount = self.sdk_read(ctx, ServicePayload {
            name:    "asset",
            method:  "get_balance",
            payload: serde_json::json!({
                "asset_id": asset_id,
                "user": caller,
            }),
        })?;

        Ok(native_account.balance)
    }

    fn sdk_read<P: Serialize, R: for<'a> Deserialize<'a>>(
        &self,
        ctx: &ServiceContext,
        service_payload: ServicePayload<P>,
    ) -> Result<R, ServiceResponse<()>> {
        let ServicePayload {
            name,
            method,
            payload,
        } = service_payload;
        let json_payload = match serde_json::to_string(&payload) {
            Ok(p) => p,
            Err(e) => return Err(ServiceError::Codec(e).into()),
        };

        let resp = self.sdk.read(&ctx, None, name, method, &json_payload);
        if resp.is_error() {
            return Err(ServiceResponse::from_error(resp.code, resp.error_message));
        }

        serde_json::from_str(&resp.succeed_data).map_err(|e| ServiceError::Codec(e).into())
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

    #[cfg(test)]
    fn admin(&self) -> Address {
        self.sdk
            .get_value(&ADMISSION_CONTROL_ADMIN_KEY.to_owned())
            .expect("admin not found")
    }
}
