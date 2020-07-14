use super::types::{GetBalanceResponse, UpdateIntervalPayload};
use super::*;

#[test]
fn test_governance() {
    let res = exec_txs!(
        u64::max_value(),
        600_000,
        ("governance", "update_interval", UpdateIntervalPayload {
            interval: 5000,
        })
    );

    assert_eq!(res.fee_inlet_balance, 150);
    assert_eq!(res.proposer_balance, 10);
}
