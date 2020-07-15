use crate::{
    types::{
        ChangeOrgAdmin, ChangeOrgApproved, ChangeServiceAdmin, EvalUserTagExpression, FixedTagList,
        Genesis, GetUserTags, NewOrgEvent, OrgName, RegisterNewOrg, TagName, UpdateUserTags,
        Validate,
    },
    ExpressionDataFeed, KycService, ServiceError, UpdateOrgSupportTags,
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

use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

macro_rules! service_call {
    ($service:expr, $method:ident, $ctx:expr) => {{
        let resp = $service.$method($ctx);
        if resp.is_error() {
            println!("{}", resp.error_message);
        }
        assert!(!resp.is_error());

        resp.succeed_data
    }};

    ($service:expr, $method:ident, $ctx:expr, $payload:expr) => {{
        let resp = $service.$method($ctx, $payload);
        if resp.is_error() {
            println!("{}", resp.error_message);
        }
        assert!(!resp.is_error());

        resp.succeed_data
    }};
}

type SDK = DefaultServiceSDK<
    GeneralServiceState<MemoryDB>,
    DefaultChainQuerier<ImplStorage<MemoryAdapter>>,
    NoopDispatcher,
>;

const CYCLE_LIMIT: u64 = 1024 * 1024 * 1024;
const SERVICE_ADMIN: &str = "0x755cdba6ae4f479f7164792b318b2a06c759833b";
const LI_BING: &str = "0xcff1002107105460941f797828f468667aa1a2db";
const CHEN_TEN: &str = "0x0000000000000000000000000000000000000001";

#[test]
fn should_correctly_init_genesis() {
    let storage = ImplStorage::new(Arc::new(MemoryAdapter::new()));
    let chain_db = DefaultChainQuerier::new(Arc::new(storage));

    let trie = MPTTrie::new(Arc::new(MemoryDB::new(false)));
    let state = GeneralServiceState::new(trie);

    let sdk = DefaultServiceSDK::new(
        Rc::new(RefCell::new(state)),
        Rc::new(chain_db),
        NoopDispatcher {},
    );

    let mut service = KycService::new(sdk);
    let org_name: OrgName = "Da_Lisi".parse().expect("Da_Lisi");
    let genesis = Genesis {
        org_name:        org_name.clone(),
        org_description: "temple ?".to_owned(),
        org_admin:       TestService::service_admin(),
        supported_tags:  vec![],
        service_admin:   TestService::service_admin(),
    };

    service.init_genesis(genesis);

    // Fetch org comes with genesis
    let caller = Address::from_hex(CHEN_TEN).expect("caller");
    let org_names = service_call!(service, get_orgs, mock_context(caller));
    assert_eq!(org_names, vec!["Da_Lisi".parse().expect("Da lisi")]);

    // Change service admin
    let ctx = mock_context(TestService::service_admin());
    let changed = service.change_service_admin(ctx.clone(), ChangeServiceAdmin {
        new_admin: TestService::chen_ten(),
    });
    assert!(!changed.is_error());

    // Change org admin
    let changed = service.change_org_admin(ctx, ChangeOrgAdmin {
        name:      org_name,
        new_admin: TestService::chen_ten(),
    });
    assert!(!changed.is_error());
}

#[test]
fn should_cost_10_000_cycles_per_name_on_get_orgs() {
    let kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());
    let cycles_before = ctx.get_cycles_used();

    service_call!(kyc, get_orgs, ctx.clone());

    let cycles_after = ctx.get_cycles_used();
    // We only have 1 org in genesis
    assert_eq!(cycles_after, cycles_before + 21_000 + 10_000);
}

