use crate::{expression::ExpressionError, types::OrgName};

use derive_more::{Display, From};
use protocol::traits::ServiceResponse;

#[derive(Debug, Display, From)]
pub enum ServiceError {
    #[display(fmt = "Bad payload: {}", _0)]
    BadPayload(String),

    #[display(fmt = "Codec: {}", _0)]
    Serde(serde_json::error::Error),

    #[display(fmt = "Kyc org {} not found", _0)]
    OrgNotFound(OrgName),

    #[display(fmt = "Non authorized")]
    NonAuthorized,

    #[display(fmt = "Org already exists")]
    OrgAlreadyExists,

    #[display(fmt = "Out of cycles")]
    OutOfCycles,

    #[display(fmt = "Expression {}", _0)]
    Expression(ExpressionError),

    #[display(fmt = "Unapproved org")]
    UnapprovedOrg,
}

impl ServiceError {
    pub fn code(&self) -> u64 {
        use ServiceError::*;

        match self {
            BadPayload(_) => 101,
            Serde(_) => 102,
            OrgNotFound(_) => 103,
            NonAuthorized => 104,
            OrgAlreadyExists => 105,
            OutOfCycles => 106,
            Expression(_) => 107,
            UnapprovedOrg => 108,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
