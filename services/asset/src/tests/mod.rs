use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;
use cita_trie::MemoryDB;

use framework::binding::sdk::{DefalutServiceSDK, DefaultChainQuerier};
use framework::binding::state::{GeneralServiceState, MPTTrie};
use protocol::traits::{Context, NoopDispatcher, Storage};
use protocol::types::{
    Address, Block, Hash, Proof, Receipt, ServiceContext, ServiceContextParams, SignedTransaction,
};
use protocol::{types::Bytes, ProtocolResult};

use crate::types::{
    ApprovePayload, CreateAssetPayload, GetAllowancePayload, GetAssetPayload, GetBalancePayload,
    TransferFromPayload, TransferPayload,
};
use crate::AssetService;

macro_rules! service_call {
    ($service:expr, $method:ident, $ctx:expr, $payload:expr) => {{
        let resp = $service.$method($ctx, $payload);
        assert!(!resp.is_error());

        resp.succeed_data
    }};
}

macro_rules! create_asset {
    ($service:expr, $ctx:expr, $supply:expr, $precision:expr) => {{
        service_call!($service, create_asset, $ctx, CreateAssetPayload {
            name:      "test".to_owned(),
            symbol:    "test".to_owned(),
            supply:    $supply,
            precision: $precision,
        })
    }};
}

#[test]
fn test_create_asset() {
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let supply = 1024 * 1024;
    let precision = 2;
    let caller = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let ctx = mock_context(cycles_limit, caller.clone());

    let mut service = new_asset_service();

    // test create_asset
    let asset = create_asset!(service, ctx.clone(), supply, precision);
    let asset_got = service_call!(service, get_asset, ctx.clone(), GetAssetPayload {
        id: asset.id.clone(),
    });
    assert_eq!(asset_got, asset);

    let asset_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });

    assert_eq!(asset_balance.balance, supply);
    assert_eq!(asset_balance.asset_id, asset.id);
}

#[test]
fn test_transfer() {
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let supply = 1024 * 1024;
    let precision = 2;
    let caller = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let to_address = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let ctx = mock_context(cycles_limit, caller.clone());

    let mut service = new_asset_service();
    let asset = create_asset!(service, ctx.clone(), supply, precision);

    service_call!(service, transfer, ctx.clone(), TransferPayload {
        asset_id: asset.id.clone(),
        to:       to_address.clone(),
        value:    1024,
    });

    let asset_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });
    assert_eq!(asset_balance.balance, supply - 1024);

    let ctx = mock_context(cycles_limit, to_address.clone());
    let asset_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id,
        user:     to_address,
    });
    assert_eq!(asset_balance.balance, 1024);
}

#[test]
fn test_approve() {
    let supply = 1024 * 1024;
    let precision = 2;
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let caller = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let ctx = mock_context(cycles_limit, caller.clone());

    let mut service = new_asset_service();
    let asset = create_asset!(service, ctx.clone(), supply, precision);

    let to_address = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    service_call!(service, approve, ctx.clone(), ApprovePayload {
        asset_id: asset.id.clone(),
        to:       to_address.clone(),
        value:    1024,
    });

    let allowance = service_call!(service, get_allowance, ctx, GetAllowancePayload {
        asset_id: asset.id.clone(),
        grantor:  caller,
        grantee:  to_address.clone(),
    });
    assert_eq!(allowance.asset_id, asset.id);
    assert_eq!(allowance.grantee, to_address);
    assert_eq!(allowance.value, 1024);
}

