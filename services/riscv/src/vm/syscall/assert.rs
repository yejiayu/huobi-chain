//! Provedis a debug function, let the contract print information to standard
//! output.
use std::{cell::RefCell, rc::Rc};

use ckb_vm::instructions::Register;

use crate::vm::interpreter::ErrorResponse;
use crate::vm::syscall::common::get_str;
use crate::vm::syscall::convention::SYSCODE_ASSERT;

pub struct SyscallAssert {
    error_response: Rc<RefCell<Option<ErrorResponse>>>,
}

impl SyscallAssert {
    pub fn new(error_response: Rc<RefCell<Option<ErrorResponse>>>) -> SyscallAssert {
        Self { error_response }
    }
}

impl<Mac: ckb_vm::SupportMachine> ckb_vm::Syscalls<Mac> for SyscallAssert {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), ckb_vm::Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, ckb_vm::Error> {
        let code = &machine.registers()[ckb_vm::registers::A7];
        if code.to_u64() != SYSCODE_ASSERT {
            return Ok(false);
        }

        let assertion = machine.registers()[ckb_vm::registers::A0].to_u64();
        if assertion == 0 {
            let msg_ptr = machine.registers()[ckb_vm::registers::A1].to_u64();
            let msg = if msg_ptr != 0 {
                let msg = get_str(machine, msg_ptr)?;
                log::debug!(target: "riscv_debug", "{}", msg);
                msg
            } else {
                String::new()
            };

            *self.error_response.borrow_mut() = Some(ErrorResponse::new(SYSCODE_ASSERT, 0, msg));
            Err(ckb_vm::Error::InvalidEcall(SYSCODE_ASSERT))
        } else {
            Ok(true)
        }
    }
}
