use pci_driver::{
    pci_struct,
    regions::{structured::PciRegisterRw, PciRegion},
};

use crate::error::Result;

pci_struct! {
    pub struct CQE<'a> : 0x40 {
        cmd_type            @ 0x00 : PciRegisterRw<'a, u8>,
        input_length        @ 0x04 : PciRegisterRw<'a, u32>,
        input_mb_ptr_hi     @ 0x08 : PciRegisterRw<'a, u32>,
        input_mb_ptr_lo     @ 0x0c : PciRegisterRw<'a, u32>,
        cmd_input_inline0   @ 0x10 : PciRegisterRw<'a, u32>,
        cmd_input_inline1   @ 0x14 : PciRegisterRw<'a, u32>,
        cmd_input_inline2   @ 0x18 : PciRegisterRw<'a, u32>,
        cmd_input_inline3   @ 0x1c : PciRegisterRw<'a, u32>,
        cmd_output_inline0  @ 0x20 : PciRegisterRw<'a, u32>,
        cmd_output_inline1  @ 0x24 : PciRegisterRw<'a, u32>,
        cmd_output_inline2  @ 0x28 : PciRegisterRw<'a, u32>,
        cmd_output_inline3  @ 0x2c : PciRegisterRw<'a, u32>,
        output_mb_ptr_hi    @ 0x30 : PciRegisterRw<'a, u32>,
        output_mb_ptr_lo    @ 0x34 : PciRegisterRw<'a, u32>,
        output_length       @ 0x38 : PciRegisterRw<'a, u32>,
        token               @ 0x3c : PciRegisterRw<'a, u8>,
        signature           @ 0x3d : PciRegisterRw<'a, u8>,
        status              @ 0x3f : PciRegisterRw<'a, u8>,
    }
}

impl<'a> CQE<'a> {
    pub fn update_signature(&self) -> Result<()> {
        self.signature().write(0x00)?;
        let mut cmd_data = vec![0u8; self.len() as usize];
        self.read_bytes(0, &mut cmd_data)?;
        let mut signature = 0xffu8;
        for x in cmd_data {
            signature ^= x;
        }
        self.signature().write(signature)?;
        Ok(())
    }

    pub fn set_input_mb(&self, ptr: u64) -> Result<()> {
        self.input_mb_ptr_hi().write(((ptr >> 32) as u32).to_be())?;
        self.input_mb_ptr_lo()
            .write(((ptr & 0xffffffff) as u32).to_be())?;
        Ok(())
    }

    pub fn set_output_mb(&self, ptr: u64) -> Result<()> {
        self.output_mb_ptr_hi()
            .write(((ptr >> 32) as u32).to_be())?;
        self.output_mb_ptr_lo()
            .write(((ptr & 0xffffffff) as u32).to_be())?;
        Ok(())
    }
}
