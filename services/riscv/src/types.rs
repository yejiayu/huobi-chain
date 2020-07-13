use crate::ServiceError;

use serde::{Deserialize, Serialize};

use protocol::fixed_codec::{FixedCodec, FixedCodecError};
use protocol::types::{Address, Hash};
use protocol::{Bytes, ProtocolResult};

use std::{convert::TryFrom, ops::Deref};

#[repr(u8)]
#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub enum InterpreterType {
    Binary = 1,
}

impl TryFrom<u8> for InterpreterType {
    type Error = &'static str;

    fn try_from(val: u8) -> Result<InterpreterType, Self::Error> {
        match val {
            1 => Ok(InterpreterType::Binary),
            _ => Err("unsupport interpreter"),
        }
    }
}

impl Default for InterpreterType {
    fn default() -> Self {
        Self::Binary
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DeployPayload {
    pub code:      String,
    #[serde(default)]
    pub intp_type: InterpreterType,
    pub init_args: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct DeployResp {
    pub address:  Address,
    pub init_ret: String,
}

#[derive(Debug, Clone)]
pub struct ExecArgs(String);

impl From<String> for ExecArgs {
    fn from(args: String) -> ExecArgs {
        ExecArgs(args)
    }
}

impl From<Bytes> for ExecArgs {
    fn from(args: Bytes) -> ExecArgs {
        ExecArgs(String::from_utf8_lossy(args.as_ref()).to_string())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ExecPayload {
    pub address: Address,
    pub args:    String,
}

impl ExecPayload {
    pub fn new<A: Into<ExecArgs>>(address: Address, args: A) -> ExecPayload {
        let args: ExecArgs = args.into();

        Self {
            address,
            args: args.0,
        }
    }

    pub fn json(&self) -> Result<String, ServiceError> {
        serde_json::to_string(self).map_err(ServiceError::Serde)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct ExecResp {
    pub ret:      String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct Authorizer(Option<Address>);

impl Authorizer {
    pub fn new(addr: Address) -> Self {
        Authorizer(Some(addr))
    }

    pub fn none() -> Self {
        Authorizer(None)
    }

    pub fn inner(self) -> Option<Address> {
        self.0
    }
}

impl Deref for Authorizer {
    type Target = Option<Address>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FixedCodec for Authorizer {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(&self.0).into())
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        let v: Option<Address> = rlp::decode(&bytes).map_err(FixedCodecError::from)?;

        Ok(Authorizer(v))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Contract {
    pub code_hash:  Hash,
    pub intp_type:  InterpreterType,
    pub authorizer: Option<Address>,
}

impl Contract {
    pub fn new(code_hash: Hash, intp_type: InterpreterType) -> Contract {
        Self {
            code_hash,
            intp_type,
            authorizer: None,
        }
    }
}

impl FixedCodec for Contract {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(self).into())
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        Ok(rlp::decode(&bytes).map_err(FixedCodecError::from)?)
    }
}

impl rlp::Encodable for Contract {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(3)
            .append(&self.code_hash)
            .append(&(self.intp_type as u8))
            .append(&self.authorizer);
    }
}

impl rlp::Decodable for Contract {
    fn decode(r: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let code_hash: Hash = r.val_at(0)?;
        let intp_type = {
            let t: u8 = r.val_at(1)?;
            InterpreterType::try_from(t).map_err(rlp::DecoderError::Custom)?
        };
        let authorizer: Option<Address> = r.val_at(2)?;

        Ok(Contract {
            code_hash,
            intp_type,
            authorizer,
        })
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetContractPayload {
    pub address:      Address,
    #[serde(default)]
    pub get_code:     bool,
    #[serde(default)]
    pub storage_keys: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct AddressList {
    pub addresses: Vec<Address>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GetContractResp {
    pub code_hash:      Hash,
    pub intp_type:      InterpreterType,
    pub code:           String,
    pub storage_values: Vec<String>,
    pub authorizer:     Option<Address>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct InitGenesisPayload {
    #[serde(default)]
    pub enable_authorization: bool,
    #[serde(default)]
    pub deploy_auth:          Vec<Address>,
    #[serde(default)]
    pub admins:               Vec<Address>,
}
