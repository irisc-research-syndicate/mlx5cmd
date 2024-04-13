use deku::prelude::*;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")]
pub struct AllocPD {

}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct AllocPDOutput {
    pub base: BaseOutput,
    #[deku(pad_bits_before = "8", bits = "24")]
    pub pd: u32
}

impl Command for AllocPD {
    type Output = AllocPDOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}