#[test]
fn should_retrieve_org_info() {
    let kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let genesis = TestService::genesis();
    let opt_org = service_call!(kyc, get_org_info, ctx.clone(), genesis.org_name.clone());

    assert!(opt_org.is_some());
    if let Some(org) = opt_org {
        assert_eq!(org.name, genesis.org_name);
        assert_eq!(org.description, genesis.org_description);
        assert_eq!(org.supported_tags, genesis.supported_tags);
        assert_eq!(org.admin, genesis.org_admin);
    }
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_report_not_found_error_when_retrieve_none_exists_org_info() {
    let kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let got = kyc.get_org_info(ctx.clone(), "JinYiWei".parse().expect("JinYiWei"));

    assert!(got.is_error());
    assert_eq!(
        got.error_message,
        ServiceError::OrgNotFound("JinYiWei".parse().unwrap()).to_string()
    );
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_retrieve_org_supported_tags() {
    let kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let genesis = TestService::genesis();
    let tag_names = service_call!(
        kyc,
        get_org_supported_tags,
        ctx.clone(),
        genesis.org_name.clone()
    );

    assert_eq!(tag_names, genesis.supported_tags);
    assert_eq!(
        ctx.get_cycles_used(),
        21_000 + genesis.supported_tags.len() as u64 * 10_000
    );
}

#[test]
fn should_report_not_found_error_when_retrieve_none_exists_org_supported_tags() {
    let kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let got = kyc.get_org_supported_tags(ctx.clone(), "JinYiWei".parse().expect("JinYiWei"));

    assert!(got.is_error());
    assert_eq!(got.error_message, "Kyc org JinYiWei not found");
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_register_unapproved_org() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let org = RegisterNewOrg {
        name:           "Guan8Train".parse().expect("guan_8"),
        description:    "Help you pass guan 8 exam".to_owned(),
        admin:          TestService::li_bing(),
        supported_tags: vec![
            "level8".parse().expect("level8"),
            "level4".parse().expect("level4"),
        ],
    };

    service_call!(kyc, register_org, ctx.clone(), org.clone());

    let required_cycles = {
        let string_bytes = org.name.len() + org.description.len() + org.admin.as_bytes().len();
        let tags = org.supported_tags.len();

        string_bytes * 1000 + tags * 10_000
    };
    assert_eq!(ctx.get_cycles_used(), 21_000 + required_cycles as u64);

    let opt_registered = service_call!(kyc, get_org_info, ctx.clone(), org.name.clone());
    assert!(opt_registered.is_some());

    if let Some(registered) = opt_registered {
        assert_eq!(registered.name, org.name);
        assert_eq!(registered.description, org.description);
        assert_eq!(registered.admin, org.admin);
        assert_eq!(registered.supported_tags, org.supported_tags);
        assert_eq!(registered.approved, false);
    }

    let events = ctx.get_events();
    assert_eq!(events.len(), 1);

    let event: NewOrgEvent = serde_json::from_str(&events[0].data).expect("parse event");
    assert_eq!(events[0].name, "RegisterOrg");
    assert_eq!(event.name, org.name);
    assert_eq!(event.supported_tags, org.supported_tags);
}

#[test]
fn should_require_admin_to_register_org() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::chen_ten());

    let org = RegisterNewOrg {
        name:           "Guan8Train".parse().expect("guan_8"),
        description:    "Help you pass guan 8 exam".to_owned(),
        admin:          TestService::li_bing(),
        supported_tags: vec![
            "level8".parse().expect("level8"),
            "level4".parse().expect("level4"),
        ],
    };

    let registered = kyc.register_org(ctx.clone(), org);
    assert!(registered.is_error());
    assert_eq!(
        registered.error_message,
        ServiceError::NonAuthorized.to_string()
    );
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_reject_org_registeration_using_too_long_description() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let org = RegisterNewOrg {
        name:           "Guna8Train".parse().expect("guan_8"),
        description:    "pass".repeat(100),
        admin:          TestService::li_bing(),
        supported_tags: vec![],
    };

    let registered = kyc.register_org(ctx.clone(), org.clone());
    assert!(registered.is_error());
    assert_eq!(
        registered.error_message,
        org.validate().err().expect("err").to_string()
    );
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_reject_org_registeration_using_invalid_admin() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let org = RegisterNewOrg {
        name:           "Guna8Train".parse().expect("guan_8"),
        description:    "pass".to_owned(),
        admin:          Address::default(),
        supported_tags: vec![],
    };

    let registered = kyc.register_org(ctx.clone(), org.clone());
    assert!(registered.is_error());
    assert_eq!(
        registered.error_message,
        org.validate().err().expect("err").to_string()
    );
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_reject_already_exists_org_to_register_again() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let genesis = TestService::genesis();
    let org = RegisterNewOrg {
        name:           genesis.org_name,
        description:    genesis.org_description,
        admin:          TestService::li_bing(),
        supported_tags: genesis.supported_tags,
    };

    let registered = kyc.register_org(ctx.clone(), org);
    assert!(registered.is_error());
    assert_eq!(
        registered.error_message,
        ServiceError::OrgAlreadyExists.to_string(),
    );
    assert_eq!(ctx.get_cycles_used(), 21_000);
}

