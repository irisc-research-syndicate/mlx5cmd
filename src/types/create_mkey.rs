use deku::ctx::Endian;
use deku::prelude::*;

use super::{BaseOutput, Command};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big", magic = b"\x02\x00")]
pub struct CreateMKey {
    pub pg_access: bool,
    pub mkey: MKey,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub struct MKey {
    free: bool,
    umr_en: bool,
    rw: bool,
    rr: bool,
    lw: bool,
    lr: bool,

    access_mode: AccessMode,
    /// 24 bits
    qpn: u32,
    mkey: u8,
    /// 24 bits
    pd: u32,
    start_addr: u64,
    len: u64,
    translations_oct_word_size: u32,
}

#[derive(Debug, PartialEq, Copy, Clone, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum AccessMode {
    PA = 0,
    MTT = 1,
    KLMs = 2,
}
