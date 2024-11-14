use deku::DekuContainerRead;
use irisc_asm::assemble;

pub mod vfio;

use crate::{commands::{access_register::{AccessRegister, AccessRegisterOpMod}, BaseOutputStatus, Command, CommandErrorStatus, ExecShellcode64}, error::{Error, Result}, registers::Register};

pub trait CmdIf {
    fn exec_command(&self, input: &[u8], outlen: u32) -> Result<Vec<u8>>;

    fn do_command<Cmd: Command + core::fmt::Debug>(&self, cmd: Cmd) -> Result<Cmd::Output> {
        log::debug!("Command: {cmd:x?}");
        let msg = cmd.to_bytes()?;
        let out = self.exec_command(&msg, cmd.outlen() as u32)?;
        let base_output = BaseOutputStatus::from_bytes((&out, 0))?.1;
        if base_output.0.status != CommandErrorStatus::Ok {
            return Err(Error::Command {
                status: base_output.0.status,
                syndrome: base_output.0.syndrome,
            });
        }

        let res = Cmd::Output::from_bytes((&out, 0))?.1;
        log::debug!("Output: {res:x?}");
        Ok(res)
    }

   fn read_register<Reg: Register + core::fmt::Debug>(&self, reg: Reg, argument: u32) -> Result<Reg> {
        log::debug!("Reading register {reg:x?}");
        let resp = self.do_command(AccessRegister {
            op_mod: AccessRegisterOpMod::Read,
            argument,
            register_id: Reg::REGISTER_ID,
            register_data: reg.to_bytes()?,
        })?;
        let reg = Reg::from_bytes((&resp.register_data, 0))?.1;
        log::debug!("Register value: {reg:x?}");
        Ok(reg)
    }

   fn write_register<Reg: Register + core::fmt::Debug>(&self, reg: Reg, argument: u32) -> Result<Reg> {
        log::debug!("Writing register {reg:x?}");
        let resp = self.do_command(AccessRegister {
            op_mod: AccessRegisterOpMod::Write,
            argument,
            register_id: Reg::REGISTER_ID,
            register_data: reg.to_bytes()?,
        })?;
        let reg = Reg::from_bytes((&resp.register_data, 0))?.1;
        log::debug!("Register value after write {reg:x?}");
        Ok(reg)
    }

    fn run_shellcode(&self, shellcode: &str) -> anyhow::Result<[u64;3]> {
        let (code, _labels) = assemble(0, shellcode)?;
        let mut shellcode = [0u8; 0xa0];
        shellcode[..code.len()].copy_from_slice(&code);
        Ok(self.do_command(ExecShellcode64{
            op_mod: 0,
            args: [0, 0, 0],
            shellcode,
        })?.results)
    }
}