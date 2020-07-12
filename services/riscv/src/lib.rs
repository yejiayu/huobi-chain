mod common;
#[cfg(test)]
mod tests;

pub mod authorization;
pub mod error;
pub mod types;
pub mod vm;

use authorization::Authorization;
use error::ServiceError;
use types::{
    AddressList, Authorizer, Contract, DeployPayload, DeployResp, ExecPayload, GetContractPayload,
    GetContractResp, InitGenesisPayload,
};
use vm::{
    Cause, ChainInterface, Interpreter, InterpreterParams, ReadonlyChain, WriteableChain,
    SYSCODE_ASSERT,
};

use binding_macro::{genesis, read, service, write};
use protocol::{
    traits::{ExecutorParams, ServiceResponse, ServiceSDK},
    types::{Address, Hash, ServiceContext},
    Bytes,
};
use serde::Serialize;

use std::{cell::RefCell, rc::Rc};

#[macro_export]
macro_rules! sub_cycles {
    ($ctx:expr, $cycles:expr) => {
        if !$ctx.sub_cycles($cycles) {
            return ServiceError::OutOfCycles.into();
        }
    };
}

macro_rules! require_admin {
    ($authorization:expr, $ctx:expr) => {
        if !$authorization.is_admin($ctx) {
            return ServiceError::NonAuthorized.into();
        }
    };
}

macro_rules! require_approved {
    ($authorization:expr, $contract_address:expr) => {
        if !$authorization.granted($contract_address, authorization::Kind::Contract) {
            return ServiceError::NonAuthorized.into();
        }
    };
}

pub struct RiscvService<SDK> {
    sdk:           Rc<RefCell<SDK>>,
    authorization: Authorization,
}

