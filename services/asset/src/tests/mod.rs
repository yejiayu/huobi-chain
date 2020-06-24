use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;

use cita_trie::MemoryDB;
use core_storage::{adapter::memory::MemoryAdapter, ImplStorage};
use framework::binding::sdk::{DefalutServiceSDK, DefaultChainQuerier};
use framework::binding::state::{GeneralServiceState, MPTTrie};
use protocol::traits::NoopDispatcher;
use protocol::types::{Address, Bytes, Hash, Hex, ServiceContext, ServiceContextParams};

use crate::types::{
    ApprovePayload, BurnAsset, BurnEvent, CreateAssetPayload, GetAllowancePayload, GetAssetPayload,
    GetBalancePayload, InitGenesisPayload, MintAsset, MintEvent, NewAdmin, TransferFromPayload,
    TransferPayload,
};
use crate::AssetService;

macro_rules! service_call {
    ($service:expr, $method:ident, $ctx:expr, $payload:expr) => {{
        let resp = $service.$method($ctx, $payload);
        if resp.is_error() {
            println!("{}", resp.error_message);
        }
        assert!(!resp.is_error());

        resp.succeed_data
    }};
}

macro_rules! create_asset {
    ($service:expr, $ctx:expr, $supply:expr, $precision:expr) => {{
        service_call!($service, create_asset, $ctx, CreateAssetPayload {
            name:      "miao".to_owned(),
            symbol:    "😺".to_owned(),
            supply:    $supply,
            precision: $precision,
        })
    }};
}

type SDK = DefalutServiceSDK<
    GeneralServiceState<MemoryDB>,
    DefaultChainQuerier<ImplStorage<MemoryAdapter>>,
    NoopDispatcher,
>;

const CYCLE_LIMIT: u64 = 1024 * 1024 * 1024;
const ADMIN: &str = "0x755cdba6ae4f479f7164792b318b2a06c759833b";
const CALLER: &str = "0x0000000000000000000000000000000000000001";

#[test]
fn test_create_asset() {
    let precision = 2;
    let supply = 1024 * 1024;
    let caller = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    let mut service = TestService::new();
    let ctx = mock_context(caller.clone());

    // test create_asset
    let asset = create_asset!(service, ctx.clone(), supply, precision);
    let asset_got = service_call!(service, get_asset, ctx.clone(), GetAssetPayload {
        id: asset.id.clone(),
    });
    assert_eq!(asset_got, asset);

    let resp = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });
    assert_eq!(resp.balance, supply);
    assert_eq!(resp.asset_id, asset.id);
}

#[test]
fn test_transfer() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller.clone());
    let asset = create_asset!(service, ctx.clone(), 10000, 10);

    let recipient = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    service_call!(service, transfer, ctx.clone(), TransferPayload {
        asset_id: asset.id.clone(),
        to:       recipient.clone(),
        value:    1024,
    });

    let caller_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });
    assert_eq!(caller_balance.balance, asset.supply - 1024);

    let ctx = mock_context(recipient.clone());
    let recipient_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id,
        user:     recipient,
    });
    assert_eq!(recipient_balance.balance, 1024);
}

#[test]
fn test_approve() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller.clone());
    let asset = create_asset!(service, ctx.clone(), 1000, 10);

    let recipient = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    service_call!(service, approve, ctx.clone(), ApprovePayload {
        asset_id: asset.id.clone(),
        to:       recipient.clone(),
        value:    1024,
    });

    let allowance = service_call!(service, get_allowance, ctx, GetAllowancePayload {
        asset_id: asset.id.clone(),
        grantor:  caller,
        grantee:  recipient.clone(),
    });
    assert_eq!(allowance.asset_id, asset.id);
    assert_eq!(allowance.grantee, recipient);
    assert_eq!(allowance.value, 1024);
}

#[test]
fn test_transfer_from() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller.clone());
    let asset = create_asset!(service, ctx.clone(), 1000, 10);

    let recipient = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    service_call!(service, approve, ctx.clone(), ApprovePayload {
        asset_id: asset.id.clone(),
        to:       recipient.clone(),
        value:    1024,
    });

    let recipient_ctx = mock_context(recipient.clone());
    service_call!(
        service,
        transfer_from,
        recipient_ctx.clone(),
        TransferFromPayload {
            asset_id:  asset.id.clone(),
            sender:    caller.clone(),
            recipient: recipient.clone(),
            value:     24,
        }
    );

    let allowance = service_call!(service, get_allowance, ctx.clone(), GetAllowancePayload {
        asset_id: asset.id.clone(),
        grantor:  caller.clone(),
        grantee:  recipient.clone(),
    });
    assert_eq!(allowance.asset_id, asset.id.clone());
    assert_eq!(allowance.grantee, recipient.clone());
    assert_eq!(allowance.value, 1000);

    let sender_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });
    assert_eq!(sender_balance.balance, asset.supply - 24);

    let recipient_balance = service_call!(service, get_balance, recipient_ctx, GetBalancePayload {
        asset_id: asset.id,
        user:     recipient,
    });
    assert_eq!(recipient_balance.balance, 24);
}

