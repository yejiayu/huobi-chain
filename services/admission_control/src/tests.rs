use crate::{
    types::{AddressList, Event, Genesis, NewAdmin},
    AdmissionControlService, ServiceError,
};

use cita_trie::MemoryDB;
use core_storage::{adapter::memory::MemoryAdapter, ImplStorage};
use framework::binding::{
    sdk::{DefaultChainQuerier, DefaultServiceSDK},
    state::{GeneralServiceState, MPTTrie},
};
use protocol::{
    traits::NoopDispatcher,
    types::{Address, ServiceContext, ServiceContextParams},
};

use std::{cell::RefCell, rc::Rc, sync::Arc};

type SDK = DefaultServiceSDK<
    GeneralServiceState<MemoryDB>,
    DefaultChainQuerier<ImplStorage<MemoryAdapter>>,
    NoopDispatcher,
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

    let resp = service.is_blocked(mock_context(caller.clone()), caller);

    assert!(!resp.is_error());
    assert_eq!(resp.succeed_data, true);
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

    let event: Event<NewAdmin> = ctx.get_events()[0].data.parse().expect("parse event");
    assert_eq!(event.topic, "change_admin");
    assert_eq!(event.data, NewAdmin {
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

    let event: Event<AddressList> = ctx.get_events()[0].data.parse().expect("parse event");
    assert_eq!(event.topic, "forbid");
    assert_eq!(event.data, AddressList {
        addrs: deny_list.clone(),
    });

    for addr in deny_list {
        let resp = service.is_blocked(mock_context(admin.clone()), addr);
        assert_eq!(resp.succeed_data, true);
    }
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

    let event: Event<AddressList> = ctx.get_events()[0].data.parse().expect("parse event");
    assert_eq!(event.topic, "permit");
    assert_eq!(event.data, AddressList {
        addrs: deny_list.clone(),
    });

    for addr in deny_list {
        let resp = service.is_blocked(mock_context(admin.clone()), addr);
        assert_eq!(resp.succeed_data, false);
    }
}

fn new_raw_service() -> AdmissionControlService<SDK> {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()));
    let chain_db = DefaultChainQuerier::new(Arc::new(storage));

    let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
    let state = GeneralServiceState::new(trie);

    let sdk = DefaultServiceSDK::new(
        Rc::new(RefCell::new(state)),
        Rc::new(chain_db),
        NoopDispatcher {},
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
