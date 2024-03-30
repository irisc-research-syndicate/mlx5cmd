use deku::ctx::Endian;
use deku::prelude::*;

use crate::impl_command_output;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x08")]
pub struct ManagePages {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "4")]
    pub op_mod: ManagePagesOpMod,

    pub input_num_entries: u32,

    #[deku(count = "input_num_entries")]
    pub items: Vec<u64>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct ManagePagesOutput {
    pub base: BaseOutput,

    #[deku(pad_bytes_after = "4")]
    pub output_num_entries: u32,

    #[deku(count = "output_num_entries")]
    pub items: Vec<u64>,
}

impl_command_output!(ManagePagesOutput);

impl Command for ManagePages {
    type Output = ManagePagesOutput;

    fn size(&self) -> usize {
        0x10 + self.items.len() * 8
    }

    fn outlen(&self) -> usize {
        0x10 + self.items.len() * 8
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum ManagePagesOpMod {
    AllocationFail = 0x0,
    AllocationSuccess = 0x1,
    HCAReturnPages = 0x2,
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_manage_pages() {
        let cmd = ManagePages {
            op_mod: ManagePagesOpMod::AllocationSuccess,
            input_num_entries: 3,
            items: vec![0x12345678, 0x0, u64::MAX],
        };

        let res = cmd.to_bytes().unwrap();
        assert_eq!(res.len(), cmd.size());

        #[rustfmt::skip]
        let cmd_bytes = &[
            0x01, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(res, cmd_bytes);

        #[rustfmt::skip]
        let output: &[u8] = &[
            0xab, 0x00, 0x00, 0x00, 0x12, 0x34, 0x56, 0x78, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x87, 0x65, 0x43, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];

        assert_eq!(output.len(), cmd.outlen());
        assert_eq!(
            ManagePagesOutput::try_from(output).unwrap(),
            ManagePagesOutput {
                base: BaseOutput {
                    status: 0xab,
                    syndrome: 0x12345678,
                },
                output_num_entries: 3,
                items: vec![0x87654321, 0x0, u64::MAX],
            }
        );
    }
}
