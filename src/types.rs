pub mod access_register;
pub mod exec_shellcode;
pub mod hca;
pub mod issi;
pub mod manage_pages;
pub mod query_adapter;
pub mod query_hca_cap;
pub mod query_pages;
pub mod set_driver_version;

pub use exec_shellcode::*;
pub use hca::*;
pub use issi::*;
pub use manage_pages::*;
pub use query_adapter::*;
pub use query_hca_cap::*;
pub use query_pages::*;
pub use set_driver_version::*;
use thiserror::Error;

use std::fmt::Debug;

use deku::ctx::Endian;
use deku::prelude::*;

pub trait Command: DekuContainerWrite {
    type Output: for<'a> DekuContainerRead<'a> + Debug;

    fn size(&self) -> usize;
    fn outlen(&self) -> usize;
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub struct BaseOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: CommandErrorStatus,

    pub syndrome: u32,
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct BaseOutputStatus(pub BaseOutput);

#[derive(Error, Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(type = "u8", endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub enum CommandErrorStatus {
    #[error["Ok"]]
    #[deku(id = "0x00")]
    Ok,

    #[error["Internal error"]]
    #[deku(id = "0x01")]
    InternalError,

    #[error["Bad operation"]]
    #[deku(id = "0x02")]
    BadOperation,

    #[error["Bad parameter"]]
    #[deku(id = "0x03")]
    BadParameter,

    #[error["Bad system State"]]
    #[deku(id = "0x04")]
    BadSystemState,

    #[error["Bad resource"]]
    #[deku(id = "0x05")]
    BadResource,

    #[error["Resource busy"]]
    #[deku(id = "0x06")]
    ResourceBusy,

    // 0x07 ???
    #[error["Exceeded limit"]]
    #[deku(id = "0x08")]
    ExceededLimit,

    #[error["Bad resource state"]]
    #[deku(id = "0x09")]
    BadResourceState,

    #[error["Bad index"]]
    #[deku(id = "0x0a")]
    BadIndex,

    #[error["No resources"]]
    #[deku(id = "0x0f")]
    NoResources,

    #[error["Bad input length"]]
    #[deku(id = "0x50")]
    BadInputLen,

    #[error["Bad output length"]]
    #[deku(id = "0x51")]
    BadOutputLen,

    #[error["Unkonwn error status code: {0}"]]
    #[deku(id_pat = "_")]
    UnknownError(u8),
}
