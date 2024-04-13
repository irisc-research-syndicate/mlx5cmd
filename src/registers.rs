pub mod mtrc;

use deku::{DekuContainerRead, DekuContainerWrite};


pub trait Register: DekuContainerWrite + for<'a> DekuContainerRead<'a> {
    const REGISTER_ID: u16;
    fn size(&self) -> usize;
}