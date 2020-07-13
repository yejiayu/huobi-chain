//! Provedis a debug function, let the contract print information to standard
//! output.
use ckb_vm::instructions::Register;

use crate::vm::syscall::common::get_str;
use crate::vm::syscall::convention::SYSCODE_DEBUG;

pub struct SyscallDebug;

impl<Mac: ckb_vm::SupportMachine> ckb_vm::Syscalls<Mac> for SyscallDebug {
    fn initialize(&mut self, _machine: &mut Mac) -> Result<(), ckb_vm::Error> {
        Ok(())
    }

    fn ecall(&mut self, machine: &mut Mac) -> Result<bool, ckb_vm::Error> {
        let code = machine.registers()[ckb_vm::registers::A7].to_u64();
        if code != SYSCODE_DEBUG {
            return Ok(false);
        }

        let ptr = machine.registers()[ckb_vm::registers::A0].to_u64();
        if ptr == 0 {
            return Err(ckb_vm::Error::IO(std::io::ErrorKind::InvalidInput));
        }

        let msg = get_str(machine, ptr)?;
        log::debug!(target: "riscv_debug", "{}", msg);
        Ok(true)
    }
}