#[test]
fn test_transfer_from() {
    let supply = 1024 * 1024;
    let precision = 2;
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let caller = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let ctx = mock_context(cycles_limit, caller.clone());

    let mut service = new_asset_service();
    let asset = create_asset!(service, ctx.clone(), supply, precision);

    let to_address = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    service_call!(service, approve, ctx.clone(), ApprovePayload {
        asset_id: asset.id.clone(),
        to:       to_address.clone(),
        value:    1024,
    });

    let to_ctx = mock_context(cycles_limit, to_address.clone());
    service_call!(
        service,
        transfer_from,
        to_ctx.clone(),
        TransferFromPayload {
            asset_id:  asset.id.clone(),
            sender:    caller.clone(),
            recipient: to_address.clone(),
            value:     24,
        }
    );

    let allowance = service_call!(service, get_allowance, ctx.clone(), GetAllowancePayload {
        asset_id: asset.id.clone(),
        grantor:  caller.clone(),
        grantee:  to_address.clone(),
    });
    assert_eq!(allowance.asset_id, asset.id.clone());
    assert_eq!(allowance.grantee, to_address.clone());
    assert_eq!(allowance.value, 1000);

    let asset_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });
    assert_eq!(asset_balance.balance, supply - 24);

    let asset_balance = service_call!(service, get_balance, to_ctx, GetBalancePayload {
        asset_id: asset.id,
        user:     to_address,
    });
    assert_eq!(asset_balance.balance, 24);
}

fn new_asset_service() -> AssetService<
    DefalutServiceSDK<
        GeneralServiceState<MemoryDB>,
        DefaultChainQuerier<MockStorage>,
        NoopDispatcher,
    >,
> {
    let chain_db = DefaultChainQuerier::new(Arc::new(MockStorage {}));
    let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
    let state = GeneralServiceState::new(trie);

    let sdk = DefalutServiceSDK::new(
        Rc::new(RefCell::new(state)),
        Rc::new(chain_db),
        NoopDispatcher {},
    );

    AssetService::new(sdk)
}

fn mock_context(cycles_limit: u64, caller: Address) -> ServiceContext {
    let params = ServiceContextParams {
        tx_hash: None,
        nonce: None,
        cycles_limit,
        cycles_price: 1,
        cycles_used: Rc::new(RefCell::new(0)),
        caller,
        height: 1,
        timestamp: 0,
        service_name: "service_name".to_owned(),
        service_method: "service_method".to_owned(),
        service_payload: "service_payload".to_owned(),
        extra: None,
        events: Rc::new(RefCell::new(vec![])),
    };

    ServiceContext::new(params)
}

struct MockStorage;

#[async_trait]
impl Storage for MockStorage {
    async fn insert_transactions(
        &self,
        _: Context,
        _: u64,
        _: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn get_transactions(
        &self,
        _: Context,
        _: u64,
        _: Vec<Hash>,
    ) -> ProtocolResult<Vec<Option<SignedTransaction>>> {
        unimplemented!()
    }

    async fn get_transaction_by_hash(
        &self,
        _: Context,
        _: Hash,
    ) -> ProtocolResult<Option<SignedTransaction>> {
        unimplemented!()
    }

    async fn insert_block(&self, _: Context, _: Block) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn get_block(&self, _: Context, _: u64) -> ProtocolResult<Option<Block>> {
        unimplemented!()
    }

    async fn insert_receipts(&self, _: Context, _: u64, _: Vec<Receipt>) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn get_receipt_by_hash(&self, _: Context, _: Hash) -> ProtocolResult<Option<Receipt>> {
        unimplemented!()
    }

    async fn get_receipts(
        &self,
        _: Context,
        _: u64,
        _: Vec<Hash>,
    ) -> ProtocolResult<Vec<Option<Receipt>>> {
        unimplemented!()
    }

    async fn update_latest_proof(&self, _: Context, _: Proof) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn get_latest_proof(&self, _: Context) -> ProtocolResult<Proof> {
        unimplemented!()
    }

    async fn get_latest_block(&self, _: Context) -> ProtocolResult<Block> {
        unimplemented!()
    }

    async fn update_overlord_wal(&self, _: Context, _: Bytes) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn load_overlord_wal(&self, _: Context) -> ProtocolResult<Bytes> {
        unimplemented!()
    }
}
