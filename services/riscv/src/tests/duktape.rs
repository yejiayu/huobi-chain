use std::time::{SystemTime, UNIX_EPOCH};

use protocol::{
    types::{Hash, ServiceContext},
    Bytes,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{TestContext, TestRiscvService, CALLER, CYCLE_LIMIT};
use crate::types::{DeployPayload, ExecPayload, InterpreterType};

macro_rules! deploy_test_code {
    () => {{
        let mut context = TestContext::default();
        let mut service = TestRiscvService::new();

        // No init
        let code = include_str!("./test_code.js");
        let payload = DeployPayload {
            code:      hex::encode(Bytes::from(code)),
            intp_type: InterpreterType::Duktape,
            init_args: "".into(),
        };

        let ret = service!(service, deploy, context.make_admin(), payload);
        assert_eq!(ret.init_ret, "");

        (service, context, ret.address)
    }};
}

#[test]
fn should_support_pvm_init() {
    let (mut service, mut context, ..) = deploy_test_code!();

    let code = include_str!("./test_code.js");
    let payload = DeployPayload {
        code:      hex::encode(Bytes::from(code)),
        intp_type: InterpreterType::Duktape,
        init_args: "do init".into(),
    };

    let ret = service!(service, deploy, context.make_admin(), payload);
    assert_eq!(ret.init_ret, "do init");
}

#[test]
fn should_support_pvm_load_args() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_load_args"}).to_string();
    let payload = ExecPayload::new(address, args.clone());

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret, args);
}

#[test]
fn should_support_pvm_load_json_args() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_load_json_args"}).to_string();
    let payload = ExecPayload::new(address, args.clone());

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret, args);
}

#[test]
fn should_support_pvm_cycle_limit() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_cycle_limit"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret.parse::<u64>().expect("cycle limit"), CYCLE_LIMIT);
}

#[test]
fn should_support_pvm_cycle_used() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_cycle_used"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    // Hardcode in context make
    assert_eq!(ret.parse::<u64>().expect("cycle used"), 3);
}

#[test]
fn should_support_pvm_cycle_price() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_cycle_price"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    // Hardcode in context make
    assert_eq!(ret.parse::<u64>().expect("cycle price"), 1);
}

#[test]
fn should_support_pvm_caller() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_caller"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret, CALLER);
}

#[test]
fn should_support_pvm_origin() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args =
        json!({"method": "test_origin", "address": address.as_hex(), "call_args": json!({"method": "_ret_caller_and_origin"}).to_string()})
            .to_string();

    let payload = ExecPayload::new(address.clone(), args);
    let ret = service!(service, exec, context.make(), payload);

    #[derive(Debug, Deserialize)]
    struct ExpectRet {
        caller: String,
        origin: String,
    }

    let ret: ExpectRet = serde_json::from_str(&ret).expect("decode test origin ret");
    assert_eq!(ret.caller, address.as_hex());
    assert_eq!(ret.origin, CALLER);
}

#[test]
fn should_support_pvm_address() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_address"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret, address.as_hex());
}

#[test]
fn should_support_pvm_block_height() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_block_height"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ctx = context.make();
    let ret = service!(service, exec, ctx.clone(), payload);

    assert_eq!(
        ret.parse::<u64>().expect("block height"),
        ctx.get_current_height()
    );
}

#[test]
fn should_support_pvm_extra() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_no_extra"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret, "no extra");

    // Should return extra data
    let extra = "final mixed ??? no !!!";
    let mut ctx_params = context.new_params();
    ctx_params.extra = Some(Bytes::from(extra));
    let ctx = ServiceContext::new(ctx_params);

    let args = json!({"method": "test_extra"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, ctx, payload);
    assert_eq!(ret, extra);
}

#[test]
fn should_support_pvm_timestamp() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_timestamp"}).to_string();
    let payload = ExecPayload::new(address, args);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("timestamp")
        .as_secs();

    let mut ctx_params = context.new_params();
    ctx_params.timestamp = now;
    let ctx = ServiceContext::new(ctx_params);

    let ret = service!(service, exec, ctx.clone(), payload);
    assert_eq!(ret.parse::<u64>().expect("timestamp"), ctx.get_timestamp());
}

#[test]
fn should_support_pvm_emit_event() {
    let (mut service, mut context, address) = deploy_test_code!();

    let msg = "emit test event";
    let args = json!({"method": "test_emit_event", "msg": msg}).to_string();
    let payload = ExecPayload::new(address, args);

    let ctx = context.make();
    let ret = service!(service, exec, ctx.clone(), payload);
    assert_eq!(ret, "emit success");

    let events = ctx.get_events();
    assert!(events.iter().any(|ev| ev.data == msg));
}

#[test]
fn should_support_pvm_tx_hash() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_tx_hash"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ctx = context.make();
    let ret = service!(service, exec, ctx.clone(), payload);

    assert_eq!(
        Some(ret),
        ctx.get_tx_hash().map(|h| h.as_hex()),
        "should return tx hash"
    );

    // No tx hash
    let mut ctx_params = context.new_params();
    ctx_params.tx_hash = None;
    let ctx = ServiceContext::new(ctx_params);

    let args = json!({"method": "test_no_tx_hash"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, ctx, payload);
    assert_eq!(ret, "no tx hash");
}

#[test]
fn should_support_pvm_tx_nonce() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_no_tx_nonce"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ctx = context.make();
    let ret = service!(service, exec, ctx, payload);

    assert_eq!(ret, "no tx nonce");

    // Should return tx nonce
    let mut ctx_params = context.new_params();
    ctx_params.nonce = Some(Hash::digest(Bytes::from("test_nonce".to_owned())));
    let ctx = ServiceContext::new(ctx_params);

    let args = json!({"method": "test_tx_nonce"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, ctx.clone(), payload);
    assert_eq!(Some(ret), ctx.get_nonce().map(|n| n.as_hex()));
}

#[test]
fn should_support_pvm_storage() {
    let (mut service, mut context, address) = deploy_test_code!();

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct Carmen {
        color: String,
    }

    let carmen = json!({"color": "red"}).to_string();
    let args = json!({"method": "test_storage", "key": "carmen", "val": carmen}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    let ret: Carmen = serde_json::from_str(&ret).expect("get json storage");

    assert_eq!(ret.color, "red");
}

#[test]
fn should_support_pvm_contract_call() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args =
        json!({"method": "test_contract_call", "address": address.as_hex(), "call_args": json!({"method": "_ret_self"}).to_string()})
            .to_string();

    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    assert_eq!(ret, "self");
}

#[test]
fn should_support_pvm_service_call() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({
        "method": "test_service_call",
        "call_service": "riscv",
        "call_method": "exec",
        "call_payload": json!({
            "address": address.as_hex(),
            "args": json!({
                "method": "_ret_self",
            }).to_string(),
        }).to_string(),
    })
    .to_string();

    let payload = ExecPayload::new(address, args);

    let ret = service!(service, exec, context.make(), payload);
    let expect_ret = serde_json::to_string("self").expect("should be json encoded");
    assert_eq!(ret, expect_ret);
}
