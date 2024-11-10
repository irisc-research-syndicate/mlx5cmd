use deku::ctx::{ByteSize, Endian};
use deku::prelude::*;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian, _ctx_bytes: ByteSize")]
pub struct EQContext {
    #[deku(bits = "4")]
    pub status: u8,
    #[deku(bits = "1", pad_bits_before = "9")]
    pub ec: bool,
    #[deku(bits = "1")]
    pub oi: bool,
    #[deku(bits = "4", pad_bits_before = "5", pad_bits_after = "8")]
    pub st:u8,

    // page_offset must be 0, skipping

    #[deku(pad_bytes_before="8", pad_bits_before="3", bits="5")]
    pub log_eq_size: u8,
    #[deku(bits="24")]
    pub uar_page: u32,

    #[deku(pad_bytes_before="7", bits="8")]
    pub intr: u8,

    #[deku(pad_bits_before="3", bits="5", pad_bits_after="24")]
    pub log_page_size: u8,

    #[deku(pad_bytes_before="8", pad_bits_before="8", bits="24")]
    pub consumer_counter: u32,

    #[deku(pad_bits_before="8", bits="24", pad_bytes_after="16")]
    pub producer_counter: u32,
}

#[derive(Debug, PartialEq, DekuWrite)]
#[deku(endian = "big", magic = b"\x03\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00")]
pub struct CreateEQ {
    #[deku(bytes="64")]
    pub ctx: EQContext,

    #[deku(pad_bytes_before = "12")]
    pub event_bitmask: u64,

    #[deku(pad_bytes_before = "176")]
    pub pas: Vec<u64>,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct CreateEQOutput {
    pub base: BaseOutput,

    #[deku(pad_bits_before="24", pad_bytes_after="4")]
    pub eq: u8,
}

impl Command for CreateEQ {
    type Output = CreateEQOutput;

    fn size(&self) -> usize {
        0x110 + 8 * self.pas.len()
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuWrite)]
#[deku(endian = "big", magic = b"\x03\x02\x00\x00\x00\x00\x00\x00")]
pub struct DestroyEQ {
    #[deku(pad_bits_before="24", pad_bytes_after="4")]
    pub eq: u8,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct DestroyEQOutput {
    pub base: BaseOutput,
}

impl Command for DestroyEQ {
    type Output = DestroyEQOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, DekuWrite)]
#[deku(endian = "big", magic = b"\x03\x03\x00\x00\x00\x00\x00\x00")]
pub struct QueryEQ {
    #[deku(pad_bits_before="24", pad_bytes_after="4")]
    pub eq: u8,
}


#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct QueryEQOutput {
    pub base: BaseOutput,

    #[deku(pad_bytes_before="8", bytes="64")]
    pub ctx: EQContext,

    #[deku(pad_bytes_before="12")]
    pub event_mask: u64,
}

impl Command for QueryEQ {
    type Output = QueryEQOutput;

    fn size(&self) -> usize {
        0x10
    }

    fn outlen(&self) -> usize {
        0x110
    }
}

#[derive(Debug, PartialEq, DekuWrite)]
#[deku(endian = "big", magic = b"\x03\x04\x00\x00\x00\x00\x00\x00")]
pub struct GenEQE {
    #[deku(pad_bits_before="24", pad_bytes_after="4")]
    pub eq: u8,

    pub eqe: [u8; 0x40]
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct GenEQEOutput {
    pub base: BaseOutput,
}

impl Command for GenEQE {
    type Output = GenEQEOutput;

    fn size(&self) -> usize {
        0x50
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    fn test_eqcontext() {

        #[derive(Debug, PartialEq, DekuRead, DekuWrite)]
        #[deku(endian="big")]
        struct EQContextContainer {
            #[deku(bytes = "64")]
            ctx: EQContext,
        }

        let eqctx = EQContextContainer {
            ctx: EQContext {
                status: 0x7,
                ec: true,
                oi: false,
                st: 0x3,
                log_eq_size: 4,
                uar_page: 0x123456,
                intr: 0x55,
                log_page_size: 5,
                consumer_counter: 0xaa55aa,
                producer_counter: 0xbadbad,
            }
        };
//        assert_eq!(eqctx.to_bytes().unwrap().len(), 64);

        assert_eq!(eqctx.to_bytes().unwrap(), vec![
            (7 << 4), (1 << 2)|(0 << 1), 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x04, 0x12, 0x34, 0x56,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x55,
            0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0xaa, 0x55, 0xaa,
            0x00, 0xba, 0xdb, 0xad,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ])
    }

    #[test]
    fn test_create_eq() {
        let cmd = CreateEQ {
            ctx: EQContext {
                status: 0x7,
                ec: true,
                oi: false,
                st: 0x3,
                log_eq_size: 4,
                uar_page: 0x123456,
                intr: 0x55,
                log_page_size: 5,
                consumer_counter: 0xaa55aa,
                producer_counter: 0xbadbad,
            },
            event_bitmask: 0x12345678aa55aa55,
            pas: vec![0x55aa55aa_55aa55aa, 0x13371337_13371337],
        };

        let bytes = cmd.to_bytes().unwrap();

        assert_eq!(&bytes[0x00..0x10], vec![
            0x03, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);

        assert_eq!(&bytes[0x10..0x50], vec![
            (7 << 4), (1 << 2)|(0 << 1), 0x03, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x04, 0x12, 0x34, 0x56,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x55,
            0x05, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0xaa, 0x55, 0xaa,
            0x00, 0xba, 0xdb, 0xad,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ]);

        assert_eq!(&bytes[0x50..0x60], vec![
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x12, 0x34, 0x56, 0x78,
            0xaa, 0x55, 0xaa, 0x55,
        ]);

        assert_eq!(bytes[0x60..0x110], vec![0; 0xb0]);

        assert_eq!(bytes[0x110..], vec![
            0x55, 0xaa, 0x55, 0xaa,
            0x55, 0xaa, 0x55, 0xaa,
            0x13, 0x37, 0x13, 0x37,
            0x13, 0x37, 0x13, 0x37,
        ]);
    }

    #[test]
    fn test_destroy_eq() {
        let cmd = DestroyEQ {
            eq: 0x47
        };
        let bytes = cmd.to_bytes().unwrap();
        assert_eq!(bytes, vec![
            0x03, 0x02, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x47,
            0x00, 0x00, 0x00, 0x00,
        ]);
    }
}