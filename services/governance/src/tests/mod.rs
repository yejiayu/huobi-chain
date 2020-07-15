use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use cita_trie::MemoryDB;

use asset::AssetService;
use framework::binding::sdk::{DefaultChainQuerier, DefaultServiceSDK};
use framework::binding::state::{GeneralServiceState, MPTTrie};
use framework::executor::ServiceExecutor;
use metadata::MetadataService;
use protocol::traits::{
    Context, Dispatcher, Executor, ExecutorParams, Service, ServiceMapping, ServiceResponse,
    ServiceSDK, Storage,
};
use protocol::types::{
    Address, Block, Genesis, Hash, Proof, RawTransaction, Receipt, ServiceContext,
    ServiceContextParams, SignedTransaction, TransactionRequest,
};
use protocol::ProtocolResult;

use crate::types::{
    AccumulateProfitPayload, Asset, DiscountLevel, GovernanceInfo, SetAdminPayload,
};
use crate::{GovernanceService, INFO_KEY, TX_FEE_INLET_KEY};

lazy_static::lazy_static! {
    static ref ADDRESS_1: Address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    static ref ADDRESS_2: Address = Address::from_hex("0xf8389d774afdad8755ef8e629e5a154fddc6325a").unwrap();
    static ref ADMIN: Address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
}

macro_rules! service {
    ($service:expr, $method:ident, $ctx:expr) => {{
        let resp = $service.$method($ctx);
        assert!(!resp.is_error());

        resp.succeed_data
    }};
    ($service:expr, $method:ident, $ctx:expr, $payload:expr) => {{
        let resp = $service.$method($ctx, $payload);
        assert!(!resp.is_error());

        resp.succeed_data
    }};
}

#[test]
fn test_update_metadata() {
    let memdb = Arc::new(MemoryDB::new(false));
    let arcs = Arc::new(MockStorage {});

    let toml_str = include_str!("./test_genesis.toml");
    let genesis: Genesis = toml::from_str(toml_str).unwrap();

    let root = ServiceExecutor::create_genesis(
        genesis.services,
        Arc::clone(&memdb),
        Arc::new(MockStorage {}),
        Arc::new(MockServiceMapping {}),
    )
    .unwrap();

    let mut executor = ServiceExecutor::with_root(
        root.clone(),
        Arc::clone(&memdb),
        Arc::clone(&arcs),
        Arc::new(MockServiceMapping {}),
    )
    .unwrap();

    let params = ExecutorParams {
        state_root:   root,
        height:       1,
        timestamp:    0,
        cycles_limit: std::u64::MAX,
        proposer:     Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap(),
    };

    let raw = RawTransaction {
        chain_id:     Hash::from_empty(),
        nonce:        Hash::from_empty(),
        timeout:      0,
        cycles_price: 1,
        cycles_limit: 80_000,
        request:      TransactionRequest {
            service_name: "governance".to_owned(),
            method:       "update_metadata".to_owned(),
            payload:      r#"{ "verifier_list": [{"bls_pub_key": "0xFFFFFFF9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22", "address": "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1", "propose_weight": 6, "vote_weight": 6}], "interval": 6, "propose_ratio": 6, "prevote_ratio": 6, "precommit_ratio": 6, "brake_ratio": 6, "timeout_gap": 20, "cycles_limit": 3000000, "cycles_price": 3000, "tx_num_limit": 20000, "max_tx_size": 500000 }"#
                .to_owned(),
        },
        sender: Address::from_hex("0xf8389d774afdad8755ef8e629e5a154fddc6325a").unwrap(),
    };
    let stx = SignedTransaction {
        raw,
        tx_hash: Hash::from_empty(),
        pubkey: Bytes::from(
            hex::decode("031288a6788678c25952eba8693b2f278f66e2187004b64ac09416d07f83f96d5b")
                .unwrap(),
        ),
        signature: BytesMut::from("").freeze(),
    };

    let executor_resp = executor.exec(Context::new(), &params, &[stx]).unwrap();
    let receipt = &executor_resp.receipts[0];
    let event = &receipt.events[1];

    let expect_event = r#"{"verifier_list":[{"bls_pub_key":"0xFFFFFFF9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22","address":"0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1","propose_weight":6,"vote_weight":6}],"interval":6,"propose_ratio":6,"prevote_ratio":6,"precommit_ratio":6,"brake_ratio":6,"timeout_gap":20,"cycles_limit":3000000,"cycles_price":3000,"tx_num_limit":20000,"max_tx_size":500000}"#.to_owned();

    assert_eq!(expect_event, event.data);
}

#[test]
fn test_set_admin() {
    let admin_1 = ADDRESS_1.clone();
    let admin_2 = ADDRESS_2.clone();

    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let context = mock_context(cycles_limit, admin_1.clone());

    let mut service = new_governance_service(admin_1.clone());
    let old_admin = service!(service, get_admin_address, context.clone());
    assert_eq!(old_admin, admin_1);

    service!(service, set_admin, context.clone(), SetAdminPayload {
        admin: admin_2.clone(),
    });
    let new_admin = service!(service, get_admin_address, context);
    assert_eq!(new_admin, admin_2);
}

#[test]
fn test_get_fee() {
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let context = mock_context(cycles_limit, ADMIN.clone());

    let service = new_governance_service(ADMIN.clone());
    let floor_fee = service!(service, get_tx_floor_fee, context.clone());
    let failure_fee = service!(service, get_tx_failure_fee, context);

    assert_eq!(floor_fee, 10);
    assert_eq!(failure_fee, 20);
}

