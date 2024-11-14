use deku::ctx::{BitSize, ByteSize, Endian};
use deku::prelude::*;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x02\x00")]
pub struct CreateMKey {
    #[deku(pad_bytes_before = "10", bits = "1")]
    pub pg_access: bool,
    #[deku(bits="1")]
    pub umem_valid: bool,
    #[deku(pad_bits_before = "30", bytes = "64")]
    pub context: MKeyContext,
    #[deku(pad_bytes_before = "16")]
    pub translation_octwords_actual_size: u32,
    #[deku(pad_bytes_before = "172", count="translation_octwords_actual_size")]
    pub translation_entries: Vec<u64>
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian="big")]
pub struct CreateMKeyOutput {
    pub base: BaseOutput,

    #[deku(pad_bits_before="8", bits="24")]
    pub mkey_index: u32

}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian, _ctx_bytes: ByteSize")]
pub struct MKeyContext {
    #[deku(pad_bits_before = "1", bits = "1")]
    pub free: bool,

    #[deku(pad_bits_before = "14", bits = "1")]
    pub umr_en: bool,
    #[deku(bits = "1")]
    pub a: bool,
    #[deku(bits = "1")]
    pub rw: bool,
    #[deku(bits = "1")]
    pub rr: bool,
    #[deku(bits = "1")]
    pub lw: bool,
    #[deku(bits = "1")]
    pub lr: bool,
    #[deku(bits = "2", pad_bits_after = "8")]
    pub access_mode: AccessMode,

    #[deku(bits = "24")]
    pub qpn: u32,
    #[deku(bits = "8")]
    pub mkey: u8,

    #[deku(pad_bytes_before = "4", bits = "1")]
    pub length64: bool,
    #[deku(pad_bits_before = "7", bits = "24")]
    pub pd: u32,

    pub start_addr: u64,
    pub len: u64,

    pub bsf_octword_size: u32,

    #[deku(pad_bytes_before = "16")]
    pub translation_octword_size: u32,

    #[deku(pad_bits_before = "27", bits = "5", pad_bytes_after = "4")]
    pub log_entry_size: u8,
}

impl Command for CreateMKey {
    type Output = CreateMKeyOutput;

    fn size(&self) -> usize {
        0x110 + 16 * self.translation_octwords_actual_size as usize
    }

    fn outlen(&self) -> usize {
        0x10
    }
}

#[derive(Debug, PartialEq, Copy, Clone, DekuRead, DekuWrite)]
#[deku(type = "u8", bits = "2", endian = "ctx_endian", ctx = "ctx_endian: Endian, _ctx_bits: BitSize")]
pub enum AccessMode {
    PA = 0,
    MTT = 1,
    KLMs = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mkeycontext() {

        #[derive(Debug, PartialEq, DekuRead, DekuWrite)]
        #[deku(endian="big")]
        struct MCtxContainer {
            #[deku(bytes = "64")]
            ctx: MKeyContext,
        }

        let mctx = MCtxContainer{
            ctx: MKeyContext {
                free: true,
                umr_en: false,
                a: false,
                rw: true,
                rr: true,
                lw: true,
                lr: true,
                access_mode: AccessMode::MTT,
                qpn: 0xffffff,
                mkey: 0x41,
                length64: false,
                pd: 17,
                start_addr: 0x12345678_9abcdef0,
                len: 0x41424344_45464748,
                bsf_octword_size: 0,
                translation_octword_size: 0x98765432,
                log_entry_size: 1,
            },
        };
        assert_eq!(mctx.to_bytes().unwrap().len(), 64);
        assert_eq!(&mctx.to_bytes().unwrap(), &[
            1 << 6, 0x00, (15 << 2) | (1 << 0), 0x00,
            0xff, 0xff, 0xff, 0x41,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 17,
            0x12, 0x34, 0x56, 0x78,
            0x9a, 0xbc, 0xde, 0xf0,
            0x41, 0x42, 0x43, 0x44,
            0x45, 0x46, 0x47, 0x48,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x98, 0x76, 0x54, 0x32,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00,
        ]);
    }
}