use deku::ctx::Endian;
use deku::prelude::*;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x08\x05")]
pub struct AccessRegister {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "2")]
    pub op_mod: AccessRegisterOpMod,

    pub register_id: u16,
    pub argument: u32,
    #[deku(bytes_read = "deku::rest.len()")]
    pub register_data: Vec<u8>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum AccessRegisterOpMod {
    Write = 0,
    Read = 1,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct AccessRegisterOutput {
    #[deku(pad_bytes_after = "8")]
    pub base: BaseOutput,

    //    #[deku(bytes_read = "deku::rest.len()")]
    pub register_data: [u8; 128],
}

impl Command for AccessRegister {
    type Output = AccessRegisterOutput;

    fn size(&self) -> usize {
        0x10 + self.register_data.len()
    }

    fn outlen(&self) -> usize {
        0x10 + 128
    }
}



#[cfg(test)]
mod tests {
    use crate::types::CommandErrorStatus;

    use super::*;

    #[ignore]
    #[test]
    fn test_access_registers() {
        let cmd = AccessRegister {
            op_mod: AccessRegisterOpMod::Read,
            register_id: 0x1337,
            argument: 0x12345678,
            register_data: vec![0x12, 0x0, u8::MAX],
        };

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        #[rustfmt::skip]
        assert_eq!(res, &[
            0x08, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x13, 0x37, 0x12, 0x34, 0x56, 0x78,
            0x87, 0x65, 0x43, 0x21, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff
        ]);

        #[rustfmt::skip]
        let output: &[u8] = &[
            0xab, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78,
            0x87, 0x65, 0x43, 0x21, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff
        ];
        assert_eq!(output.len(), cmd.outlen());

        assert_eq!(
            AccessRegisterOutput::try_from(output).unwrap(),
            AccessRegisterOutput {
                base: BaseOutput {
                    status: CommandErrorStatus::UnknownError(0xab),
                    syndrome: 0,
                },
                register_data: [0xff; 128],
            }
        );
    }
}