#[test]
fn should_update_org_supported_tags() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let genesis = TestService::genesis();
    service_call!(
        kyc,
        update_supported_tags,
        ctx.clone(),
        UpdateOrgSupportTags {
            org_name:       genesis.org_name.clone(),
            supported_tags: vec!["level1".parse().expect("level1")],
        }
    );
    assert_eq!(ctx.get_cycles_used(), 21_000 + 10_000);

    let tag_names = service_call!(kyc, get_org_supported_tags, ctx, genesis.org_name);
    assert_eq!(tag_names, vec!["level1".parse().expect("level1")]);
}

#[test]
fn should_reject_update_org_supported_tags_without_admin_permission() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::li_bing());

    let genesis = TestService::genesis();
    let updated = kyc.update_supported_tags(ctx, UpdateOrgSupportTags {
        org_name:       genesis.org_name,
        supported_tags: vec!["level1".parse().expect("level1")],
    });

    assert!(updated.is_error());
    assert_eq!(
        updated.error_message,
        ServiceError::NonAuthorized.to_string()
    );
}

#[test]
fn should_report_not_found_error_for_none_exists_org_on_update_org_supported_tags() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let updated = kyc.update_supported_tags(ctx, UpdateOrgSupportTags {
        org_name:       "JinYiWei".parse().expect("JinYiWei"),
        supported_tags: vec!["level1".parse().expect("level1")],
    });

    assert!(updated.is_error());
    assert_eq!(
        updated.error_message,
        ServiceError::OrgNotFound("JinYiWei".parse().unwrap()).to_string()
    );
}

#[test]
fn should_update_user_tags() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::li_bing());

    let genesis = TestService::genesis();
    let mut tags: HashMap<TagName, FixedTagList> = HashMap::new();
    tags.insert(
        "title".parse().unwrap(),
        FixedTagList::from_vec(vec!["ZaYi".parse().unwrap()]).expect("fixed tag list"),
    );
    tags.insert(
        "speci".parse().unwrap(),
        FixedTagList::from_vec(vec!["Human".parse().unwrap()]).expect("fixed tag list"),
    );
    tags.insert(
        "skills".parse().unwrap(),
        FixedTagList::from_vec(vec!["Guan1".parse().unwrap()]).expect("fixed tag list"),
    );

    let update_user_tags = UpdateUserTags {
        org_name: genesis.org_name.clone(),
        user:     TestService::chen_ten(),
        tags:     tags.clone(),
    };
    service_call!(kyc, update_user_tags, ctx.clone(), update_user_tags.clone());
    assert_eq!(ctx.get_cycles_used(), 21_000 + (3 + 3) * 10_000);

    let events = ctx.get_events();
    assert_eq!(events.len(), 1);
    let event: UpdateUserTags = serde_json::from_str(&events[0].data).expect("parse event");
    assert_eq!(events[0].name, "UpdateUserTag");
    assert_eq!(event, update_user_tags);

    let updated_tags = service_call!(kyc, get_user_tags, ctx, GetUserTags {
        org_name: genesis.org_name,
        user:     TestService::chen_ten(),
    });
    assert_eq!(updated_tags, tags);
}

