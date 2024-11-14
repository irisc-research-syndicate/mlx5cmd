use deku::{DekuRead, DekuWrite};
use deku::prelude::*;

use super::Register;

#[derive(Debug, Default, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MFPA {
    #[deku(bits = "1")]
    pub add_cap_32b: bool,
    #[deku(pad_bits_before = "22", bits = "1")]
    pub p: bool,
    #[deku(pad_bits_before = "2", bits = "2", pad_bits_after = "4")]
    pub fs: usize,

    pub boot_address: u32,

    #[deku(pad_bytes_before = "8", bits = "1")]
    pub wip: bool,
    #[deku(bits = "1")]
    pub bulk_32kb_erase_en: bool,
    #[deku(bits = "1")]
    pub bulk_64kb_erase_en: bool,
    #[deku(pad_bits_before = "11", bits = "1")]
    pub sector_wrp_en: bool,
    #[deku(bits = "1")]
    pub sub_sector_wrp_en: bool,
    #[deku(pad_bits_before = "12", bits = "4")]
    pub flash_num: usize,

    #[deku(pad_bytes_before="1", bits=24)]
    pub jedec_id: u32,
    #[deku(bits = "8")]
    pub block_size: usize,
    #[deku(bits = "8")]
    pub block_alignment: usize,
    #[deku(pad_bits_before = "6", bits = "10")]
    pub sector_size: usize,

    pub capability_mask: u32
}

impl Register for MFPA {
    const REGISTER_ID: u16 = 0x9010;

    fn size(&self) -> usize {
        0x20
    }
}


#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MFBA {
    #[deku(bits = "1")]
    pub add_cap_32b: bool,
    #[deku(pad_bits_before = "22", bits = "1")]
    pub p: bool,
    #[deku(pad_bits_before = "2", bits = "2", pad_bits_after = "4")]
    pub fs: usize,

    #[deku(pad_bits_before = "23", bits = "9")]
    pub size: usize,

    #[deku(bits="32")]
    pub address: usize,

    pub data: [u8; 0x40]
}

impl Default for MFBA {
    fn default() -> Self {
        Self { add_cap_32b: false, p: false, fs: 0, size: 0, address: 0, data: [0u8; 0x40] }
    }
}

impl Register for MFBA {
    const REGISTER_ID: u16 = 0x9011;

    fn size(&self) -> usize {
        0x10c
    }
}