use serde::{Deserialize, Serialize};

use protocol::types::{Metadata, ValidatorExtend};

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
