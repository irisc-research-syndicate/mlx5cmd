use deku::prelude::*;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x02\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct InitHCA(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct InitHCAOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}

impl Command for InitHCA {
    type Output = InitHCAOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x04\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct EnableHCA(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct EnableHCAOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}

impl Command for EnableHCA {
    type Output = EnableHCAOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x01\x05\0\0\0\0\0\0\0\0\0\0\0\0\0\0")]
pub struct DisableHCA(pub ());

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct DisableHCAOutput {
    #[deku(pad_bytes_after = "4")]
    pub base: BaseOutput,
}

impl Command for DisableHCA {
    type Output = DisableHCAOutput;

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
    fn test_init_hca() {
        let cmd = InitHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_enable_hca() {
        let cmd = EnableHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_disable_hca() {
        let cmd = DisableHCA(());

        let res = cmd.to_bytes().unwrap();

        assert_eq!(res.len(), cmd.size());
        assert_eq!(res, &[0x01, 0x05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }
}
