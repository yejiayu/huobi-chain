use std::str::FromStr;

use crate::ServiceError;

use protocol::types::Address;
use serde::{Deserialize, Serialize};

pub trait Validate {
    fn validate(&self) -> Result<(), ServiceError>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Genesis {
    pub admin:     Address,
    pub deny_list: Vec<Address>,
}

impl Validate for Genesis {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.admin == Address::default() {
            Err(ServiceError::BadPayload("invalid admin address"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewAdmin {
    pub new_admin: Address,
}

impl Validate for NewAdmin {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.new_admin == Address::default() {
            Err(ServiceError::BadPayload("invalid admin address"))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddressList {
    pub addrs: Vec<Address>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct StatusList {
    pub status: Vec<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event<Data> {
    pub topic: String,
    pub data:  Data,
}

impl<Data: for<'a> Deserialize<'a>> FromStr for Event<Data> {
    type Err = serde_json::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(str)
    }
}
