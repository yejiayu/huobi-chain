use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use muta_codec_derive::RlpFixedCodec;
use protocol::fixed_codec::{FixedCodec, FixedCodecError};
use protocol::types::{Address, Hash, Hex};
use protocol::ProtocolResult;

use crate::ServiceError;

/// Payload
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct InitGenesisPayload {
    pub id:          Hash,
    pub name:        String,
    pub symbol:      String,
    pub supply:      u64,
    pub precision:   u64,
    pub issuer:      Address,
    pub fee_account: Address,
    pub fee:         u64,
    pub admin:       Address,
}

impl InitGenesisPayload {
    pub fn verify(&self) -> Result<(), &'static str> {
        if self.id == Hash::default() {
            return Err("invalid asset id");
        }
        if self.issuer == Address::default() {
            return Err("invalid issuer");
        }
        if self.fee_account == Address::default() {
            return Err("invalid fee account");
        }
        if self.admin == Address::default() {
            return Err("invalid admin");
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CreateAssetPayload {
    pub name:      String,
    pub symbol:    String,
    pub supply:    u64,
    pub precision: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetAssetPayload {
    pub id: Hash,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransferPayload {
    pub asset_id: Hash,
    pub to:       Address,
    pub value:    u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransferEvent {
    pub asset_id: Hash,
    pub from:     Address,
    pub to:       Address,
    pub value:    u64,
}

pub type ApprovePayload = TransferPayload;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ApproveEvent {
    pub asset_id: Hash,
    pub grantor:  Address,
    pub grantee:  Address,
    pub value:    u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransferFromPayload {
    pub asset_id:  Hash,
    pub sender:    Address,
    pub recipient: Address,
    pub value:     u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TransferFromEvent {
    pub asset_id:  Hash,
    pub caller:    Address,
    pub sender:    Address,
    pub recipient: Address,
    pub value:     u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetBalancePayload {
    pub asset_id: Hash,
    pub user:     Address,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GetBalanceResponse {
    pub asset_id: Hash,
    pub user:     Address,
    pub balance:  u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetAllowancePayload {
    pub asset_id: Hash,
    pub grantor:  Address,
    pub grantee:  Address,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct GetAllowanceResponse {
    pub asset_id: Hash,
    pub grantor:  Address,
    pub grantee:  Address,
    pub value:    u64,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct Asset {
    pub id:        Hash,
    pub name:      String,
    pub symbol:    String,
    pub supply:    u64,
    pub precision: u64,
    pub issuer:    Address,
}

pub struct AssetBalance {
    pub value:     u64,
    pub allowance: BTreeMap<Address, u64>,
}

impl AssetBalance {
    pub fn new(supply: u64) -> Self {
        AssetBalance {
            value:     supply,
            allowance: BTreeMap::new(),
        }
    }

    pub fn checked_add(&mut self, amount: u64) -> Result<(), ServiceError> {
        let (checked_value, overflow) = self.value.overflowing_add(amount);
        if overflow {
            return Err(ServiceError::BalanceOverflow);
        }

        self.value = checked_value;
        Ok(())
    }

    pub fn checked_sub(&mut self, amount: u64) -> Result<(), ServiceError> {
        let (checked_value, overflow) = self.value.overflowing_sub(amount);
        if overflow {
            return Err(ServiceError::BalanceOverflow);
        }

        self.value = checked_value;
        Ok(())
    }

    pub fn allowance(&self, spender: &Address) -> u64 {
        *self.allowance.get(spender).unwrap_or_else(|| &0)
    }

    pub fn update_allowance(&mut self, spender: Address, value: u64) {
        self.allowance
            .entry(spender)
            .and_modify(|b| *b = value)
            .or_insert(value);
    }
}

impl Deref for AssetBalance {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for AssetBalance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl PartialOrd<u64> for AssetBalance {
    fn partial_cmp(&self, other: &u64) -> Option<Ordering> {
        Some(self.value.cmp(other))
    }
}

impl PartialEq<u64> for AssetBalance {
    fn eq(&self, other: &u64) -> bool {
        self.value == *other
    }
}

#[derive(RlpFixedCodec)]
struct AllowanceCodec {
    pub addr:  Address,
    pub total: u64,
}

impl rlp::Decodable for AssetBalance {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let value = rlp.at(0)?.as_val()?;
        let codec_list: Vec<AllowanceCodec> = rlp::decode_list(rlp.at(1)?.as_raw());
        let mut allowance = BTreeMap::new();
        for v in codec_list {
            allowance.insert(v.addr, v.total);
        }

        Ok(AssetBalance { value, allowance })
    }
}

impl rlp::Encodable for AssetBalance {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(2);
        s.append(&self.value);

        let mut codec_list = Vec::with_capacity(self.allowance.len());

        for (address, allowance) in self.allowance.iter() {
            let fixed_codec = AllowanceCodec {
                addr:  address.clone(),
                total: *allowance,
            };

            codec_list.push(fixed_codec);
        }

        s.append_list(&codec_list);
    }
}

impl FixedCodec for AssetBalance {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        Ok(Bytes::from(rlp::encode(self)))
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        Ok(rlp::decode(bytes.as_ref()).map_err(FixedCodecError::from)?)
    }
}

impl Default for AssetBalance {
    fn default() -> Self {
        AssetBalance {
            value:     0,
            allowance: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintAsset {
    pub asset_id: Hash,
    pub to:       Address,
    pub amount:   u64,
    pub proof:    Hex,
    pub memo:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnAsset {
    pub asset_id: Hash,
    pub amount:   u64,
    pub proof:    Hex,
    pub memo:     String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintEvent {
    pub asset_id: Hash,
    pub to:       Address,
    pub amount:   u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BurnEvent {
    pub asset_id: Hash,
    pub from:     Address,
    pub amount:   u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAdmin {
    pub addr: Address,
}
