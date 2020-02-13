use serde::{Deserialize, Serialize};

use protocol::types::{Address, ValidatorExtend};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct InitGenesisPayload {
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetAdminPayload {
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SetAdminEvent {
    pub topic: String,
    pub admin: Address,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateMetadataPayload {
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateMetadataEvent {
    pub topic:           String,
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateValidatorsPayload {
    pub verifier_list: Vec<ValidatorExtend>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateValidatorsEvent {
    pub topic:         String,
    pub verifier_list: Vec<ValidatorExtend>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateIntervalPayload {
    pub interval: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateIntervalEvent {
    pub topic:    String,
    pub interval: u64,
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
    pub topic:           String,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
}