#[service]
impl<SDK: ServiceSDK + 'static> RiscvService<SDK> {
    pub fn init(sdk: SDK) -> Self {
        let sdk = Rc::new(RefCell::new(sdk));
        let authorization = Authorization::new(&sdk);

        Self { sdk, authorization }
    }

    // # Panic
    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        self.authorization.init_genesis(payload);
    }

    #[read]
    fn call(&self, ctx: ServiceContext, payload: ExecPayload) -> ServiceResponse<String> {
        require_approved!(self.authorization, &payload.address);

        let (contract, code) = match self.load_contract_code(&payload.address) {
            Ok(c) => c,
            Err(err) => return err.into(),
        };

        let intp_params =
            InterpreterParams::new(payload.address.clone(), code, payload.args.clone().into());
        let readonly_chain = Rc::new(RefCell::new(ReadonlyChain::new(
            ctx.clone(),
            payload,
            Rc::<_>::clone(&self.sdk),
        )));

        self.run_interpreter(ctx, contract, readonly_chain, intp_params)
    }

    #[read]
    fn check_deploy_auth(
        &self,
        ctx: ServiceContext,
        payload: AddressList,
    ) -> ServiceResponse<AddressList> {
        let mut res = AddressList::default();
        sub_cycles!(ctx, payload.addresses.len() as u64 * 1000);

        let authorization = &self.authorization;
        for addr in payload.addresses {
            if authorization.contains(&addr, authorization::Kind::Deploy) {
                res.addresses.push(addr);
            }
        }
        ServiceResponse::from_succeed(res)
    }

    #[read]
    fn get_contract(
        &self,
        ctx: ServiceContext,
        payload: GetContractPayload,
    ) -> ServiceResponse<GetContractResp> {
        sub_cycles!(ctx, 21000);

        let maybe_c2 = if payload.get_code {
            self.load_contract_code(&payload.address)
        } else {
            self.load_contract(&payload.address)
                .map(|contract| (contract, Bytes::new()))
        };

        let (contract, code) = match maybe_c2 {
            Ok(c2) => c2, // C.C. Geass
            Err(e) => return e.into(),
        };

        let code = if !code.is_empty() {
            sub_cycles!(ctx, code.len() as u64);
            hex::encode(&code)
        } else {
            String::new()
        };

        let mut resp = GetContractResp {
            code_hash: contract.code_hash.clone(),
            intp_type: contract.intp_type,
            code,
            authorizer: contract.authorizer,
            ..Default::default()
        };

        for key in payload.storage_keys.iter() {
            sub_cycles!(ctx, key.len() as u64);
            let decoded_key = match hex::decode(key) {
                Ok(k) => k,
                Err(_) => return ServiceError::InvalidKey(key.to_owned()).into(),
            };

            let addr_bytes = payload.address.as_bytes();
            let contract_key = common::combine_key(addr_bytes.as_ref(), &decoded_key);

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

    #[write]
    fn exec(&mut self, ctx: ServiceContext, payload: ExecPayload) -> ServiceResponse<String> {
        require_approved!(self.authorization, &payload.address);

        let (contract, code) = match self.load_contract_code(&payload.address) {
            Ok(c) => c,
            Err(err) => return err.into(),
        };

        let intp_params =
            InterpreterParams::new(payload.address.clone(), code, payload.args.clone().into());
        let writeable_chain = Rc::new(RefCell::new(WriteableChain::new(
            ctx.clone(),
            payload,
            Rc::<_>::clone(&self.sdk),
        )));

        self.run_interpreter(ctx, contract, writeable_chain, intp_params)
    }

    #[write]
    fn grant_deploy_auth(
        &mut self,
        ctx: ServiceContext,
        payload: AddressList,
    ) -> ServiceResponse<()> {
        require_admin!(self.authorization, &ctx);
        sub_cycles!(ctx, payload.addresses.len() as u64 * 10_000);

        let authorization_mut = &mut self.authorization;
        let auth_kind = authorization::Kind::Deploy;

        for addr in payload.addresses.clone() {
            authorization_mut.grant(addr, auth_kind, Authorizer::new(ctx.get_caller()));
        }

        Self::emit_event(&ctx, "GrantAuth".to_owned(), payload)
    }

    #[write]
    fn revoke_deploy_auth(
        &mut self,
        ctx: ServiceContext,
        payload: AddressList,
    ) -> ServiceResponse<()> {
        require_admin!(self.authorization, &ctx);
        sub_cycles!(ctx, payload.addresses.len() as u64 * 10_000);

        for addr in payload.addresses.iter() {
            self.authorization
                .revoke(&addr, authorization::Kind::Deploy);
        }

        Self::emit_event(&ctx, "RevokeAuth".to_owned(), payload)
    }

    #[write]
    fn deploy(
        &mut self,
        ctx: ServiceContext,
        payload: DeployPayload,
    ) -> ServiceResponse<DeployResp> {
        // Check authority list
        if !self
            .authorization
            .granted(&ctx.get_caller(), authorization::Kind::Deploy)
        {
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
        self.sdk
            .borrow_mut()
            .set_value(code_hash.clone(), code.clone());

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
            .set_value(contract_address.clone(), contract.clone());

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

        let intp_params = InterpreterParams::new_for_init(
            contract_address.clone(),
            code,
            init_payload.args.clone().into(),
        );
        let writeable_chain = Rc::new(RefCell::new(WriteableChain::new(
            ctx.clone(),
            init_payload,
            Rc::<_>::clone(&self.sdk),
        )));

        let resp = self.run_interpreter(ctx, contract, writeable_chain, intp_params);
        if resp.is_error() {
            ServiceResponse::from_error(resp.code, resp.error_message)
        } else {
            ServiceResponse::from_succeed(DeployResp {
                address:  contract_address,
                init_ret: resp.succeed_data,
            })
        }
    }

    #[write]
    fn approve_contracts(
        &mut self,
        ctx: ServiceContext,
        payload: AddressList,
    ) -> ServiceResponse<()> {
        require_admin!(self.authorization, &ctx);
        sub_cycles!(ctx, payload.addresses.len() as u64 * 10_000);

        let authorizer = Authorizer::new(ctx.get_caller());
        let auth_kind = authorization::Kind::Contract;

        for address in payload.addresses.clone() {
            if let Err(e) = self.load_contract(&address) {
                return e.into();
            };

            self.authorization
                .grant(address, auth_kind, authorizer.clone());
        }

        Self::emit_event(&ctx, "ApproveContract".to_owned(), payload)
    }

    #[write]
    fn revoke_contracts(
        &mut self,
        ctx: ServiceContext,
        payload: AddressList,
    ) -> ServiceResponse<()> {
        require_admin!(self.authorization, &ctx);
        sub_cycles!(ctx, payload.addresses.len() as u64 * 10_000);

        for address in payload.addresses.iter() {
            if let Err(e) = self.load_contract(&address) {
                return e.into();
            };

            self.authorization
                .revoke(&address, authorization::Kind::Contract);
        }

        Self::emit_event(&ctx, "RevokeContract".to_owned(), payload)
    }

    fn load_contract(&self, address: &Address) -> Result<Contract, ServiceError> {
        let mut contract = self
            .sdk
            .borrow()
            .get_value::<_, Contract>(address)
            .ok_or_else(|| ServiceError::ContractNotFound(address.as_hex()))?;

        let authorizer = self
            .authorization
            .authorizer(address, authorization::Kind::Contract);

        contract.authorizer = authorizer.inner();

        Ok(contract)
    }

    fn load_contract_code(&self, address: &Address) -> Result<(Contract, Bytes), ServiceError> {
        let contract = self.load_contract(address)?;
        let code = self
            .sdk
            .borrow()
            .get_value::<_, Bytes>(&contract.code_hash)
            .ok_or(ServiceError::CodeNotFound)?;

        Ok((contract, code))
    }

    fn run_interpreter(
        &self,
        ctx: ServiceContext,
        contract: Contract,
        chain: Rc<RefCell<dyn ChainInterface>>,
        params: InterpreterParams,
    ) -> ServiceResponse<String> {
        let interpreter = Interpreter::new(ctx.clone(), contract.intp_type, params, chain);

        match interpreter.run() {
            Ok(exit) => {
                sub_cycles!(ctx, exit.cycles_used);

                let decoded_data = String::from_utf8_lossy(exit.data.as_ref()).to_string();

                if exit.code == 0 {
                    ServiceResponse::from_succeed(decoded_data)
                } else {
                    ServiceError::NonZeroExit {
                        code: exit.code,
                        msg:  decoded_data,
                    }
                    .into()
                }
            }
            Err(err) => {
                sub_cycles!(ctx, err.cycles_used);

                match err.cause {
                    Cause::CkbVM(e) => ServiceError::CkbVm(e).into(),
                    Cause::ErrorResponse(resp) if resp.ecall == SYSCODE_ASSERT => {
                        ServiceError::AssertFailed(resp.msg).into()
                    }
                    Cause::ErrorResponse(resp) => ServiceResponse::from_error(resp.code, resp.msg),
                }
            }
        }
    }

    fn emit_event<T: Serialize>(
        ctx: &ServiceContext,
        name: String,
        event: T,
    ) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::Serde(err).into(),
            Ok(json) => {
                ctx.emit_event(name, json);
                ServiceResponse::from_succeed(())
            }
        }
    }
}
