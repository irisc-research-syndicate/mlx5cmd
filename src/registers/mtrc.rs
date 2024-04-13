use deku::{ctx::Endian, DekuRead, DekuWrite};
use deku::prelude::*;

use super::Register;


#[derive(Debug, Default, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub struct StringDbParam {
    pub address: u32,
    pub size: u32,
}

#[derive(Debug, Default, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MtrcCapReg {
    #[deku(bits = "1")]
    pub trace_owner: bool,

    #[deku(bits = "1")]
    pub trace_to_memory: bool,

    #[deku(pad_bits_before = "4", bits = "2")]
    pub trc_ver: u8,

    #[deku(pad_bits_before = "20", bits = "4")]
    pub num_string_db: u8,

    pub first_string_trace: u8,
    pub num_string_trace: u8,
    #[deku(pad_bytes_before = "5")]
    pub log_max_trace_buffer_size: u8,

    #[deku(pad_bytes_before = "4")]
    pub string_db_param: [StringDbParam; 8],
}

impl Register for MtrcCapReg {
    const REGISTER_ID: u16 = 0x9040;
    fn size(&self) -> usize {
        80
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MtrcStdbReg {
    #[deku(bits = "4")]
    pub index: u8,

    #[deku(pad_bits_before = "4", bits = "24")]
    pub size: u32,

    pub offset: u32,

    pub data: [u8; 64],
}

impl Default for MtrcStdbReg {
    fn default() -> Self {
        Self {
            index: 0,
            size: 0,
            offset: 0,
            data: [0u8; 64],
        }
    }
}

impl Register for MtrcStdbReg {
    const REGISTER_ID: u16 = 0x9042;
    fn size(&self) -> usize {
        4 + 4 + 64
    }
}

#[derive(Debug, Default, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MtrcConfReg {
    #[deku(pad_bits_before = "28", bits = "4")]
    pub trace_mode: u8,

    #[deku(pad_bits_before = "24", bits = "8")]
    pub log_trace_buffer_size: u8,

    pub trace_mkey: u32,
}

impl Register for MtrcConfReg {
    const REGISTER_ID: u16 = 0x9041;
    fn size(&self) -> usize {
        4 + 4 + 4
    }
}

#[derive(Debug, Default, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MtrcCtrlReg {
    #[deku(bits = "2")]
    pub trace_status: u8,

    #[deku(pad_bits_before = "2", bits = "1")]
    pub arm_event: bool,

    #[deku(pad_bits_before = "11", bits = "16")]
    pub modify_field_select: u16,

    #[deku(pad_bytes_before = "4")]
    pub timestamp_hi: u32,
    pub timestamp_lo: u32,
}

impl Register for MtrcCtrlReg {
    const REGISTER_ID: u16 = 0x9043;
    fn size(&self) -> usize {
        4 + 4 + 4
    }
}