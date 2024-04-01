pub mod hca;
pub mod issi;
pub mod manage_pages;
pub mod query_adapter;
pub mod query_hca_cap;
pub mod query_pages;
pub mod set_driver_version;

pub use hca::*;
pub use issi::*;
pub use manage_pages::*;
pub use query_adapter::*;
pub use query_hca_cap::*;
pub use query_pages::*;
pub use set_driver_version::*;

use std::fmt::Debug;

use deku::ctx::Endian;
use deku::prelude::*;

pub trait Command: DekuContainerWrite {
    type Output: for<'a> DekuContainerRead<'a> + CommandOutput;

    fn size(&self) -> usize;
    fn outlen(&self) -> usize;
}

pub trait CommandOutput: Debug {
    fn status(&self) -> u8;
    fn syndrome(&self) -> u32;
}

#[macro_export]
macro_rules! impl_command_output {
    ($ty: ty) => {
        impl crate::types::CommandOutput for $ty {
            fn status(&self) -> u8 {
                self.base.status
            }

            fn syndrome(&self) -> u32 {
                self.base.syndrome
            }
        }
    };
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "ctx_endian", ctx = "ctx_endian: Endian")]
pub struct BaseOutput {
    #[deku(pad_bytes_after = "3")]
    pub status: u8,

    pub syndrome: u32,
}
