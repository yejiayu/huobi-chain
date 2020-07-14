use std::sync::Arc;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use cita_trie::MemoryDB;
use protocol::traits::{
    Context, Executor, ExecutorParams, ExecutorResp, Service, ServiceMapping, ServiceResponse,
    ServiceSDK, Storage,
};
use protocol::types::{
    Address, Block, Genesis, Hash, Proof, RawTransaction, Receipt, SignedTransaction,
    TransactionRequest,
};
use protocol::ProtocolResult;

use admission_control::AdmissionControlService;
use asset::AssetService;
use authorization::AuthorizationService;
use framework::executor::ServiceExecutor;
use governance::GovernanceService;
use kyc::KycService;
use metadata::MetadataService;
use multi_signature::MultiSignatureService;

lazy_static::lazy_static! {
    pub static ref ADMIN_ACCOUNT: Address = Address::from_hex("0xcff1002107105460941f797828f468667aa1a2db").unwrap();
    pub static ref FEE_ACCOUNT: Address = Address::from_hex("0xcff1002107105460941f797828f468667aa1a2db").unwrap();
    pub static ref FEE_INLET_ACCOUNT: Address = Address::from_hex("0x503492f4bddc731a72b8caa806183f921c284f8e").unwrap();
    pub static ref PROPOSER_ACCOUNT: Address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    pub static ref NATIVE_ASSET_ID: Hash = Hash::from_hex("0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c").unwrap();
}

macro_rules! exec_txs {
    ($exec_cycle_limit: expr, $tx_cycle_limit: expr $(, ($service: expr, $method: expr, $payload: expr))*) => {
        {
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
                cycles_limit: $exec_cycle_limit,
                proposer:     PROPOSER_ACCOUNT.clone(),
            };

            let mut stxs = Vec::new();
            $(stxs.push(construct_stx(
                    $tx_cycle_limit,
                    $service.to_owned(),
                    $method.to_owned(),
                    serde_json::to_string(&$payload).unwrap()
                ));
            )*

            let resp = executor.exec(Context::new(), &params, &stxs).unwrap();

            let params = ExecutorParams {
                state_root: resp.state_root.clone(),
                height: 1,
                timestamp: 0,
                cycles_limit: u64::max_value(),
                proposer: PROPOSER_ACCOUNT.clone(),
            };

            let balances = tx_requests().iter().map(|req| {
                let res: ServiceResponse<String> =
                    executor.read(&params, &FEE_ACCOUNT.clone(), 1, req).expect("query balance");

                assert_eq!(res.is_error(), false);
                serde_json::from_str::<GetBalanceResponse>(&res.succeed_data)
                    .expect("decode get balance response json").balance
            }).collect::<Vec<_>>();

            Response {
                exec_resp: resp,
                fee_balance: balances[0],
                fee_inlet_balance: balances[1],
                proposer_balance: balances[2],
            }
        }
    };
}

mod service_test;
mod types;

#[derive(Clone, Debug)]
pub struct Response {
    exec_resp:         ExecutorResp,
    fee_balance:       u64,
    fee_inlet_balance: u64,
    proposer_balance:  u64,
}

pub fn construct_stx(
    tx_cycle_limit: u64,
    service_name: String,
    method: String,
    payload: String,
) -> SignedTransaction {
    let raw_tx = RawTransaction {
        chain_id:     Hash::from_empty(),
        nonce:        Hash::from_empty(),
        timeout:      0,
        cycles_price: 1,
        cycles_limit: tx_cycle_limit,
        request:      TransactionRequest {
            service_name,
            method,
            payload,
        },
        sender:       FEE_ACCOUNT.clone(),
    };

    SignedTransaction {
        raw:       raw_tx,
        tx_hash:   Hash::from_empty(),
        pubkey:    Bytes::from(
            hex::decode("031288a6788678c25952eba8693b2f278f66e2187004b64ac09416d07f83f96d5b")
                .unwrap(),
        ),
        signature: BytesMut::from("").freeze(),
    }
}

pub fn tx_requests() -> Vec<TransactionRequest> {
    vec![
        gen_tx_request(FEE_ACCOUNT.clone()),
        gen_tx_request(FEE_INLET_ACCOUNT.clone()),
        gen_tx_request(PROPOSER_ACCOUNT.clone()),
    ]
}

fn gen_tx_request(addr: Address) -> TransactionRequest {
    TransactionRequest {
        service_name: "asset".to_owned(),
        method:       "get_balance".to_owned(),
        payload:      gen_get_balance_payload(addr),
    }
}

fn gen_get_balance_payload(addr: Address) -> String {
    serde_json::to_string(&types::GetBalancePayload {
        asset_id: NATIVE_ASSET_ID.clone(),
        user:     addr,
    })
    .expect("encode get balance payload json")
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
            "authorization" => Box::new(AuthorizationService::new(sdk)) as Box<dyn Service>,
            "asset" => Box::new(AssetService::new(sdk)) as Box<dyn Service>,
            "metadata" => Box::new(MetadataService::new(sdk)) as Box<dyn Service>,
            "kyc" => Box::new(KycService::new(sdk)) as Box<dyn Service>,
            "multi_signature" => Box::new(MultiSignatureService::new(sdk)) as Box<dyn Service>,
            "governance" => Box::new(GovernanceService::new(sdk)) as Box<dyn Service>,
            "admission_control" => Box::new(AdmissionControlService::new(sdk)) as Box<dyn Service>,
            _ => panic!("not found service"),
        };

        Ok(service)
    }

    fn list_service_name(&self) -> Vec<String> {
        vec![
            "authorization".to_owned(),
            "asset".to_owned(),
            "metadata".to_owned(),
            "kyc".to_owned(),
            "multi_signature".to_owned(),
            "governance".to_owned(),
            "admission_control".to_owned(),
        ]
    }
}
