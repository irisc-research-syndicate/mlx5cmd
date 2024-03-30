use deku::ctx::Endian;
use deku::prelude::*;

use crate::impl_command_output;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x0d")]
pub struct SetDriverVersion {
    #[deku(pad_bytes_before = "14")]
    pub driver_version: [u8; 64],
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct SetDriverVersionOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}

impl_command_output!(SetDriverVersionOutput);

impl Command for SetDriverVersion {
    type Output = SetDriverVersionOutput;

    fn size(&self) -> usize {
        0x50
    }

    fn outlen(&self) -> usize {
        0x10
    }
}
#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn test_set_driver_version() {
        let mut cmd = SetDriverVersion {
            driver_version: [0; 64],
        };

        cmd.driver_version
            .as_mut_slice()
            .write(b"test-version\0")
            .unwrap();

        let res = cmd.to_bytes().unwrap();
        assert_eq!(res.len(), cmd.size());

        assert_eq!(
            res,
            &[
                1, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116, 45, 118, 101,
                114, 115, 105, 111, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
    }
}