#[test]
fn test_change_admin() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller.clone());

    let changed = service.change_admin(ctx, NewAdmin {
        addr: caller.clone(),
    });
    assert!(changed.is_error());

    let ctx = mock_context(TestService::admin());
    service_call!(service, change_admin, ctx, NewAdmin {
        addr: caller.clone(),
    });

    assert_eq!(service.admin(), caller);
}

#[test]
fn test_mint() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller);
    let asset = create_asset!(service, ctx.clone(), 10000, 10);

    let recipient = Address::from_hex("0x666cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    let asset_to_mint = MintAsset {
        asset_id: asset.id.clone(),
        to:       recipient.clone(),
        amount:   100,
        proof:    Hex::default(),
        memo:     "".to_owned(),
    };
    let minted = service.mint(ctx, asset_to_mint.clone());
    assert!(minted.is_error(), "mint require admin permission");

    let ctx = mock_context(TestService::admin());
    service_call!(service, mint, ctx.clone(), asset_to_mint);
    assert_eq!(ctx.get_events().len(), 1);

    let event: MintEvent = serde_json::from_str(&ctx.get_events()[0].data).expect("event");
    assert_eq!(event.asset_id, asset.id);
    assert_eq!(event.to, recipient);
    assert_eq!(event.amount, 100);

    let recipient_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id,
        user:     recipient,
    });
    assert_eq!(recipient_balance.balance, 100);
}

#[test]
fn test_burn() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller.clone());
    let asset = create_asset!(service, ctx.clone(), 10000, 10);

    let asset_to_burn = BurnAsset {
        asset_id: asset.id.clone(),
        amount:   100,
        proof:    Hex::default(),
        memo:     "".to_owned(),
    };
    service_call!(service, burn, ctx.clone(), asset_to_burn.clone());

    assert_eq!(ctx.get_events().len(), 2);
    let event: BurnEvent = serde_json::from_str(&ctx.get_events()[1].data).expect("event");
    assert_eq!(event.asset_id, asset.id);
    assert_eq!(event.from, caller);
    assert_eq!(event.amount, 100);

    let caller_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id.clone(),
        user:     caller,
    });
    assert_eq!(caller_balance.balance, asset.supply - 100);
}

#[test]
fn test_transfer_to_self() {
    let mut service = TestService::new();
    let caller = TestService::caller();
    let ctx = mock_context(caller.clone());
    let asset = create_asset!(service, ctx.clone(), 10000, 10);

    service_call!(service, transfer, ctx.clone(), TransferPayload {
        asset_id: asset.id.clone(),
        to:       caller.clone(),
        value:    100,
    });

    let caller_balance = service_call!(service, get_balance, ctx, GetBalancePayload {
        asset_id: asset.id,
        user:     caller,
    });
    assert_eq!(caller_balance.balance, asset.supply);
}

struct TestService(AssetService<SDK>);

impl Deref for TestService {
    type Target = AssetService<SDK>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TestService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TestService {
    fn new() -> TestService {
        let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()));
        let chain_db = DefaultChainQuerier::new(Arc::new(storage));

        let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
        let state = GeneralServiceState::new(trie);

        let sdk = DefalutServiceSDK::new(
            Rc::new(RefCell::new(state)),
            Rc::new(chain_db),
            NoopDispatcher {},
        );

        let mut service = AssetService::new(sdk);
        service.init_genesis(Self::genesis());

        TestService(service)
    }

    fn admin() -> Address {
        Address::from_hex(ADMIN).expect("admin")
    }

    fn caller() -> Address {
        Address::from_hex(CALLER).expect("caller")
    }

    fn genesis() -> InitGenesisPayload {
        let admin = Self::admin();

        InitGenesisPayload {
            id: Hash::digest(Bytes::from_static(b"test")),
            name: "test".to_owned(),
            symbol: "test".to_owned(),
            supply: 1000,
            precision: 10,
            issuer: admin.clone(),
            fee_account: admin.clone(),
            fee: 1,
            admin,
        }
    }
}

fn mock_context(caller: Address) -> ServiceContext {
    let params = ServiceContextParams {
        tx_hash: None,
        nonce: None,
        cycles_limit: CYCLE_LIMIT,
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
