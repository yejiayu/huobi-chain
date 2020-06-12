use protocol::{types::Address, Bytes};
use serde::{Deserialize, Serialize};

use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Genesis {
    pub admin:      Address,
    pub block_list: Vec<Address>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewAdmin {
    pub new_admin: Address,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AddressList {
    pub addrs: Vec<Address>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UnverifiedTransaction {
    pub stx_bytes: Bytes,
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
