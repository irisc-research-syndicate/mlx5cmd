use deku::ctx::Endian;
use deku::prelude::*;

use crate::impl_command_output;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x07")]
pub struct QueryPages {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "8")]
    pub op_mod: QueryPagesOpMod,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum QueryPagesOpMod {
    BootPages = 0x1,
    InitPages = 0x2,
    RegularPages = 0x3,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryPagesOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
    pub num_pages: i32,
}

impl_command_output!(QueryPagesOutput);

impl Command for QueryPages {
    type Output = QueryPagesOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_pages() {
        let cmd = QueryPages {
            op_mod: QueryPagesOpMod::BootPages,
        };

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(
            res,
            &[0x01, 0x07, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0]
        );

        let output: &[u8] = &[
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x20, 0x22,
        ];

        assert_eq!(output.len(), cmd.outlen());

        assert_eq!(
            QueryPagesOutput::try_from(output).unwrap(),
            QueryPagesOutput {
                base: BaseOutput {
                    status: 0,
                    syndrome: 0
                },
                num_pages: 8226,
            }
        );
    }
}