#[test]
fn should_remove_unused_previous_user_tags_after_update_user_tags() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::li_bing());

    let genesis = TestService::genesis();
    let mut tags: HashMap<TagName, FixedTagList> = HashMap::new();
    tags.insert(
        "title".parse().unwrap(),
        FixedTagList::from_vec(vec!["ZaYi".parse().unwrap()]).expect("fixed tag list"),
    );

    service_call!(kyc, update_user_tags, ctx.clone(), UpdateUserTags {
        org_name: genesis.org_name.clone(),
        user:     TestService::chen_ten(),
        tags:     tags.clone(),
    });

    let updated_tags = service_call!(kyc, get_user_tags, ctx.clone(), GetUserTags {
        org_name: genesis.org_name.clone(),
        user:     TestService::chen_ten(),
    });
    assert_eq!(updated_tags, tags);

    tags.clear();
    tags.insert(
        "skills".parse().unwrap(),
        FixedTagList::from_vec(vec!["Guan1".parse().unwrap()]).expect("fixed tag list"),
    );

    // title isn't included, so will be removed.
    service_call!(kyc, update_user_tags, ctx.clone(), UpdateUserTags {
        org_name: genesis.org_name.clone(),
        user:     TestService::chen_ten(),
        tags:     tags.clone(),
    });

    let updated_tags = service_call!(kyc, get_user_tags, ctx, GetUserTags {
        org_name: genesis.org_name.clone(),
        user:     TestService::chen_ten(),
    });
    assert_eq!(updated_tags, tags);

    let maybe_title = kyc.get_tags(
        TestService::chen_ten(),
        genesis.org_name.to_string(),
        "title".to_owned(),
    );
    assert_eq!(maybe_title, Ok(vec!["NULL".to_owned()]));
}

#[test]
fn should_report_not_found_error_for_none_exists_org_on_update_user_tags() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::li_bing());

    let updated = kyc.update_user_tags(ctx, UpdateUserTags {
        org_name: "JinYiWei".parse().unwrap(),
        user:     TestService::chen_ten(),
        tags:     HashMap::new(),
    });

    assert!(updated.is_error());
    assert_eq!(
        updated.error_message,
        ServiceError::OrgNotFound("JinYiWei".parse().unwrap()).to_string()
    );
}

#[test]
fn should_reject_unapproved_org_to_update_user_tags() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let org = RegisterNewOrg {
        name:           "Guan8Train".parse().expect("guan_8"),
        description:    "Help you pass guan 8 exam".to_owned(),
        admin:          TestService::li_bing(),
        supported_tags: vec![
            "level8".parse().expect("level8"),
            "level4".parse().expect("level4"),
        ],
    };

    service_call!(kyc, register_org, ctx.clone(), org.clone());
    let opt_registered = service_call!(kyc, get_org_info, ctx, org.name);
    assert_eq!(opt_registered.as_ref().map(|o| o.approved), Some(false));

    let ctx = mock_context(TestService::li_bing());
    let updated = kyc.update_user_tags(ctx, UpdateUserTags {
        org_name: "Guan8Train".parse().unwrap(),
        user:     TestService::li_bing(),
        tags:     HashMap::new(),
    });

    assert!(updated.is_error());
    assert_eq!(
        updated.error_message,
        ServiceError::UnapprovedOrg.to_string()
    );
}

#[test]
fn should_reject_update_user_tags_from_none_org_admin() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::chen_ten());

    let genesis = TestService::genesis();
    let mut tags: HashMap<TagName, FixedTagList> = HashMap::new();
    tags.insert(
        "title".parse().unwrap(),
        FixedTagList::from_vec(vec!["ZaYi".parse().unwrap()]).expect("fixed tag list"),
    );

    let updated = kyc.update_user_tags(ctx, UpdateUserTags {
        org_name: genesis.org_name,
        user: TestService::chen_ten(),
        tags,
    });

    assert!(updated.is_error());
    assert_eq!(
        updated.error_message,
        ServiceError::NonAuthorized.to_string()
    );
}

#[test]
fn should_reject_change_org_admin_from_none_org_admin() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::chen_ten());

    let genesis = TestService::genesis();
    let changed = kyc.change_org_admin(ctx, ChangeOrgAdmin {
        name:      genesis.org_name,
        new_admin: TestService::chen_ten(),
    });

    assert!(changed.is_error());
    assert_eq!(
        changed.error_message,
        ServiceError::NonAuthorized.to_string()
    );
}

#[test]
fn should_report_not_found_error_for_change_none_exists_org_admin() {
    let mut kyc = TestService::new();

    let ctx = mock_context(TestService::service_admin());
    let changed = kyc.change_org_admin(ctx, ChangeOrgAdmin {
        name:      "JinYiWei".parse().unwrap(),
        new_admin: TestService::chen_ten(),
    });

    assert!(changed.is_error());
    assert_eq!(
        changed.error_message,
        ServiceError::OrgNotFound("JinYiWei".parse().unwrap()).to_string()
    );
}