#[test]
fn test_accumulate_profit() {
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let context = mock_context(cycles_limit, ADMIN.clone());

    let mut service = new_governance_service(ADMIN.clone());
    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_1.clone(),
            accumulated_profit: 1,
        }
    );
    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_2.clone(),
            accumulated_profit: 1_000_000,
        }
    );
    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_2.clone(),
            accumulated_profit: 5_000_000,
        }
    );

    assert_eq!(service.calc_profit_records(&context).unwrap(), 6_000_001);
}

#[test]
fn test_calc_fee_above_floor_fee() {
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let context = mock_context(cycles_limit, ADMIN.clone());
    let mut service = new_governance_service(ADMIN.clone());

    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_1.clone(),
            accumulated_profit: 5_000_000,
        }
    );
    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_2.clone(),
            accumulated_profit: 10_000_000,
        }
    );

    // total profit =5m+10m=15m
    // fee = 15m *3%% = 45
    // mocked balance 100_000
    // discount 50 percent
    // 45 * 50% = 22
    // floor fee 10
    // max(27,10) = 22
    assert_eq!(service.calc_tx_fee(&context).unwrap(), 2);
}

#[test]
fn test_calc_fee_below_floor_fee() {
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let context = mock_context(cycles_limit, ADMIN.clone());
    let mut service = new_governance_service(ADMIN.clone());

    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_1.clone(),
            accumulated_profit: 1_000_000,
        }
    );
    service!(
        service,
        accumulate_profit,
        context.clone(),
        AccumulateProfitPayload {
            address:            ADDRESS_2.clone(),
            accumulated_profit: 2_000_000,
        }
    );

    // total profit =1m+2m=3m
    // fee = 3m *3%% = 9
    // mocked balance 100_000
    // discount 50 percent
    // 9 * 50% = 4
    // floor fee 10
    // max(4,10) = 10
    assert_eq!(service.calc_tx_fee(&context).unwrap(), -10);
}

#[test]
fn test_reset_profits_in_tx_hook_after() {
    let admin = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let ctx = mock_context(cycles_limit, admin.clone());

    let mut service = new_governance_service(admin.clone());
    assert_eq!(service.profits_len(), 0, "should not have any profits");

    service!(
        service,
        accumulate_profit,
        ctx.clone(),
        AccumulateProfitPayload {
            address:            admin,
            accumulated_profit: 1,
        }
    );

    // Manually call tx_hook_after
    service.deduct_fee(ctx);

    assert_eq!(service.profits_len(), 1, "should reset profits");
}

fn new_governance_service(
    admin: Address,
) -> GovernanceService<
    DefaultServiceSDK<
        GeneralServiceState<MemoryDB>,
        DefaultChainQuerier<MockStorage>,
        MockDispatcher,
    >,
> {
    let chain_db = DefaultChainQuerier::new(Arc::new(MockStorage {}));
    let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
    let state = GeneralServiceState::new(trie);

    let mut sdk = DefaultServiceSDK::new(
        Rc::new(RefCell::new(state)),
        Rc::new(chain_db),
        MockDispatcher {},
    );

    let fee_addr = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    sdk.set_value(TX_FEE_INLET_KEY.to_string(), fee_addr);
    sdk.set_value(INFO_KEY.to_string(), mock_governance_info(admin));

    GovernanceService::new(sdk)
}

fn mock_governance_info(admin: Address) -> GovernanceInfo {
    let levels = vec![
        DiscountLevel {
            threshold:        1000,
            discount_percent: 90,
        },
        DiscountLevel {
            threshold:        10000,
            discount_percent: 70,
        },
        DiscountLevel {
            threshold:        100_000,
            discount_percent: 50,
        },
    ];

    GovernanceInfo {
        admin,
        tx_failure_fee: 20,
        tx_floor_fee: 10,
        profit_deduct_rate_per_million: 3,
        miner_benefit: 20,
        tx_fee_discount: levels,
    }
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

pub struct MockServiceMapping;

impl ServiceMapping for MockServiceMapping {
    fn get_service<SDK: 'static + ServiceSDK>(
        &self,
        name: &str,
        sdk: SDK,
    ) -> ProtocolResult<Box<dyn Service>> {
        let service = match name {
            "asset" => Box::new(AssetService::new(sdk)) as Box<dyn Service>,
            "metadata" => Box::new(MetadataService::new(sdk)) as Box<dyn Service>,
            "governance" => Box::new(GovernanceService::new(sdk)) as Box<dyn Service>,
            _ => panic!("not found service"),
        };

        Ok(service)
    }

    fn list_service_name(&self) -> Vec<String> {
        vec![
            "asset".to_owned(),
            "metadata".to_owned(),
            "governance".to_owned(),
        ]
    }
}

struct MockDispatcher;

impl Dispatcher for MockDispatcher {
    fn read(&self, ctx: ServiceContext) -> ServiceResponse<String> {
        let service = ctx.get_service_name();
        let method = ctx.get_service_method();

        if service != "asset" || method != "get_native_asset" {
            return ServiceResponse::<String>::from_error(
                2,
                format!("not found method:{:?} of service:{:?}", method, service),
            );
        }

        let asset = Asset {
            id:        Hash::digest(Bytes::from_static(b"7")),
            name:      "da_wan".to_owned(),
            symbol:    "guan_mian".to_owned(),
            supply:    2_020_626,
            precision: 311,
            issuers:   vec![
                Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap(),
            ],
        };

        let json_asset = serde_json::to_string(&asset).expect("serde asset");
        ServiceResponse::from_succeed(json_asset)
    }

    fn write(&self, ctx: ServiceContext) -> ServiceResponse<String> {
        let service = ctx.get_service_name();
        let method = ctx.get_service_method();

        if service != "asset" || method != "transfer_from" {
            return ServiceResponse::<String>::from_error(
                2,
                format!("not found method:{:?} of service:{:?}", method, service),
            );
        }

        ServiceResponse::from_succeed("".to_owned())
    }
}
