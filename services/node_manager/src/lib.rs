#[cfg(test)]
mod tests;
mod types;

use bytes::Bytes;
use derive_more::{Display, From};

use binding_macro::{cycles, genesis, service};
use protocol::traits::{ExecutorParams, ServiceSDK};
use protocol::types::{Address, Metadata, ServiceContext};
use protocol::{ProtocolError, ProtocolErrorKind, ProtocolResult};

use crate::types::{
    InitGenesisPayload, SetAdminEvent, SetAdminPayload, UpdateIntervalEvent, UpdateIntervalPayload,
    UpdateMetadataEvent, UpdateMetadataPayload, UpdateRatioEvent, UpdateRatioPayload,
    UpdateValidatorsEvent, UpdateValidatorsPayload,
};

const ADMIN_KEY: &str = "admin";
static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"node_manager");

pub struct NodeManagerService<SDK> {
    sdk: SDK,
}

#[service]
impl<SDK: ServiceSDK> NodeManagerService<SDK> {
    pub fn new(sdk: SDK) -> ProtocolResult<Self> {
        Ok(Self { sdk })
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) -> ProtocolResult<()> {
        self.sdk.set_value(ADMIN_KEY.to_string(), payload.admin)
    }

    #[cycles(210_00)]
    #[read]
    fn get_admin(&self, ctx: ServiceContext) -> ProtocolResult<Address> {
        let admin: Address = self
            .sdk
            .get_value(&ADMIN_KEY.to_owned())?
            .expect("Admin should not be none");
        Ok(admin)
    }

    #[cycles(210_00)]
    #[write]
    fn set_admin(&mut self, ctx: ServiceContext, payload: SetAdminPayload) -> ProtocolResult<()> {
        if self.verify_authority(ctx.get_caller())? {
            self.sdk
                .set_value(ADMIN_KEY.to_owned(), payload.admin.clone())?;

            let event = SetAdminEvent {
                topic: "Set New Admin".to_owned(),
                admin: payload.admin,
            };
            let event_str = serde_json::to_string(&event).map_err(ServiceError::JsonParse)?;
            ctx.emit_event(event_str)
        } else {
            Err(ServiceError::NonAuthorized.into())
        }
    }

