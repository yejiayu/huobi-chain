use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use cita_trie::MemoryDB;

use framework::binding::sdk::{DefalutServiceSDK, DefaultChainQuerier};
use framework::binding::state::{GeneralServiceState, MPTTrie};
use framework::executor::ServiceExecutor;
use metadata::MetadataService;
use protocol::traits::{
    Executor, ExecutorParams, NoopDispatcher, Service, ServiceMapping, ServiceSDK, Storage,
};
use protocol::types::{
    Address, Block, Genesis, Hash, Proof, RawTransaction, Receipt, ServiceContext,
    ServiceContextParams, SignedTransaction, TransactionRequest,
};
use protocol::ProtocolResult;

use crate::types::SetAdminPayload;
use crate::{NodeManagerService, ADMIN_KEY};

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
    };

    let raw = RawTransaction {
        chain_id:     Hash::from_empty(),
        nonce:        Hash::from_empty(),
        timeout:      0,
        cycles_price: 1,
        cycles_limit: 60_000,
        request:      TransactionRequest {
            service_name: "node_manager".to_owned(),
            method:       "update_metadata".to_owned(),
            payload:      r#"{ "verifier_list": [{"bls_pub_key": "FFFFFFF9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22", "address": "016cbd9ee47a255a6f68882918dcdd9e14e6bee1", "propose_weight": 6, "vote_weight": 6}], "interval": 6, "propose_ratio": 6, "prevote_ratio": 6, "precommit_ratio": 6, "brake_ratio": 6 }"#
                .to_owned(),
        },
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

    let txs = vec![stx];
    let executor_resp = executor.exec(&params, &txs).unwrap();
    let receipt = &executor_resp.receipts[0];
    let event = &receipt.events[0];

    let expect_event = r#"{"topic":"Metadata Updated","verifier_list":[{"bls_pub_key":"FFFFFFF9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22","address":"016cbd9ee47a255a6f68882918dcdd9e14e6bee1","propose_weight":6,"vote_weight":6}],"interval":6,"propose_ratio":6,"prevote_ratio":6,"precommit_ratio":6,"brake_ratio":6}"#.to_owned();

    assert_eq!(expect_event, event.data);
}

#[test]
fn test_set_admin() {
    let admin_1: Address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    let admin_2: Address = Address::from_hex("f8389d774afdad8755ef8e629e5a154fddc6325a").unwrap();

    let cycles_limit = 1024 * 1024 * 1024; // 1073741824
    let context = mock_context(cycles_limit, admin_1.clone());

    let mut service = new_node_manager_service(admin_1.clone());
    let old_admin = service.get_admin(context.clone()).unwrap();
    assert_eq!(old_admin, admin_1);

    service
        .set_admin(context.clone(), SetAdminPayload {
            admin: admin_2.clone(),
        })
        .unwrap();
    let new_admin = service.get_admin(context).unwrap();
    assert_eq!(new_admin, admin_2);
}

fn new_node_manager_service(
    admin: Address,
) -> NodeManagerService<
    DefalutServiceSDK<
        GeneralServiceState<MemoryDB>,
        DefaultChainQuerier<MockStorage>,
        NoopDispatcher,
    >,
> {
    let chain_db = DefaultChainQuerier::new(Arc::new(MockStorage {}));
    let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
    let state = GeneralServiceState::new(trie);

    let mut sdk = DefalutServiceSDK::new(
        Rc::new(RefCell::new(state)),
        Rc::new(chain_db),
        NoopDispatcher {},
    );

    sdk.set_value(ADMIN_KEY.to_string(), admin).unwrap();

    NodeManagerService::new(sdk).unwrap()
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
    async fn insert_transactions(&self, _: Vec<SignedTransaction>) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn insert_block(&self, _: Block) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn insert_receipts(&self, _: Vec<Receipt>) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn update_latest_proof(&self, _: Proof) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn get_transaction_by_hash(&self, _: Hash) -> ProtocolResult<SignedTransaction> {
        unimplemented!()
    }

    async fn get_transactions(&self, _: Vec<Hash>) -> ProtocolResult<Vec<SignedTransaction>> {
        unimplemented!()
    }

    async fn get_latest_block(&self) -> ProtocolResult<Block> {
        unimplemented!()
    }

    async fn get_block_by_height(&self, _: u64) -> ProtocolResult<Block> {
        unimplemented!()
    }

    async fn get_block_by_hash(&self, _: Hash) -> ProtocolResult<Block> {
        unimplemented!()
    }

    async fn get_receipt(&self, _: Hash) -> ProtocolResult<Receipt> {
        unimplemented!()
    }

    async fn get_receipts(&self, _: Vec<Hash>) -> ProtocolResult<Vec<Receipt>> {
        unimplemented!()
    }

    async fn get_latest_proof(&self) -> ProtocolResult<Proof> {
        unimplemented!()
    }

    async fn update_overlord_wal(&self, _info: Bytes) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn update_muta_wal(&self, _info: Bytes) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn load_overlord_wal(&self) -> ProtocolResult<Bytes> {
        unimplemented!()
    }

    async fn load_muta_wal(&self) -> ProtocolResult<Bytes> {
        unimplemented!()
    }

    async fn update_exec_queue_wal(&self, _info: Bytes) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn load_exec_queue_wal(&self) -> ProtocolResult<Bytes> {
        unimplemented!()
    }

    async fn insert_wal_transactions(
        &self,
        _block_hash: Hash,
        _signed_txs: Vec<SignedTransaction>,
    ) -> ProtocolResult<()> {
        unimplemented!()
    }

    async fn get_wal_transactions(
        &self,
        _block_hash: Hash,
    ) -> ProtocolResult<Vec<SignedTransaction>> {
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
            "metadata" => Box::new(MetadataService::new(sdk)?) as Box<dyn Service>,
            "node_manager" => Box::new(NodeManagerService::new(sdk)?) as Box<dyn Service>,
            _ => panic!("not found service"),
        };

        Ok(service)
    }

    fn list_service_name(&self) -> Vec<String> {
        vec!["metadata".to_owned(), "node_manager".to_owned()]
    }
}
