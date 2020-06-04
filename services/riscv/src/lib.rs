#[cfg(test)]
mod tests;
pub mod types;
pub mod vm;

use std::cell::RefCell;
use std::rc::Rc;

use derive_more::{Display, From};

use binding_macro::{genesis, read, service, write};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreBool, StoreMap};
use protocol::types::{Address, Hash, ServiceContext};
use protocol::{Bytes, BytesMut};

use crate::types::{
    Addresses, Contract, DeployPayload, DeployResp, ExecPayload, GetContractPayload,
    GetContractResp, InitGenesisPayload,
};
use crate::vm::{ChainInterface, Interpreter, InterpreterConf, InterpreterParams};

macro_rules! sub_cycles {
    ($ctx:expr, $cycles:expr) => {
        if !$ctx.sub_cycles($cycles) {
            return ServiceError::OutOfCycles.into();
        }
    };
}

pub struct RiscvService<SDK> {
    sdk:              Rc<RefCell<SDK>>,
    deploy_auth:      Box<dyn StoreMap<Address, bool>>,
    admins:           Box<dyn StoreMap<Address, bool>>,
    enable_whitelist: Box<dyn StoreBool>,
}

#[service]
impl<SDK: ServiceSDK + 'static> RiscvService<SDK> {
    pub fn init(sdk: SDK) -> Self {
        let sdk = Rc::new(RefCell::new(sdk));

        let deploy_auth = sdk.borrow_mut().alloc_or_recover_map("deploy_auth");
        let enable_whitelist = sdk.borrow_mut().alloc_or_recover_bool("enable_whitelist");
        let admins = sdk.borrow_mut().alloc_or_recover_map("admins");

        Self {
            sdk,
            deploy_auth,
            enable_whitelist,
            admins,
        }
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        if payload.enable_whitelist && payload.admins.is_empty() {
            panic!(
                "If riscv whitelist is enabled, you should set at least one admin in genesis.toml"
            );
        }

        self.enable_whitelist.set(payload.enable_whitelist);

        for addr in payload.whitelist {
            self.deploy_auth.insert(addr, true);
        }
        for addr in payload.admins {
            self.admins.insert(addr, true);
        }
    }

    fn run(
        &self,
        ctx: ServiceContext,
        payload: ExecPayload,
        is_init: bool,
    ) -> ServiceResponse<String> {
        let contract = match self.sdk.borrow().get_value::<_, Contract>(&payload.address) {
            Some(c) => c,
            None => return ServiceError::ContractNotFound(payload.address.as_hex()).into(),
        };
        let code = match self.sdk.borrow().get_value::<_, Bytes>(&contract.code_hash) {
            Some(c) => c,
            None => return ServiceError::CodeNotFound.into(),
        };

        let interpreter_params = InterpreterParams {
            address: payload.address.clone(),
            code,
            args: payload.args.clone().into(),
            is_init,
        };
        let chain_interface =
            ChainInterfaceImpl::new(ctx.clone(), payload, Rc::<_>::clone(&self.sdk));

        let mut interpreter = Interpreter::new(
            ctx.clone(),
            InterpreterConf::default(),
            contract.intp_type,
            interpreter_params,
            Rc::new(RefCell::new(chain_interface)),
        );

        match interpreter.run() {
            Ok(int_ret) if int_ret.ret_code == 0 => {
                sub_cycles!(ctx, int_ret.cycles_used);

                let ret = String::from_utf8_lossy(int_ret.ret.as_ref()).to_string();
                ServiceResponse::from_succeed(ret)
            }
            Ok(int_ret) => ServiceError::NonZeroExitCode {
                exitcode: int_ret.ret_code,
                ret:      String::from_utf8_lossy(int_ret.ret.as_ref()).to_string(),
            }
            .into(),
            Err(err) => ServiceError::CkbVm(err).into(),
        }
    }

    #[read]
    fn call(&self, ctx: ServiceContext, payload: ExecPayload) -> ServiceResponse<String> {
        self.run(ctx, payload, false)
    }

    #[write]
    fn exec(&mut self, ctx: ServiceContext, payload: ExecPayload) -> ServiceResponse<String> {
        self.run(ctx, payload, false)
    }

    #[write]
    fn grant_deploy_auth(
        &mut self,
        ctx: ServiceContext,
        payload: Addresses,
    ) -> ServiceResponse<()> {
        if self.no_authority(&ctx) {
            return ServiceError::NonAuthorized.into();
        }
        sub_cycles!(ctx, payload.addresses.len() as u64 * 10_000);

        for addr in payload.addresses {
            self.deploy_auth.insert(addr, true);
        }
        ServiceResponse::from_succeed(())
    }

    #[write]
    fn revoke_deploy_auth(
        &mut self,
        ctx: ServiceContext,
        payload: Addresses,
    ) -> ServiceResponse<()> {
        if self.no_authority(&ctx) {
            return ServiceError::NonAuthorized.into();
        }
        sub_cycles!(ctx, payload.addresses.len() as u64 * 10_000);

        for addr in payload.addresses {
            self.deploy_auth.remove(&addr);
        }
        ServiceResponse::from_succeed(())
    }

    #[read]
    fn check_deploy_auth(
        &self,
        ctx: ServiceContext,
        payload: Addresses,
    ) -> ServiceResponse<Addresses> {
        let mut res = Addresses::default();
        sub_cycles!(ctx, payload.addresses.len() as u64 * 1000);

        for addr in payload.addresses {
            if self.deploy_auth.contains(&addr) {
                res.addresses.push(addr);
            }
        }
        ServiceResponse::from_succeed(res)
    }

    #[write]
    fn deploy(
        &mut self,
        ctx: ServiceContext,
        payload: DeployPayload,
    ) -> ServiceResponse<DeployResp> {
        // Check authority list
        let enable_whitelist = self.enable_whitelist.get();
        if enable_whitelist && !self.deploy_auth.contains(&ctx.get_caller()) {
            return ServiceError::NonAuthorized.into();
        }

        let code = match hex::decode(&payload.code) {
            Ok(c) => Bytes::from(c),
            Err(err) => return ServiceError::HexDecode(err).into(),
        };

        // Save code
        let code_hash = Hash::digest(code.clone());
        let code_len = code.len() as u64;

        // Every bytes cost 10 cycles
        sub_cycles!(ctx, code_len * 10);
        self.sdk.borrow_mut().set_value(code_hash.clone(), code);

        // Generate contract address
        let tx_hash = match ctx.get_tx_hash() {
            Some(h) => h,
            None => return ServiceError::NotInExecContext("riscv deploy".to_owned()).into(),
        };
        let addr_in_bytes = Hash::digest(tx_hash.as_bytes()).as_bytes().slice(0..20);
        let contract_address = match Address::from_bytes(addr_in_bytes) {
            Ok(a) => a,
            Err(_) => return ServiceError::InvalidContractAddress.into(),
        };

        let intp_type = payload.intp_type;
        let contract = Contract::new(code_hash, intp_type);
        self.sdk
            .borrow_mut()
            .set_value(contract_address.clone(), contract);

        if payload.init_args.is_empty() {
            return ServiceResponse::from_succeed(DeployResp {
                address:  contract_address,
                init_ret: String::new(),
            });
        }

        // Run init
        let init_payload = ExecPayload {
            address: contract_address.clone(),
            args:    payload.init_args,
        };

        let resp = self.run(ctx, init_payload, true);
        if resp.is_error() {
            ServiceResponse::from_error(resp.code, resp.error_message)
        } else {
            ServiceResponse::from_succeed(DeployResp {
                address:  contract_address,
                init_ret: resp.succeed_data,
            })
        }
    }

    #[read]
    fn get_contract(
        &self,
        ctx: ServiceContext,
        payload: GetContractPayload,
    ) -> ServiceResponse<GetContractResp> {
        sub_cycles!(ctx, 21000);

        let contract = match self.sdk.borrow().get_value::<_, Contract>(&payload.address) {
            Some(c) => c,
            None => return ServiceError::ContractNotFound(payload.address.as_hex()).into(),
        };

        let mut resp = GetContractResp {
            code_hash: contract.code_hash.clone(),
            intp_type: contract.intp_type,
            ..Default::default()
        };

        if payload.get_code {
            let code = match self.sdk.borrow().get_value::<_, Bytes>(&contract.code_hash) {
                Some(c) => c,
                None => return ServiceError::CodeNotFound.into(),
            };
            sub_cycles!(ctx, code.len() as u64);

            resp.code = hex::encode(&code);
        }

        for key in payload.storage_keys.iter() {
            sub_cycles!(ctx, key.len() as u64);
            let decoded_key = match hex::decode(key) {
                Ok(k) => k,
                Err(_) => return ServiceError::InvalidKey(key.to_owned()).into(),
            };

            let addr_bytes = payload.address.as_bytes();
            let contract_key = combine_key(addr_bytes.as_ref(), &decoded_key);

            let value = self
                .sdk
                .borrow()
                .get_value::<_, Bytes>(&contract_key)
                .unwrap_or_default();
            sub_cycles!(ctx, value.len() as u64);

            resp.storage_values.push(hex::encode(value));
        }

        ServiceResponse::from_succeed(resp)
    }

    fn no_authority(&self, ctx: &ServiceContext) -> bool {
        self.admins.contains(&ctx.get_caller())
    }
}

