use crate::{common, types::ExecPayload, ServiceError};

use protocol::{
    traits::{ServiceResponse, ServiceSDK},
    types::{Address, Hash, ServiceContext},
    Bytes,
};

use std::{cell::RefCell, rc::Rc};

pub trait ChainInterface {
    fn get_storage(&self, key: &Bytes) -> Bytes;

    // Note: Only Throw ServiceError::WriteInReadonlyContext
    fn set_storage(&mut self, key: Bytes, val: Bytes) -> ServiceResponse<()>;

    fn contract_call(
        &mut self,
        address: Address,
        args: Bytes,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)>;

    // Note: We need mut here to update cycles count in ServiceContext
    fn service_read(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)>;

    fn service_write(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)>;
}

#[derive(Debug)]
struct CycleContext {
    inner:           ServiceContext,
    all_cycles_used: u64,
}

impl CycleContext {
    pub fn new(ctx: ServiceContext, all_cycles_used: u64) -> Self {
        CycleContext {
            inner: ctx,
            all_cycles_used,
        }
    }
}

pub struct WriteableChain<SDK> {
    ctx:             ServiceContext,
    payload:         ExecPayload,
    sdk:             Rc<RefCell<SDK>>,
    all_cycles_used: u64,
}

impl<SDK: ServiceSDK + 'static> WriteableChain<SDK> {
    pub fn new(
        ctx: ServiceContext,
        payload: ExecPayload,
        sdk: Rc<RefCell<SDK>>,
    ) -> WriteableChain<SDK> {
        Self {
            ctx,
            payload,
            sdk,
            all_cycles_used: 0,
        }
    }

    fn serve<F: FnMut() -> ServiceResponse<String>>(
        cycle_ctx: &mut CycleContext,
        current_cycle: u64,
        mut f: F,
    ) -> ServiceResponse<(String, u64)> {
        let vm_cycle = current_cycle.wrapping_sub(cycle_ctx.all_cycles_used);
        if vm_cycle != 0 {
            crate::sub_cycles!(cycle_ctx.inner, vm_cycle);
        } else {
            return ServiceError::OutOfCycles.into();
        }

        let resp = f();
        if resp.is_error() {
            return ServiceResponse::from_error(resp.code, resp.error_message);
        }

        cycle_ctx.all_cycles_used = cycle_ctx.inner.get_cycles_used();
        ServiceResponse::from_succeed((resp.succeed_data, cycle_ctx.all_cycles_used))
    }

    fn contract_key(&self, key: &Bytes) -> Hash {
        common::combine_key(self.payload.address.as_bytes().as_ref(), key)
    }
}

impl<SDK> ChainInterface for WriteableChain<SDK>
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

    fn set_storage(&mut self, key: Bytes, val: Bytes) -> ServiceResponse<()> {
        let contract_key = self.contract_key(&key);
        self.sdk.borrow_mut().set_value(contract_key, val);
        ServiceResponse::from_succeed(())
    }

    fn contract_call(
        &mut self,
        address: Address,
        args: Bytes,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        let json_payload = match ExecPayload::new(address, args).json() {
            Ok(p) => p,
            Err(e) => return e.into(),
        };

        let resp = self.service_write("riscv", "exec", &json_payload, current_cycle);
        decode_json_response(resp)
    }

    fn service_read(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        let mut cycle_ctx = CycleContext::new(self.ctx.clone(), self.all_cycles_used);

        let resp = Self::serve(&mut cycle_ctx, current_cycle, || -> _ {
            self.sdk.borrow().read(
                &self.ctx,
                Some(Bytes::from(self.payload.address.as_hex())),
                service,
                method,
                payload,
            )
        });

        self.all_cycles_used = cycle_ctx.all_cycles_used;
        resp
    }

    fn service_write(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        let mut cycle_ctx = CycleContext::new(self.ctx.clone(), self.all_cycles_used);

        let resp = Self::serve(&mut cycle_ctx, current_cycle, || -> _ {
            self.sdk.borrow_mut().write(
                &self.ctx,
                Some(Bytes::from(self.payload.address.as_hex())),
                service,
                method,
                payload,
            )
        });

        self.all_cycles_used = cycle_ctx.all_cycles_used;
        resp
    }
}

pub struct ReadonlyChain<SDK> {
    inner: WriteableChain<SDK>,
}

impl<SDK: ServiceSDK + 'static> ReadonlyChain<SDK> {
    pub fn new(
        ctx: ServiceContext,
        payload: ExecPayload,
        sdk: Rc<RefCell<SDK>>,
    ) -> ReadonlyChain<SDK> {
        Self {
            inner: WriteableChain::new(ctx, payload, sdk),
        }
    }
}

impl<SDK: ServiceSDK + 'static> ChainInterface for ReadonlyChain<SDK> {
    fn get_storage(&self, key: &Bytes) -> Bytes {
        self.inner.get_storage(key)
    }

    fn set_storage(&mut self, _key: Bytes, _val: Bytes) -> ServiceResponse<()> {
        ServiceError::WriteInReadonlyContext.into()
    }

    fn contract_call(
        &mut self,
        address: Address,
        args: Bytes,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        let json_payload = match ExecPayload::new(address, args).json() {
            Ok(p) => p,
            Err(e) => return e.into(),
        };

        let resp = self.service_read("riscv", "call", &json_payload, current_cycle);
        decode_json_response(resp)
    }

    fn service_read(
        &mut self,
        service: &str,
        method: &str,
        payload: &str,
        current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        self.inner
            .service_read(service, method, payload, current_cycle)
    }

    fn service_write(
        &mut self,
        _service: &str,
        _method: &str,
        _payload: &str,
        _current_cycle: u64,
    ) -> ServiceResponse<(String, u64)> {
        ServiceError::WriteInReadonlyContext.into()
    }
}

fn decode_json_response(resp: ServiceResponse<(String, u64)>) -> ServiceResponse<(String, u64)> {
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