    #[cycles(210_00)]
    #[write]
    fn update_metadata(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> ProtocolResult<()> {
        if self.verify_authority(ctx.get_caller())? {
            let metadata_payload_str =
                serde_json::to_string(&payload).map_err(ServiceError::JsonParse)?;
            self.sdk.write(
                &ctx,
                Some(ADMISSION_TOKEN.clone()),
                "metadata",
                "update_metadata",
                &metadata_payload_str,
            )?;

            let event = UpdateMetadataEvent {
                topic:           "Metadata Updated".to_owned(),
                verifier_list:   payload.verifier_list,
                interval:        payload.interval,
                propose_ratio:   payload.propose_ratio,
                prevote_ratio:   payload.prevote_ratio,
                precommit_ratio: payload.precommit_ratio,
                brake_ratio:     payload.brake_ratio,
            };
            let event_str = serde_json::to_string(&event).map_err(ServiceError::JsonParse)?;
            ctx.emit_event(event_str)
        } else {
            Err(ServiceError::NonAuthorized.into())
        }
    }

    #[cycles(210_00)]
    #[write]
    fn update_validators(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateValidatorsPayload,
    ) -> ProtocolResult<()> {
        if self.verify_authority(ctx.get_caller())? {
            let metadata_str = self.sdk.read(&ctx, None, "metadata", "get_metadata", "")?;
            let metadata: Metadata =
                serde_json::from_str(&metadata_str).map_err(ServiceError::JsonParse)?;

            let update_metadata_payload = UpdateMetadataPayload {
                verifier_list:   payload.verifier_list.clone(),
                interval:        metadata.interval,
                propose_ratio:   metadata.propose_ratio,
                prevote_ratio:   metadata.prevote_ratio,
                precommit_ratio: metadata.precommit_ratio,
                brake_ratio:     metadata.brake_ratio,
            };

            let metadata_payload_str =
                serde_json::to_string(&update_metadata_payload).map_err(ServiceError::JsonParse)?;
            self.sdk.write(
                &ctx,
                Some(ADMISSION_TOKEN.clone()),
                "metadata",
                "update_metadata",
                &metadata_payload_str,
            )?;

            let event = UpdateValidatorsEvent {
                topic:         "Validators Updated".to_owned(),
                verifier_list: payload.verifier_list,
            };

            let event_str = serde_json::to_string(&event).map_err(ServiceError::JsonParse)?;
            ctx.emit_event(event_str)
        } else {
            Err(ServiceError::NonAuthorized.into())
        }
    }

    #[cycles(210_00)]
    #[write]
    fn update_interval(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateIntervalPayload,
    ) -> ProtocolResult<()> {
        if self.verify_authority(ctx.get_caller())? {
            let metadata_str = self.sdk.read(&ctx, None, "metadata", "get_metadata", "")?;
            let metadata: Metadata =
                serde_json::from_str(&metadata_str).map_err(ServiceError::JsonParse)?;

            let update_metadata_payload = UpdateMetadataPayload {
                verifier_list:   metadata.verifier_list,
                interval:        payload.interval,
                propose_ratio:   metadata.propose_ratio,
                prevote_ratio:   metadata.prevote_ratio,
                precommit_ratio: metadata.precommit_ratio,
                brake_ratio:     metadata.brake_ratio,
            };

            let metadata_payload_str =
                serde_json::to_string(&update_metadata_payload).map_err(ServiceError::JsonParse)?;
            self.sdk.write(
                &ctx,
                Some(ADMISSION_TOKEN.clone()),
                "metadata",
                "update_metadata",
                &metadata_payload_str,
            )?;

            let event = UpdateIntervalEvent {
                topic:    "Interval Updated".to_owned(),
                interval: payload.interval,
            };

            let event_str = serde_json::to_string(&event).map_err(ServiceError::JsonParse)?;
            ctx.emit_event(event_str)
        } else {
            Err(ServiceError::NonAuthorized.into())
        }
    }

    #[cycles(210_00)]
    #[write]
    fn update_ratio(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateRatioPayload,
    ) -> ProtocolResult<()> {
        if self.verify_authority(ctx.get_caller())? {
            let metadata_str = self.sdk.read(&ctx, None, "metadata", "get_metadata", "")?;
            let metadata: Metadata =
                serde_json::from_str(&metadata_str).map_err(ServiceError::JsonParse)?;

            let update_metadata_payload = UpdateMetadataPayload {
                verifier_list:   metadata.verifier_list,
                interval:        metadata.interval,
                propose_ratio:   payload.propose_ratio,
                prevote_ratio:   payload.prevote_ratio,
                precommit_ratio: payload.precommit_ratio,
                brake_ratio:     payload.brake_ratio,
            };

            let metadata_payload_str =
                serde_json::to_string(&update_metadata_payload).map_err(ServiceError::JsonParse)?;
            self.sdk.write(
                &ctx,
                Some(ADMISSION_TOKEN.clone()),
                "metadata",
                "update_metadata",
                &metadata_payload_str,
            )?;

            let event = UpdateRatioEvent {
                topic:           "Ratio Updated".to_owned(),
                propose_ratio:   payload.propose_ratio,
                prevote_ratio:   payload.prevote_ratio,
                precommit_ratio: payload.precommit_ratio,
                brake_ratio:     payload.brake_ratio,
            };

            let event_str = serde_json::to_string(&event).map_err(ServiceError::JsonParse)?;
            ctx.emit_event(event_str)
        } else {
            Err(ServiceError::NonAuthorized.into())
        }
    }

    fn verify_authority(&self, caller: Address) -> ProtocolResult<bool> {
        let admin: Address = self
            .sdk
            .get_value(&ADMIN_KEY.to_string())?
            .expect("Admin should not be none");

        if caller == admin {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    NonAuthorized,

    #[display(fmt = "Parsing payload to json failed {:?}", _0)]
    JsonParse(serde_json::Error),
}

impl std::error::Error for ServiceError {}

impl From<ServiceError> for ProtocolError {
    fn from(err: ServiceError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Service, Box::new(err))
    }
}
