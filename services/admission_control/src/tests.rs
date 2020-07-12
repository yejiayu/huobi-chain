use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

use cita_trie::MemoryDB;
use core_storage::{adapter::memory::MemoryAdapter, ImplStorage};
use framework::binding::{
    sdk::{DefaultChainQuerier, DefaultServiceSDK},
    state::{GeneralServiceState, MPTTrie},
};
use protocol::{
    traits::{Dispatcher, ServiceResponse},
    types::{
        Address, Bytes, Hash, RawTransaction, ServiceContext, ServiceContextParams,
        SignedTransaction, TransactionRequest,
    },
};
use serde::Deserialize;

use crate::{
    types::{AddressList, Genesis, NewAdmin},
    AdmissionControlService, ServiceError,
};

type SDK = DefaultServiceSDK<
    GeneralServiceState<MemoryDB>,
    DefaultChainQuerier<ImplStorage<MemoryAdapter>>,
    MockDispatcher,
>;

const ADMIN: &str = "0x755cdba6ae4f479f7164792b318b2a06c759833b";
const CHRIS: &str = "0x0000000000000000000000000000000000000001";
const WESKER: &str = "0x0000000000000000000000000000000000000002";
const G: &str = "0x0000000000000000000000000000000000000003";

#[test]
fn should_properly_init_genesis() {
    let mut service = new_raw_service();
    let admin = Address::from_hex(ADMIN).expect("admin");
    let caller = Address::from_hex(WESKER).expect("caller");

    service.init_genesis(Genesis {
        admin:     admin.clone(),
        deny_list: vec![caller.clone()],
    });

    let resp = service.is_permitted(mock_context(caller.clone()), mock_transaction(caller));

    assert!(resp.is_error());
    assert_eq!(service.admin(), admin);
}

#[test]
#[should_panic(expected = "Bad payload invalid admin address")]
fn should_panic_on_invalid_admin_address_in_genesis() {
    let mut service = new_raw_service();

    service.init_genesis(Genesis {
        admin:     Address::default(),
        deny_list: vec![],
    });
}

#[test]
fn should_only_change_admin_by_admin() {
    let mut service = new_service();
    let admin = Address::from_hex(ADMIN).expect("admin");
    let new_admin = Address::from_hex(CHRIS).expect("new admin");

    let ctx = mock_context(admin.clone());
    service.change_admin(ctx.clone(), NewAdmin {
        new_admin: new_admin.clone(),
    });
    assert_eq!(service.admin(), new_admin);
    assert_eq!(ctx.get_events().len(), 1);
    assert_eq!(ctx.get_events()[0].name, "ChangeAdmin");
    let event_data: NewAdmin = serde_json::from_str(ctx.get_events()[0].data.as_str()).unwrap();
    assert_eq!(event_data, NewAdmin {
        new_admin: new_admin.clone(),
    });

    let resp = service.change_admin(mock_context(admin.clone()), NewAdmin { new_admin: admin });
    assert!(resp.is_error());
    assert_eq!(service.admin(), new_admin);
}

#[test]
fn should_only_forbid_address_by_admin() {
    let mut service = new_service();
    let admin = Address::from_hex(ADMIN).expect("admin");

    let deny_list = vec![
        Address::from_hex(WESKER).expect("wesker"),
        Address::from_hex(G).expect("g"),
    ];

    let resp = service.forbid(mock_context(deny_list[0].clone()), AddressList {
        addrs: deny_list.clone(),
    });
    assert!(resp.is_error());
    assert!(resp
        .error_message
        .contains(&ServiceError::NonAuthorized.to_string()));

    let ctx = mock_context(admin.clone());
    let resp = service.forbid(ctx.clone(), AddressList {
        addrs: deny_list.clone(),
    });
    assert!(!resp.is_error());
    assert_eq!(ctx.get_events().len(), 1);

    assert_eq!(ctx.get_events()[0].name, "Forbid");
    let event_data: AddressList = serde_json::from_str(ctx.get_events()[0].data.as_str()).unwrap();
    assert_eq!(event_data, AddressList {
        addrs: deny_list.clone(),
    });

    for addr in deny_list.iter() {
        let resp = service.is_permitted(
            mock_context(admin.clone()),
            mock_transaction(addr.to_owned()),
        );
        assert_eq!(resp.is_error(), true);
    }

    let result = service
        .status(mock_context(admin), AddressList { addrs: deny_list })
        .succeed_data
        .status
        .into_iter()
        .any(|b| b);

    assert!(!result)
}