#[test]
fn should_reject_to_approve_org_from_none_service_admin() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let org = RegisterNewOrg {
        name:           "Guan8Train".parse().expect("guan_8"),
        description:    "Help you pass guan 8 exam".to_owned(),
        admin:          TestService::li_bing(),
        supported_tags: vec![
            "level8".parse().expect("level8"),
            "level4".parse().expect("level4"),
        ],
    };

    service_call!(kyc, register_org, ctx, org.clone());

    let ctx = mock_context(TestService::li_bing());
    let approved = kyc.change_org_approved(ctx, ChangeOrgApproved {
        org_name: org.name,
        approved: true,
    });

    assert!(approved.is_error());
    assert_eq!(
        approved.error_message,
        ServiceError::NonAuthorized.to_string()
    );
}

#[test]
fn should_reject_to_approve_none_exists_org() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::service_admin());

    let approved = kyc.change_org_approved(ctx, ChangeOrgApproved {
        org_name: "JinYiWei".parse().unwrap(),
        approved: true,
    });

    assert!(approved.is_error());
    assert_eq!(
        approved.error_message,
        ServiceError::OrgNotFound("JinYiWei".parse().unwrap()).to_string()
    );
}

#[test]
fn should_eval_user_tag_expression() {
    let mut kyc = TestService::new();
    let ctx = mock_context(TestService::li_bing());

    let genesis = TestService::genesis();
    let mut tags: HashMap<TagName, FixedTagList> = HashMap::new();
    tags.insert(
        "title".parse().unwrap(),
        FixedTagList::from_vec(vec!["ZaYi".parse().unwrap()]).expect("fixed tag list"),
    );
    tags.insert(
        "speci".parse().unwrap(),
        FixedTagList::from_vec(vec!["Human".parse().unwrap()]).expect("fixed tag list"),
    );
    tags.insert(
        "skills".parse().unwrap(),
        FixedTagList::from_vec(vec!["Guan1".parse().unwrap()]).expect("fixed tag list"),
    );

    let update_user_tags = UpdateUserTags {
        org_name: genesis.org_name,
        user: TestService::chen_ten(),
        tags,
    };
    service_call!(kyc, update_user_tags, ctx.clone(), update_user_tags);

    let evaluated = kyc.eval_user_tag_expression(ctx.clone(), EvalUserTagExpression {
        user:       TestService::chen_ten(),
        expression: "Da_Lisi.title@`ZaYi`".to_owned(),
    });

    assert!(!evaluated.is_error());
    assert_eq!(evaluated.succeed_data, true);

    let evaluated = kyc.eval_user_tag_expression(ctx, EvalUserTagExpression {
        user:       TestService::chen_ten(),
        expression: "Da_Lisi.speci@`Cat`".to_owned(),
    });
    assert!(!evaluated.is_error());
    assert_eq!(evaluated.succeed_data, false);
}

struct TestService(KycService<SDK>);

impl Deref for TestService {
    type Target = KycService<SDK>;

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

        let sdk = DefaultServiceSDK::new(
            Rc::new(RefCell::new(state)),
            Rc::new(chain_db),
            NoopDispatcher {},
        );

        let mut service = KycService::new(sdk);
        service.init_genesis(Self::genesis());

        TestService(service)
    }

    fn service_admin() -> Address {
        Address::from_hex(SERVICE_ADMIN).expect("service admin")
    }

    fn li_bing() -> Address {
        Address::from_hex(LI_BING).expect("li bing")
    }

    fn chen_ten() -> Address {
        Address::from_hex(CHEN_TEN).expect("chen ten")
    }

    fn genesis() -> Genesis {
        let supported_tags = vec![
            "title".parse().expect("title"),
            "speci".parse().expect("speci"),
            "skills".parse().expect("skills"),
        ];

        Genesis {
            org_name: "Da_Lisi".parse().expect("Da_Lisi"),
            org_description: "temple ?".to_owned(),
            org_admin: Self::li_bing(),
            supported_tags,
            service_admin: Self::service_admin(),
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
