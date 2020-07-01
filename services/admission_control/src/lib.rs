#[cfg(test)]
mod tests;
mod types;

use binding_macro::{cycles, genesis, read, service, write};
use derive_more::Display;
use protocol::{
    traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap},
    types::{Address, ServiceContext, SignedTransaction},
};
use serde::Serialize;

use types::{AddressList, Event, Genesis, NewAdmin, StatusList, UnverifiedTransaction, Validate};

macro_rules! require_admin {
    ($service:expr, $ctx:expr) => {{
        let admin = $service
            .sdk
            .get_value(&ADMISSION_CONTROL_ADMIN_KEY.to_owned())
            .expect("admin not found");

        if $ctx.get_caller() != admin {
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
}

impl ServiceError {
    pub fn code(&self) -> u64 {
        match self {
            ServiceError::NonAuthorized => 1000,
            ServiceError::Codec(_) => 1001,
            ServiceError::OutOfCycles => 1002,
            ServiceError::BlockedTx => 1003,
            ServiceError::BadPayload(_) => 1004,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}

const ADMISSION_CONTROL_ADMIN_KEY: &str = "admission_control_admin";

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

    #[read]
    fn is_valid(
        &self,
        ctx: ServiceContext,
        _payload: UnverifiedTransaction,
    ) -> ServiceResponse<()> {
        if self.deny_list.contains(&ctx.get_caller()) {
            return ServiceError::BlockedTx.into();
        }

        // TODO: verify signature
        // TODO: verify balance

        ServiceResponse::from_succeed(())
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

        Self::emit_event(&ctx, Event {
            topic: "change_admin".to_owned(),
            data:  payload,
        })
    }

    #[write]
    fn forbid(&mut self, ctx: ServiceContext, payload: AddressList) -> ServiceResponse<()> {
        require_admin!(self, ctx);
        sub_cycles!(ctx, payload.addrs.len() as u64 * 10_000);

        for addr in payload.addrs.iter() {
            self.deny_list.insert(addr.to_owned(), true);
        }

        Self::emit_event(&ctx, Event {
            topic: "forbid".to_owned(),
            data:  payload,
        })
    }

    #[write]
    fn permit(&mut self, ctx: ServiceContext, payload: AddressList) -> ServiceResponse<()> {
        require_admin!(self, ctx);
        sub_cycles!(ctx, payload.addrs.len() as u64 * 10_000);

        for addr in payload.addrs.iter() {
            self.deny_list.remove(addr);
        }

        Self::emit_event(&ctx, Event {
            topic: "permit".to_owned(),
            data:  payload,
        })
    }

    fn emit_event<T: Serialize>(ctx: &ServiceContext, event: T) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::Codec(err).into(),
            Ok(json) => {
                ctx.emit_event(json);
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
