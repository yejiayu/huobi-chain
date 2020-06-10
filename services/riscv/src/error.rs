use derive_more::{Display, From};
use protocol::traits::ServiceResponse;

#[derive(Debug, Display, From)]
pub enum ServiceError {
    #[display(fmt = "Method {} can not be invoke with call", _0)]
    NotInExecContext(String),

    #[display(fmt = "Contract {} not exists", _0)]
    ContractNotFound(String),

    #[display(fmt = "Code not found")]
    CodeNotFound,

    #[display(fmt = "None zero exit {} msg {}", exitcode, ret)]
    NonZeroExitCode { exitcode: i8, ret: String },

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
}

impl ServiceError {
    fn code(&self) -> u64 {
        use ServiceError::*;

        match self {
            NotInExecContext(_) => 101,
            ContractNotFound(_) => 102,
            CodeNotFound => 103,
            NonZeroExitCode { .. } => 104,
            CkbVm(_) => 105,
            Serde(_) => 106,
            HexDecode(_) => 107,
            InvalidKey(_) => 108,
            NonAuthorized => 109,
            OutOfCycles => 110,
            InvalidContractAddress => 111,
            WriteInReadonlyContext => 112,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
