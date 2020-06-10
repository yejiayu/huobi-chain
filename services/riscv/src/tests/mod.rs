#[macro_use]
mod macros;
mod duktape;

use crate::{
    types::{DeployPayload, ExecPayload, GetContractPayload, InitGenesisPayload, InterpreterType},
    RiscvService,
};

use async_trait::async_trait;
use cita_trie::MemoryDB;

use framework::binding::{
    sdk::{DefalutServiceSDK as DefaultServiceSDK, DefaultChainQuerier},
    state::{GeneralServiceState, MPTTrie},
};
use protocol::{
    traits::{Context, Dispatcher, ServiceResponse, ServiceState, Storage},
    types::{
        Address, Block, Hash, Proof, Receipt, ServiceContext, ServiceContextParams,
        SignedTransaction,
    },
    Bytes, ProtocolResult,
};

use std::{
    cell::RefCell,
    fs::File,
    io::Read,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

thread_local! {
    static STATE: Rc<RefCell<GeneralServiceState<MemoryDB>>> = {
        let trie = MPTTrie::new(Arc::new(MemoryDB::default()));
        Rc::new(RefCell::new(GeneralServiceState::new(trie)))
    };
}

const CYCLE_LIMIT: u64 = 1024 * 1024 * 1024;
const CALLER: &str = "0x0000000000000000000000000000000000000001";
const ADMIN: &str = "0x755cdba6ae4f479f7164792b318b2a06c759833b";

macro_rules! read_code {
    ($path:expr) => {{
        let mut file = File::open($path).expect("open code file");
        let mut buffer = Vec::new();

        file.read_to_end(&mut buffer).expect("read code file");
        hex::encode(Bytes::from(buffer).as_ref())
    }};
}

#[test]
fn test_deploy_and_run() {
    let mut service = TestRiscvService::new();
    let context = TestContext::default().make_admin();

    let code = read_code!("src/tests/simple_storage");
    let deploy_result = service!(service, deploy, context.clone(), DeployPayload {
        code:      code.clone(),
        intp_type: InterpreterType::Binary,
        init_args: "set k init".into(),
    });
    assert_eq!(&deploy_result.init_ret, "");

    // test get_contract
    let address = deploy_result.address;
    let get_contract_resp = service!(service, get_contract, context.clone(), GetContractPayload {
        address:      address.clone(),
        get_code:     true,
        storage_keys: vec![hex::encode("k"), "".to_owned(), "3a".to_owned()],
    });
    assert_eq!(&get_contract_resp.code, &code);
    assert_eq!(&get_contract_resp.storage_values, &vec![
        hex::encode("init"),
        "".to_owned(),
        "".to_owned()
    ]);

    let exec_result = service!(service, exec, context.clone(), ExecPayload {
        address: address.clone(),
        args:    "get k".into(),
    });
    assert_eq!(&exec_result, "init");

    let exec_result = service!(service, exec, context.clone(), ExecPayload {
        address: address.clone(),
        args:    "set k v".into(),
    });
    assert_eq!(&exec_result, "");

    let exec_result = service!(service, exec, context.clone(), ExecPayload {
        address: address.clone(),
        args:    "get k".into(),
    });
    assert_eq!(&exec_result, "v");

    // wrong command
    let exec_result = service.exec(context.clone(), ExecPayload {
        address: address.clone(),
        args:    "clear k v".into(),
    });
    assert!(exec_result.is_error());

    // wrong command 2
    let exec_result = service.exec(context, ExecPayload {
        address,
        args: "set k".into(),
    });
    assert!(exec_result.is_error());
}

#[test]
fn should_not_change_state_by_read_call() {
    let mut service = TestRiscvService::new();
    let mut ctx = TestContext::default();

    let deployed = service!(service, deploy, ctx.make_admin(), DeployPayload {
        code:      read_code!("src/tests/simple_storage"),
        intp_type: InterpreterType::Binary,
        init_args: "set k init".into(),
    });
    assert_eq!(deployed.init_ret, "");

    let inited = service!(service, exec, ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    "get k".into(),
    });

    let called = service.call(ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    "set k v".into(),
    });
    assert_eq!(called.is_error(), true);

    let called = service!(service, call, ctx.make_admin(), ExecPayload {
        address: deployed.address,
        args:    "get k".into(),
    });
    assert_eq!(called, inited, "read call should not change state");
}

#[test]
fn should_not_change_state_through_read_write_chained_invocations() {
    // env_logger::init();

    let mut service = TestRiscvService::new();
    let mut ctx = TestContext::default();

    let deployed = service!(service, deploy, ctx.make_admin(), DeployPayload {
        code:      read_code!("src/tests/write_read"),
        intp_type: InterpreterType::Binary,
        init_args: "".into(),
    });
    assert_eq!(deployed.init_ret, "");

    let called = service.call(ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    format!("f{}", deployed.address.as_hex()),
    });
    assert!(called.is_error());

    let msg_resp = service.call(ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    format!("x{}", deployed.address.as_hex()),
    });
    assert!(!msg_resp.is_error());
    assert_eq!(
        msg_resp.succeed_data,
        serde_json::to_string("").expect("json encode")
    );
}