struct ChainInterfaceImpl<SDK> {
    ctx:             ServiceContext,
    payload:         ExecPayload,
    sdk:             Rc<RefCell<SDK>>,
    all_cycles_used: u64,
}

impl<SDK: ServiceSDK + 'static> ChainInterfaceImpl<SDK> {
    fn new(ctx: ServiceContext, payload: ExecPayload, sdk: Rc<RefCell<SDK>>) -> Self {
        Self {
            ctx,
            payload,
            sdk,
            all_cycles_used: 0,
        }
    }

    fn contract_key(&self, key: &Bytes) -> Hash {
        combine_key(self.payload.address.as_bytes().as_ref(), key)
    }
}

impl<SDK> ChainInterface for ChainInterfaceImpl<SDK>
where
    SDK: ServiceSDK + 'static,
{
    fn get_storage(&self, key: &Bytes) -> Bytes {
        let contract_key = self.contract_key(key);

        self.sdk
            .borrow()
            .get_value::<Hash, Bytes>(&contract_key)
            .unwrap_or_default()
    }

    fn set_storage(&mut self, key: Bytes, val: Bytes) {
        let contract_key = self.contract_key(&key);
        self.sdk.borrow_mut().set_value(contract_key, val)
    }

    fn contract_call(
        &mut self,
        address: Address,
        args: Bytes,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        let payload = ExecPayload {
            address,
            args: String::from_utf8_lossy(args.as_ref()).to_string(),
        };

        let payload_json = match serde_json::to_string(&payload) {
            Ok(j) => j,
            Err(err) => return ServiceError::Serde(err).into(),
        };

        let resp = self.service_call("riscv", "exec", &payload_json, current_cycle, false);
        if resp.is_error() {
            return ServiceResponse::from_error(resp.code, resp.error_message);
        }

        let (json_ret, cycle) = resp.succeed_data;
        let raw_ret = match serde_json::from_str(&json_ret) {
            Ok(r) => r,
            Err(err) => return ServiceError::Serde(err).into(),
        };

        ServiceResponse::from_succeed((raw_ret, cycle))
    }

    fn service_call(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
        readonly: bool,
    ) -> ServiceResponse<(String, u64)> {
        let vm_cycle = current_cycle.wrapping_sub(self.all_cycles_used);
        if vm_cycle != 0 {
            sub_cycles!(self.ctx, vm_cycle);
        } else {
            return ServiceError::OutOfCycles.into();
        }

        let payload_addr = self.payload.address.as_hex();
        let call_resp = if readonly {
            self.sdk.borrow().read(
                &self.ctx,
                Some(Bytes::from(payload_addr)),
                service,
                method,
                payload,
            )
        } else {
            self.sdk.borrow_mut().write(
                &self.ctx,
                Some(Bytes::from(payload_addr)),
                service,
                method,
                payload,
            )
        };
        if call_resp.is_error() {
            return ServiceResponse::from_error(call_resp.code, call_resp.error_message);
        }

        self.all_cycles_used = self.ctx.get_cycles_used();
        let call_ret: String = call_resp.succeed_data;
        ServiceResponse::from_succeed((call_ret, self.all_cycles_used))
    }
}

