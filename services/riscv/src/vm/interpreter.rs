use std::cell::RefCell;
use std::rc::Rc;

use ckb_vm::machine::asm::{AsmCoreMachine, AsmMachine};
use ckb_vm::{DefaultMachineBuilder, SupportMachine};
use derive_more::Constructor;

use protocol::{
    types::{Address, ServiceContext},
    Bytes,
};

use crate::types::InterpreterType;
use crate::vm;
use crate::vm::ChainInterface;

#[derive(Debug, Constructor)]
pub struct Exit {
    pub code:        i8,
    pub data:        Bytes,
    pub cycles_used: u64,
}

#[derive(Debug, Constructor, Clone)]
pub struct ErrorResponse {
    pub ecall: u64,
    pub code:  u64,
    pub msg:   String,
}

#[derive(Debug)]
pub enum Cause {
    CkbVM(ckb_vm::Error),
    ErrorResponse(ErrorResponse),
}

#[derive(Debug, Constructor)]
pub struct Error {
    pub cause:       Cause,
    pub cycles_used: u64,
}

#[derive(Clone, Debug)]
pub struct InterpreterParams {
    pub address: Address,
    pub code:    Bytes,
    pub args:    Bytes,
    pub is_init: bool,
}

impl InterpreterParams {
    pub fn new(address: Address, code: Bytes, args: Bytes) -> InterpreterParams {
        Self {
            address,
            code,
            args,
            is_init: false,
        }
    }

    pub fn new_for_init(address: Address, code: Bytes, args: Bytes) -> InterpreterParams {
        let mut params = Self::new(address, code, args);
        params.is_init = true;
        params
    }
}

pub struct Interpreter {
    pub context: ServiceContext,
    pub r#type:  InterpreterType,
    pub params:  InterpreterParams,
    pub chain:   Rc<RefCell<dyn ChainInterface>>,
}

impl Interpreter {
    pub fn new(
        context: ServiceContext,
        r#type: InterpreterType,
        params: InterpreterParams,
        chain: Rc<RefCell<dyn ChainInterface>>,
    ) -> Self {
        Self {
            context,
            r#type,
            params,
            chain,
        }
    }

    pub fn run(&self) -> Result<Exit, Error> {
        let (code, init_payload) = match self.r#type {
            InterpreterType::Binary => (self.params.code.clone(), None),
        };

        let mut args: Vec<Bytes> = vec!["main".into()];
        if let Some(payload) = init_payload {
            args.push(payload);
        }

        let ret_data = Rc::new(RefCell::new(Vec::new()));
        let err_resp = Rc::new(RefCell::new(None));
        let cycles_limit = self.context.get_cycles_limit();

        let core_machine = AsmCoreMachine::new_with_max_cycles(cycles_limit);
        let default_machine = DefaultMachineBuilder::<Box<AsmCoreMachine>>::new(core_machine)
            .instruction_cycle_func(Box::new(vm::cost_model::instruction_cycles))
            .syscall(Box::new(vm::SyscallDebug))
            .syscall(Box::new(vm::SyscallAssert::new(Rc::<_>::clone(&err_resp))))
            .syscall(Box::new(vm::SyscallEnvironment::new(
                self.context.clone(),
                self.params.clone(),
            )))
            .syscall(Box::new(vm::SyscallIO::new(
                self.params.args.to_vec(),
                Rc::<_>::clone(&ret_data),
            )))
            .syscall(Box::new(vm::SyscallChainInterface::new(
                Rc::<_>::clone(&self.chain),
                Rc::<_>::clone(&err_resp),
            )))
            .build();

        let mut machine = AsmMachine::new(default_machine, None);
        if let Err(e) = machine.load_program(&code, &args[..]) {
            return Err(Error::new(Cause::CkbVM(e), 0));
        }

        let maybe_exit_code = machine.run();
        let cycles_used = machine.machine.cycles();

        match maybe_exit_code {
            Ok(exit_code) => Ok(Exit {
                code: exit_code,
                data: Bytes::from(ret_data.borrow().to_vec()),
                cycles_used,
            }),
            Err(ckb_vm::Error::InvalidEcall(ecall)) => {
                let cause = match (*err_resp.borrow()).clone() {
                    Some(resp) => Cause::ErrorResponse(resp),
                    None => Cause::CkbVM(ckb_vm::Error::InvalidEcall(ecall)),
                };
                Err(Error { cause, cycles_used })
            }
            Err(e) => Err(Error {
                cause: Cause::CkbVM(e),
                cycles_used,
            }),
        }
    }
}
