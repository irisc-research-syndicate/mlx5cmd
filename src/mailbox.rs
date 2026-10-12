use pci_driver::{
    pci_struct,
    regions::{
        structured::PciRegisterRw, AsPciSubregion, BackedByPciSubregion, PciRegion, PciSubregion,
    },
};

use crate::error::Result;

pub(crate) const MAILBOX_BLOCK_SIZE: usize = 0x200;
const MAILBOX_STRIDE: u64 = 0x400;
const CTRL_SIGNATURE_START: usize = 0x1c0;

pci_struct! {
    pub struct Mailbox<'a> : 0x240 {
        next_pointer_hi     @ 0x230 : PciRegisterRw<'a, u32>,
        next_pointer_lo     @ 0x234 : PciRegisterRw<'a, u32>,
        block_number        @ 0x238 : PciRegisterRw<'a, u32>,
        token               @ 0x23d : PciRegisterRw<'a, u8>,
        ctrl_signature      @ 0x23e : PciRegisterRw<'a, u8>,
        signature           @ 0x23f : PciRegisterRw<'a, u8>,
    }
}

impl<'a> Mailbox<'a> {
    pub fn set_data(&self, data: &[u8]) -> Result<()> {
        for (i, b) in data.iter().enumerate() {
            self.write_u8(i as u64, *b)?;
        }
        Ok(())
    }

    pub fn set_next(&self, ptr: u64) -> Result<()> {
        self.next_pointer_hi().write(((ptr >> 32) as u32).to_be())?;
        self.next_pointer_lo()
            .write(((ptr & 0xffffffff) as u32).to_be())?;
        Ok(())
    }

    pub fn update_signature(&self) -> Result<()> {
        self.signature().write(0x00)?;
        self.ctrl_signature().write(0x00)?;
        let mut mb_data = vec![0u8; self.len() as usize];
        self.read_bytes(0, &mut mb_data)?;
        let mut ctrl_signature = 0xff_u8;
        for x in mb_data[CTRL_SIGNATURE_START..MAILBOX_BLOCK_SIZE].iter() {
            ctrl_signature ^= x;
        }
        self.ctrl_signature().write(ctrl_signature)?;
        self.read_bytes(0, &mut mb_data)?;
        let mut signature = 0xff_u8;
        for x in mb_data.iter() {
            signature ^= x;
        }
        self.signature().write(signature)?;
        Ok(())
    }
}

pub struct MailboxAllocator<'a> {
    region: PciSubregion<'a>,
    allocation_offset: u64,
}

impl<'a> MailboxAllocator<'a> {
    pub fn new(region: PciSubregion<'a>) -> MailboxAllocator<'a> {
        Self {
            region,
            allocation_offset: 0,
        }
    }

    pub fn allocate_mailbox(&mut self) -> Result<(u64, Mailbox<'a>)> {
        let mailbox_offset = self.allocation_offset;
        self.allocation_offset += MAILBOX_STRIDE;
        Ok((
            mailbox_offset,
            Mailbox::backed_by(
                self.region
                    .subregion(mailbox_offset..mailbox_offset + MAILBOX_STRIDE),
            ),
        ))
    }

    pub fn build_mailbox(&mut self, token: u8, data: &[u8]) -> Result<Vec<Mailbox<'a>>> {
        let mut mb_vec: Vec<Mailbox<'_>> = vec![];

        for (block_number, chunk) in data.chunks(MAILBOX_BLOCK_SIZE).enumerate() {
            let mb = self.allocate_mailbox()?.1;
            if let Some(prev_mb) = mb_vec.last() {
                prev_mb.set_next(mb.as_ptr().unwrap() as u64)?;
            }
            mb.set_next(0_u64)?;
            mb.set_data(chunk)?;
            mb.token().write(token)?;

            mb.block_number().write((block_number as u32).to_be())?;

            mb_vec.push(mb);
        }

        for mb in mb_vec.iter() {
            mb.update_signature()?;
        }

        Ok(mb_vec)
    }
}
