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

pub trait Register: DekuContainerWrite + for<'a> DekuContainerRead<'a> {
    const REGISTER_ID: u16;
    fn size(&self) -> usize;
}

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
pub struct MtrcConf {
    #[deku(pad_bits_before = "28", bits = "4")]
    trace_mode: u8,

    #[deku(pad_bits_before = "24", bits = "8")]
    log_trace_buffer_size: u8,

    trace_mkey: u32,
}

impl Register for MtrcConf {
    const REGISTER_ID: u16 = 0x9041;
    fn size(&self) -> usize {
        4 + 4 + 4
    }
}

#[derive(Debug, Default, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MtrcCtrl {
    #[deku(bits = "2")]
    trace_status: u8,

    #[deku(pad_bits_before = "2", bits = "1")]
    arm_event: bool,

    #[deku(pad_bits_before = "11", bits = "16")]
    modify_field_select: u16,

    #[deku(pad_bytes_before = "4")]
    timestamp_hi: u32,
    timestamp_lo: u32,
}

impl Register for MtrcCtrl {
    const REGISTER_ID: u16 = 0x9043;
    fn size(&self) -> usize {
        4 + 4 + 4
    }
}

#[cfg(test)]
mod tests {
    use crate::types::CommandErrorStatus;

    use super::*;

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