#[test]
fn should_only_permit_address_by_admin() {
    let mut service = new_raw_service();
    let admin = Address::from_hex(ADMIN).expect("admin");
    let deny_list = vec![
        Address::from_hex(WESKER).expect("wesker"),
        Address::from_hex(G).expect("g"),
    ];

    service.init_genesis(Genesis {
        admin:     admin.clone(),
        deny_list: deny_list.clone(),
    });

    let resp = service.permit(mock_context(deny_list[0].clone()), AddressList {
        addrs: deny_list.clone(),
    });
    assert!(resp.is_error());
    assert!(resp
        .error_message
        .contains(&ServiceError::NonAuthorized.to_string()));

    let ctx = mock_context(admin.clone());
    let resp = service.permit(ctx.clone(), AddressList {
        addrs: deny_list.clone(),
    });
    assert!(!resp.is_error());
    assert_eq!(ctx.get_events().len(), 1);

    assert_eq!(ctx.get_events()[0].name, "Permit");
    let event_data: AddressList = serde_json::from_str(ctx.get_events()[0].data.as_str()).unwrap();
    assert_eq!(event_data, AddressList {
        addrs: deny_list.clone(),
    });

    for addr in deny_list {
        let resp = service.is_permitted(mock_context(admin.clone()), mock_transaction(addr));
        assert!(!resp.is_error());
    }
}

#[test]
fn should_allow_tx_with_enough_balance_to_cover_fee() {
    let mut service = new_raw_service();
    let admin = Address::from_hex(ADMIN).expect("admin");
    let chris = Address::from_hex(CHRIS).expect("chris");

    service.init_genesis(Genesis {
        admin,
        deny_list: vec![],
    });

    let ctx = mock_context(chris.clone());
    let stx = mock_transaction(chris);

    let validated = service.is_valid(ctx, stx);
    assert!(!validated.is_error());
}

#[test]
fn should_reject_tx_without_enough_balance_to_cover_fee() {
    let mut service = new_raw_service();
    let admin = Address::from_hex(ADMIN).expect("admin");
    let wesker = Address::from_hex(WESKER).expect("wesker");

    service.init_genesis(Genesis {
        admin,
        deny_list: vec![],
    });

    let ctx = mock_context(wesker.clone());
    let stx = mock_transaction(wesker);

    let validated = service.is_valid(ctx, stx);
    assert!(validated.is_error());
    assert_eq!(
        validated.error_message,
        ServiceError::BalanceLowerThanFee.to_string()
    );
}

fn new_raw_service() -> AdmissionControlService<SDK> {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()));
    let chain_db = DefaultChainQuerier::new(Arc::new(storage));

    let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
    let state = GeneralServiceState::new(trie);

    let sdk = DefaultServiceSDK::new(
        Rc::new(RefCell::new(state)),
        Rc::new(chain_db),
        MockDispatcher::default(),
    );

    AdmissionControlService::new(sdk)
}

fn new_service() -> AdmissionControlService<SDK> {
    let mut service = new_raw_service();
    service.init_genesis(Genesis {
        admin:     Address::from_hex(ADMIN).expect("admin"),
        deny_list: vec![Address::from_hex(WESKER).expect("block list")],
    });

    service
}

