use deku::prelude::*;

use crate::impl_command_output;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x00")]
pub struct QueryHCACap {
    #[deku(pad_bytes_before = "4", pad_bytes_after = "8")]
    pub op_mod: u16,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryHCACapOutput {
    #[deku(pad_bytes_after = "8")]
    pub base: BaseOutput,

    pub capabilities: [u8; 0x1000],
}

impl_command_output!(QueryHCACapOutput);

impl Command for QueryHCACap {
    type Output = QueryHCACapOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x1010
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_hca_cap() {
        let cmd = QueryHCACap { op_mod: 0x01 };

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(
            res,
            &[0x01, 0x00, 0, 0, 0, 0, 0, 0x01, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }
}
