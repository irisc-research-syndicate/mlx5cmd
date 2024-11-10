use deku::prelude::*;

use super::{BaseOutput, Command};


#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x08\x02\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")]
pub struct AllocUAR {
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct AllocUAROutput {
    pub base: BaseOutput,

    #[deku(pad_bits_before = "8", bits = "24")]
    pub uar: u32
}

impl Command for AllocUAR {
    type Output = AllocUAROutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x08\x03\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")]
pub struct DeallocUAR {
    #[deku(pad_bits_before = "8", bits = "24")]
    uar: u32
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct DeallocUAROutput {
    pub base: BaseOutput,
}

impl Command for DeallocUAR {
    type Output = DeallocUAROutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}