fn mock_context(caller: Address) -> ServiceContext {
    let params = ServiceContextParams {
        tx_hash: None,
        nonce: None,
        cycles_limit: 99999,
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

fn get_random_bytes(len: usize) -> Bytes {
    let vec: Vec<u8> = (0..len).map(|_| rand::random::<u8>()).collect();
    Bytes::from(vec)
}

fn mock_hash() -> Hash {
    Hash::digest(get_random_bytes(10))
}

fn mock_transaction(addr: Address) -> SignedTransaction {
    let tx_request = TransactionRequest {
        service_name: "mock-service".to_owned(),
        method:       "mock-method".to_owned(),
        payload:      "mock-payload".to_owned(),
    };

    let raw_tx = RawTransaction {
        chain_id:     mock_hash(),
        nonce:        mock_hash(),
        timeout:      100,
        cycles_price: 1,
        cycles_limit: 100,
        request:      tx_request,
        sender:       addr,
    };

    SignedTransaction {
        raw:       raw_tx,
        tx_hash:   mock_hash(),
        pubkey:    Default::default(),
        signature: Default::default(),
    }
}

type FFF = Box<dyn Fn(ServiceContext) -> ServiceResponse<String>>;

struct MockDispatcher {
    table: HashMap<String, FFF>,
}

fn get_native_asset(_: ServiceContext) -> ServiceResponse<String> {
    let asset = serde_json::json!({
        "id":        Hash::digest(Bytes::from_static(b"da_wan_kuan_mian")),
        "name":      "wan_da".to_owned(),
        "symbol":    "mian_change".to_owned(),
        "supply":    2_020_626,
        "precision": 311,
        "issuer":    Address::from_hex(CHRIS).unwrap(),
    });

    ServiceResponse::from_succeed(asset.to_string())
}

fn get_balance(ctx: ServiceContext) -> ServiceResponse<String> {
    #[derive(Debug, Deserialize)]
    struct Payload {
        pub asset_id: Hash,
        pub user:     Address,
    }

    let Payload { asset_id, user } =
        serde_json::from_str(ctx.get_payload()).expect("decode get balance payload");

    let account = if user == Address::from_hex(CHRIS).expect("chris address") {
        serde_json::json!({
            "id":   asset_id,
            "user": user,
            "balance": 2_020_626,
        })
    } else {
        serde_json::json!({
            "id":   asset_id,
            "user": user,
            "balance": 0,
        })
    };

    ServiceResponse::from_succeed(account.to_string())
}

fn get_tx_failure_fee(_: ServiceContext) -> ServiceResponse<String> {
    ServiceResponse::from_succeed(serde_json::to_string(&2077).expect("serde json fee"))
}

impl Default for MockDispatcher {
    fn default() -> Self {
        let mut table = HashMap::new();
        table.insert(
            "asset#get_native_asset".to_owned(),
            Box::new(get_native_asset) as FFF,
        );
        table.insert("asset#get_balance".to_owned(), Box::new(get_balance) as FFF);
        table.insert(
            "governance#get_tx_failure_fee".to_owned(),
            Box::new(get_tx_failure_fee) as FFF,
        );

        MockDispatcher { table }
    }
}

impl Dispatcher for MockDispatcher {
    fn read(&self, ctx: ServiceContext) -> ServiceResponse<String> {
        let service = ctx.get_service_name();
        let method = ctx.get_service_method();

        if let Some(f) = self.table.get(&format!("{}#{}", service, method)) {
            f(ctx)
        } else {
            let err_msg = format!("not found method:{:?} of service:{:?}", method, service);
            ServiceResponse::<String>::from_error(2, err_msg)
        }
    }

    fn write(&self, ctx: ServiceContext) -> ServiceResponse<String> {
        let service = ctx.get_service_name();
        let method = ctx.get_service_method();

        if let Some(f) = self.table.get(&format!("{}#{}", service, method)) {
            f(ctx)
        } else {
            let err_msg = format!("not found method:{:?} of service:{:?}", method, service);
            ServiceResponse::<String>::from_error(2, err_msg)
        }
    }
}
