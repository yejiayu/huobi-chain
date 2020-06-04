#[cfg(test)]
mod tests;
mod types;

use bytes::Bytes;
use derive_more::{Display, From};

use binding_macro::{cycles, genesis, service};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK};
use protocol::types::{Metadata, ServiceContext, METADATA_KEY};

use crate::types::UpdateMetadataPayload;

static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"node_manager");

pub struct MetadataService<SDK> {
    sdk: SDK,
}

#[service]
impl<SDK: ServiceSDK> MetadataService<SDK> {
    pub fn new(sdk: SDK) -> Self {
        Self { sdk }
    }

    #[genesis]
    fn init_genesis(&mut self, metadata: Metadata) {
        self.sdk.set_value(METADATA_KEY.to_string(), metadata)
    }

    #[cycles(210_00)]
    #[read]
    fn get_metadata(&self, ctx: ServiceContext) -> ServiceResponse<Metadata> {
        let metadata: Metadata = self
            .sdk
            .get_value(&METADATA_KEY.to_owned())
            .expect("Metadata should always be in the genesis block");

        ServiceResponse::from_succeed(metadata)
    }

    #[cycles(210_00)]
    #[write]
    fn update_metadata(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> ServiceResponse<()> {
        match ctx.get_extra() {
            Some(extra) if extra == ADMISSION_TOKEN => {
                let mut metadata: Metadata = self
                    .sdk
                    .get_value(&METADATA_KEY.to_owned())
                    .expect("Metadata should always be in the genesis block");

                metadata.verifier_list = payload.verifier_list;
                metadata.interval = payload.interval;
                metadata.precommit_ratio = payload.precommit_ratio;
                metadata.prevote_ratio = payload.prevote_ratio;
                metadata.propose_ratio = payload.propose_ratio;
                metadata.brake_ratio = payload.brake_ratio;

                self.sdk.set_value(METADATA_KEY.to_string(), metadata);

                ServiceResponse::from_succeed(())
            }
            Some(_) => ServiceError::AdmissionFail.into(),
            None => ServiceError::NoneAdmission.into(),
        }
    }
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    NoneAdmission,
    AdmissionFail,
}

impl ServiceError {
    fn code(&self) -> u64 {
        match self {
            ServiceError::NoneAdmission => 101,
            ServiceError::AdmissionFail => 102,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