#[test]
fn should_allow_change_state_through_write_write_chained_invocations() {
    let mut service = TestRiscvService::new();
    let mut ctx = TestContext::default();

    let deployed = service!(service, deploy, ctx.make_admin(), DeployPayload {
        code:      read_code!("src/tests/write_read"),
        intp_type: InterpreterType::Binary,
        init_args: "".into(),
    });
    assert_eq!(deployed.init_ret, "");

    let called = service.exec(ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    format!("c{}", deployed.address.as_hex()),
    });
    assert!(!called.is_error());

    let msg_resp = service.call(ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    "m".into(),
    });
    assert!(!msg_resp.is_error());
    let expect_msg = serde_json::to_string(&msg_resp.succeed_data).expect("encode expect msg");

    let msg_resp = service.call(ctx.make_admin(), ExecPayload {
        address: deployed.address.clone(),
        args:    format!("x{}", deployed.address.as_hex()),
    });
    assert!(!msg_resp.is_error());

    assert_eq!(msg_resp.succeed_data, expect_msg);
}

struct TestRiscvService(
    RiscvService<
        DefaultServiceSDK<
            GeneralServiceState<MemoryDB>,
            DefaultChainQuerier<MockStorage>,
            MockDispatcher,
        >,
    >,
);

impl TestRiscvService {
    pub fn new() -> TestRiscvService {
        let chain_db = DefaultChainQuerier::new(Arc::new(MockStorage {}));
        let state = STATE.with(|state| Rc::clone(state));

        let sdk = DefaultServiceSDK::new(state, Rc::new(chain_db), MockDispatcher {});

        let mut service = Self(RiscvService::init(sdk));

        let admin = Address::from_hex(ADMIN).expect("admin");
        service.init_genesis(InitGenesisPayload {
            enable_whitelist: true,
            whitelist:        vec![admin.clone()],
            admins:           vec![admin],
        });

        service
    }
}

impl Deref for TestRiscvService {
    type Target = RiscvService<
        DefaultServiceSDK<
            GeneralServiceState<MemoryDB>,
            DefaultChainQuerier<MockStorage>,
            MockDispatcher,
        >,
    >;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TestRiscvService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct TestContext {
    count:  usize,
    height: u64,
}

impl Default for TestContext {
    fn default() -> Self {
        TestContext {
            count:  1,
            height: 1,
        }
    }
}

impl TestContext {
    fn make(&mut self) -> ServiceContext {
        ServiceContext::new(self.new_params())
    }

    fn make_admin(&mut self) -> ServiceContext {
        let mut params = self.new_params();
        params.caller = Address::from_hex(ADMIN).expect("ctx admin");
        ServiceContext::new(params)
    }

    fn new_params(&mut self) -> ServiceContextParams {
        self.count += 1;
        self.height += 1;

        let tx_hash = Hash::digest(Bytes::from(format!("{}", self.count)));

        ServiceContextParams {
            tx_hash:         Some(tx_hash),
            nonce:           None,
            cycles_limit:    CYCLE_LIMIT,
            cycles_price:    1,
            cycles_used:     Rc::new(RefCell::new(3)),
            caller:          Address::from_hex(CALLER).expect("ctx caller"),
            height:          self.height,
            timestamp:       0,
            extra:           None,
            service_name:    "service_name".to_owned(),
            service_method:  "service_method".to_owned(),
            service_payload: "service_payload".to_owned(),
            events:          Rc::new(RefCell::new(vec![])),
        }
    }
}

struct MockDispatcher;

impl MockDispatcher {
    fn json_resp(resp: ServiceResponse<String>) -> ServiceResponse<String> {
        if resp.is_error() {
            return resp;
        }

        let mut data_json = serde_json::to_string(&resp.succeed_data).expect("json encode");
        if data_json == "null" {
            data_json = "".to_owned();
        }
        ServiceResponse::<String>::from_succeed(data_json)
    }
}

impl Dispatcher for MockDispatcher {
    fn read(&self, ctx: ServiceContext) -> ServiceResponse<String> {
        let service = ctx.get_service_name();
        let method = ctx.get_service_method();

        if service != "riscv" || method != "call" {
            return ServiceResponse::<String>::from_error(
                2,
                format!("not found method:{:?} of service:{:?}", method, service),
            );
        }

        let service = TestRiscvService::new();
        let payload = serde_json::from_str(ctx.get_payload()).expect("dispatcher payload");
        Self::json_resp(service.call(ctx, payload))
    }

    fn write(&self, ctx: ServiceContext) -> ServiceResponse<String> {
        let service = ctx.get_service_name();
        let method = ctx.get_service_method();

        if service != "riscv" || method != "exec" {
            return ServiceResponse::<String>::from_error(
                2,
                format!("not found method:{:?} of service:{:?}", method, service),
            );
        }

        let mut service = TestRiscvService::new();
        let payload = serde_json::from_str(ctx.get_payload()).expect("dispatcher payload");
        let resp = Self::json_resp(service.exec(ctx, payload));

        STATE.with(|state| {
            state.borrow_mut().commit().expect("commit");
        });

        resp
    }
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
