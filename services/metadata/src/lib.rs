#[cfg(test)]
mod tests;
mod types;

use bytes::Bytes;
use derive_more::{Display, From};

use binding_macro::{cycles, genesis, service};
use protocol::traits::{ExecutorParams, ServiceSDK};
use protocol::types::{Metadata, ServiceContext, METADATA_KEY};
use protocol::{ProtocolError, ProtocolErrorKind, ProtocolResult};

use crate::types::UpdateMetadataPayload;

static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"node_manager");

pub struct MetadataService<SDK> {
    sdk: SDK,
}

#[service]
impl<SDK: ServiceSDK> MetadataService<SDK> {
    pub fn new(sdk: SDK) -> ProtocolResult<Self> {
        Ok(Self { sdk })
    }

    #[genesis]
    fn init_genesis(&mut self, metadata: Metadata) -> ProtocolResult<()> {
        self.sdk.set_value(METADATA_KEY.to_string(), metadata)
    }

    #[cycles(210_00)]
    #[read]
    fn get_metadata(&self, ctx: ServiceContext) -> ProtocolResult<Metadata> {
        let metadata: Metadata = self
            .sdk
            .get_value(&METADATA_KEY.to_owned())?
            .expect("Metadata should always be in the genesis block");
        Ok(metadata)
    }

    #[cycles(210_00)]
    #[write]
    fn update_metadata(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> ProtocolResult<()> {
        if let Some(extra) = ctx.get_extra() {
            if extra == ADMISSION_TOKEN {
                let mut metadata: Metadata = self
                    .sdk
                    .get_value(&METADATA_KEY.to_owned())?
                    .expect("Metadata should always be in the genesis block");
                metadata.verifier_list = payload.verifier_list;
                metadata.interval = payload.interval;
                metadata.precommit_ratio = payload.precommit_ratio;
                metadata.prevote_ratio = payload.prevote_ratio;
                metadata.propose_ratio = payload.propose_ratio;
                self.sdk.set_value(METADATA_KEY.to_string(), metadata)
            } else {
                Err(ServiceError::AdmissionFail.into())
            }
        } else {
            Err(ServiceError::NoneAdmission.into())
        }
    }
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    NoneAdmission,

    AdmissionFail,
}

impl std::error::Error for ServiceError {}

impl From<ServiceError> for ProtocolError {
    fn from(err: ServiceError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Service, Box::new(err))
    }
}
