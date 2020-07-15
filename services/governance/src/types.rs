use std::cmp::Ordering;

use muta_codec_derive::RlpFixedCodec;
use serde::{Deserialize, Serialize};

use protocol::fixed_codec::{FixedCodec, FixedCodecError};
use protocol::types::{Address, Bytes, Hash, Metadata, ValidatorExtend};
use protocol::ProtocolResult;

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug)]
pub struct InitGenesisPayload {
    pub info: GovernanceInfo,
    pub tx_fee_inlet_address: Address,
    pub miner_profit_outlet_address: Address,
    pub miner_charge_map: Vec<MinerChargeConfig>,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug)]
pub struct MinerChargeConfig {
    pub address:              Address,
    pub miner_charge_address: Address,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug)]
pub struct SetMinerEvent {
    pub info: MinerChargeConfig,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, Default)]
pub struct GovernanceInfo {
    pub admin: Address,
    pub tx_failure_fee: u64,
    pub tx_floor_fee: u64,
    pub profit_deduct_rate_per_million: u64,
    pub tx_fee_discount: Vec<DiscountLevel>,
    pub miner_benefit: u64,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscountLevel {
    pub threshold:        u64,
    pub discount_percent: u64,
}

impl PartialOrd for DiscountLevel {
    fn partial_cmp(&self, other: &DiscountLevel) -> Option<Ordering> {
        self.threshold.partial_cmp(&other.threshold)
    }
}

impl Ord for DiscountLevel {
    fn cmp(&self, other: &DiscountLevel) -> Ordering {
        self.threshold.cmp(&other.threshold)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetAdminPayload {
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetGovernInfoPayload {
    pub inner: GovernanceInfo,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetAdminEvent {
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetGovernInfoEvent {
    pub info: GovernanceInfo,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RecordProfitEvent {
    pub owner:  Address,
    pub amount: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateMetadataPayload {
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
    pub timeout_gap:     u64,
    pub cycles_limit:    u64,
    pub cycles_price:    u64,
    pub tx_num_limit:    u64,
    pub max_tx_size:     u64,
}

impl From<Metadata> for UpdateMetadataPayload {
    fn from(metadata: Metadata) -> Self {
        UpdateMetadataPayload {
            verifier_list:   metadata.verifier_list,
            interval:        metadata.interval,
            propose_ratio:   metadata.propose_ratio,
            prevote_ratio:   metadata.prevote_ratio,
            precommit_ratio: metadata.precommit_ratio,
            brake_ratio:     metadata.brake_ratio,
            timeout_gap:     metadata.timeout_gap,
            cycles_limit:    metadata.cycles_limit,
            cycles_price:    metadata.cycles_price,
            tx_num_limit:    metadata.tx_num_limit,
            max_tx_size:     metadata.max_tx_size,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateMetadataEvent {
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
    pub timeout_gap:     u64,
    pub cycles_limit:    u64,
    pub cycles_price:    u64,
    pub tx_num_limit:    u64,
    pub max_tx_size:     u64,
}

impl From<UpdateMetadataPayload> for UpdateMetadataEvent {
    fn from(payload: UpdateMetadataPayload) -> Self {
        UpdateMetadataEvent {
            verifier_list:   payload.verifier_list,
            interval:        payload.interval,
            propose_ratio:   payload.propose_ratio,
            prevote_ratio:   payload.prevote_ratio,
            precommit_ratio: payload.precommit_ratio,
            brake_ratio:     payload.brake_ratio,
            timeout_gap:     payload.timeout_gap,
            cycles_limit:    payload.cycles_limit,
            cycles_price:    payload.cycles_price,
            tx_num_limit:    payload.tx_num_limit,
            max_tx_size:     payload.max_tx_size,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateValidatorsPayload {
    pub verifier_list: Vec<ValidatorExtend>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateValidatorsEvent {
    pub verifier_list: Vec<ValidatorExtend>,
}

impl From<UpdateValidatorsPayload> for UpdateValidatorsEvent {
    fn from(payload: UpdateValidatorsPayload) -> Self {
        UpdateValidatorsEvent {
            verifier_list: payload.verifier_list,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateIntervalPayload {
    pub interval: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateIntervalEvent {
    pub interval: u64,
}

impl From<UpdateIntervalPayload> for UpdateIntervalEvent {
    fn from(payload: UpdateIntervalPayload) -> Self {
        UpdateIntervalEvent {
            interval: payload.interval,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateRatioPayload {
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateRatioEvent {
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

impl From<UpdateRatioPayload> for UpdateRatioEvent {
    fn from(payload: UpdateRatioPayload) -> Self {
        UpdateRatioEvent {
            propose_ratio:   payload.propose_ratio,
            prevote_ratio:   payload.prevote_ratio,
            precommit_ratio: payload.precommit_ratio,
            brake_ratio:     payload.brake_ratio,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccumulateProfitPayload {
    pub address:            Address,
    pub accumulated_profit: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct HookTransferFromPayload {
    pub sender:    Address,
    pub recipient: Address,
    pub value:     u64,
    pub memo:      String,
}

#[derive(RlpFixedCodec, Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct Asset {
    pub id:        Hash,
    pub name:      String,
    pub symbol:    String,
    pub supply:    u64,
    pub precision: u64,
    pub issuers:   Vec<Address>,
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