fn combine_key(addr: &[u8], key: &[u8]) -> Hash {
    let mut buf = BytesMut::from(addr);
    buf.extend(key);
    Hash::digest(buf.freeze())
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    #[display(fmt = "Method {} can not be invoke with call", _0)]
    NotInExecContext(String),

    #[display(fmt = "Contract {} not exists", _0)]
    ContractNotFound(String),

    #[display(fmt = "Code not found")]
    CodeNotFound,

    #[display(fmt = "None zero exit {} msg {}", exitcode, ret)]
    NonZeroExitCode { exitcode: i8, ret: String },

    #[display(fmt = "VM: {:?}", _0)]
    CkbVm(ckb_vm::Error),

    #[display(fmt = "Codec: {:?}", _0)]
    Serde(serde_json::error::Error),

    #[display(fmt = "Hex decode: {:?}", _0)]
    HexDecode(hex::FromHexError),

    #[display(fmt = "Invalid key '{:?}', should be a hex string", _0)]
    InvalidKey(String),

    #[display(fmt = "Not authorized")]
    NonAuthorized,

    #[display(fmt = "Out of cycles")]
    OutOfCycles,

    #[display(fmt = "Invalid contract address")]
    InvalidContractAddress,
}

impl ServiceError {
    fn code(&self) -> u64 {
        use ServiceError::*;

        match self {
            NotInExecContext(_) => 101,
            ContractNotFound(_) => 102,
            CodeNotFound => 103,
            NonZeroExitCode { .. } => 104,
            CkbVm(_) => 105,
            Serde(_) => 106,
            HexDecode(_) => 107,
            InvalidKey(_) => 108,
            NonAuthorized => 109,
            OutOfCycles => 110,
            InvalidContractAddress => 111,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
