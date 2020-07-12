use derive_more::Display;
use protocol::traits::ServiceResponse;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Method {} can not be invoke with call", _0)]
    NotInExecContext(String),

    #[display(fmt = "Contract {} not exists", _0)]
    ContractNotFound(String),

    #[display(fmt = "Code not found")]
    CodeNotFound,

    #[display(fmt = "Abnormal exit, code: {} msg: {}", code, msg)]
    NonZeroExit { code: i8, msg: String },

    #[display(fmt = "VM: {:?}", _0)]
    CkbVm(ckb_vm::Error),

    #[display(fmt = "Codec: {:?}", _0)]
    Serde(serde_json::error::Error),

    #[display(fmt = "Hex decode: {:?}", _0)]
    HexDecode(hex::FromHexError),

    #[display(fmt = "Invalid key '{:?}', should be a hex string", _0)]
    InvalidKey(String),

    #[display(fmt = "Not authorized")]
    NonAuthorized,

    #[display(fmt = "Out of cycles")]
    OutOfCycles,

    #[display(fmt = "Invalid contract address")]
    InvalidContractAddress,

    #[display(fmt = "Write in readonly context")]
    WriteInReadonlyContext,

    #[display(fmt = "Assert failed: {}", _0)]
    AssertFailed(String),
}

impl ServiceError {
    pub fn code(&self) -> u64 {
        match self {
            ServiceError::NotInExecContext(_) => 101,
            ServiceError::ContractNotFound(_) => 102,
            ServiceError::CodeNotFound => 103,
            ServiceError::NonZeroExit { .. } => 104,
            ServiceError::CkbVm(_) => 105,
            ServiceError::Serde(_) => 106,
            ServiceError::HexDecode(_) => 107,
            ServiceError::InvalidKey(_) => 108,
            ServiceError::NonAuthorized => 109,
            ServiceError::OutOfCycles => 110,
            ServiceError::InvalidContractAddress => 111,
            ServiceError::WriteInReadonlyContext => 112,
            ServiceError::AssertFailed(_) => 113,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
