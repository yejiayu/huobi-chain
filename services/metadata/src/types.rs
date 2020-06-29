use serde::{Deserialize, Serialize};

use protocol::types::ValidatorExtend;

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
