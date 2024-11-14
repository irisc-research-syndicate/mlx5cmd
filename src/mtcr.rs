use pci_driver::{config::{caps::Capability, PciConfig}, device::PciDevice, regions::{AsPciSubregion, PciRegion, PciSubregion}};
use crate::{error::{Error, Result}, cmdif::vfio::Mlx5CmdIf};
use log::{trace, debug};

pub const AS_ICMD_EXT: u16 = 0x1;
pub const AS_CR_SPACE: u16 = 0x2;
pub const AS_ICMD: u16 = 0x3;
pub const AS_NODNIC_INIT_SEG: u16 = 0x4;
pub const AS_EXPANSION_ROM: u16 = 0x5;

pub const VCR_CMD_SIZE_ADDR: u32 = 0x1000;
pub const VCR_CMD_ADDR: u32 = 0x100000;
pub const VCR_CTRL_ADDR: u32 = 0x0;
pub const VCR_EXMB_ADDR: u32 = 0x8;

fn find_capability<'a>(config: &PciConfig<'a>, cap_id: u8) -> Result<PciSubregion<'a>> {
    for cap in config.capabilities()? {
        if cap.header().capability_id().read()? == cap_id {
            return Ok(config.subregion(cap.as_subregion().offset_in_underlying_region()..0x100));
        }
    }
    Err(Error::CapabilityNotFound)
}

pub struct MTCR<'a> {
    pub cap_region: PciSubregion<'a>
}

pub const PCI_CONTROL: u64 = 0x04;
pub const PCI_COUNTER: u64 = 0x08;
pub const PCI_SEMAPHORE: u64 = 0x0c;
pub const PCI_ADDRESS: u64 = 0x10;
pub const PCI_DATA: u64 = 0x14;

impl<'a> MTCR<'a> {

    pub fn from_subregion(cap_region: PciSubregion<'a>) -> Self {
        Self {
            cap_region
        }
    }
    
    pub fn change_address_space(&self, address_space: u16) -> Result<()> {
        trace!("MTCR: changing address to {}", address_space);
        let ctrl = self.cap_region.read_le_u32(PCI_CONTROL)?;
        self.cap_region.write_le_u32(PCI_CONTROL, (ctrl & 0xffff0000) | (address_space as u32))?;
        let ctrl = self.cap_region.read_le_u32(PCI_CONTROL)?;
        if (ctrl >> 29) & 0x7 == 0 {
            return Err(Error::InvalidAddressSpace);
        }
        Ok(())
    }

    pub fn is_address_space_supported(&self, address_space: u16) -> Result<bool> {
        match self.change_address_space(address_space) {
            Ok(_) => Ok(true),
            Err(Error::InvalidAddressSpace) => Ok(false),
            Err(err) => Err(err)
        }
    }

    pub fn write_dword(&self, address: u32, data: u32) -> Result<()> {
        trace!("Writing dword {:x} to {:x}", data, address);
        let address = address & 0x3fffffff | (1 << 31);
        self.cap_region.write_le_u32(PCI_DATA, data)?;
        self.cap_region.write_le_u32(PCI_ADDRESS, address)?;
        while (self.cap_region.read_le_u32(PCI_ADDRESS)? >> 31) & 1 != 0 {
            trace!("Waiting for dword write");
            
        }
        Ok(())
    }

    pub fn read_dword(&self, address: u32) -> Result<u32> {
        trace!("Reading dword from {:x}", address);
        let address = address & 0x3fffffff | (0 << 31);
        self.cap_region.write_le_u32(PCI_ADDRESS, address)?;
        while (self.cap_region.read_le_u32(PCI_ADDRESS)? >> 31) & 1 != 1 {
            trace!("Waiting for dword read");
            
        }
        let data = self.cap_region.read_le_u32(PCI_DATA)?;
        debug!("Read dword {:x}", data);
        Ok(data)
    }

    pub fn open_from_cmdif(cmdif: &'a Mlx5CmdIf) -> Result<Self> {
        Ok(MTCR { cap_region: find_capability(&cmdif.pci_device.config(), 0x09)? })
    }
}

pub struct ICmd<'a> {
    pub mtcr: MTCR<'a>,
    pub max_cmd_size: u32
}

impl<'a> ICmd<'a> {
    pub fn from_mtcr(mtcr: MTCR<'a>) -> Result<Self> {
        mtcr.change_address_space(AS_ICMD)?;
        Ok(Self {
            max_cmd_size: mtcr.read_dword(VCR_CMD_SIZE_ADDR)?,
            mtcr: mtcr,
        })
    }

    pub fn send_icmd_command(&self, opcode: u16, command: Vec<u32>, outlen: usize) -> Result<(u8, Vec<u32>)> {
        self.icmd_set_opcode(opcode)?;
        self.write_command(command)?;
        self.ctrl_modify(|ctrl| ctrl | 0x00000001)?;
        while self.ctrl_read()? & 0x00000001 == 0x00000001 {
            trace!("Waiting for icmd response");
        }
        let status = (self.ctrl_read()? >> 8) as u8;
        Ok((status, self.read_command(outlen)?))
    }

    pub fn write_command(&self, command: Vec<u32>) -> Result<()> {
        for (offset, dword) in command.into_iter().enumerate() {
            self.mtcr.write_dword(VCR_CMD_ADDR + 4*(offset as u32), dword)?
        }
        Ok(())
    }

    pub fn read_command(&self, length: usize) -> Result<Vec<u32>> {
        (0..length).map(|offset|
            self.mtcr.read_dword(VCR_CMD_ADDR + 4*(offset as u32))
        ).collect::<Result<Vec<u32>>>()
    }

    pub fn ctrl_read(&self) -> Result<u32> {
        self.mtcr.read_dword(VCR_CTRL_ADDR)
    }
    
    pub fn ctrl_write(&self, ctrl: u32) -> Result<u32> {
        self.mtcr.write_dword(VCR_CTRL_ADDR, ctrl)?;
        Ok(ctrl)
    }

    pub fn ctrl_modify(&self, f: impl Fn(u32) -> u32) -> Result<u32> {
        self.ctrl_write(f(self.ctrl_read()?))
    }

    pub fn icmd_set_opcode(&self, opcode: u16) -> Result<u32> {
        self.ctrl_modify(|ctrl| (ctrl & 0x0000ffff) | ((opcode as u32) << 16))
    }
}

impl<'a> ICmd<'a> {
    pub fn set_itrace(&self, mask: u32, level: u8, delay: u16) -> Result<()> {
        self.send_icmd_command(0xf003, vec![mask, ((delay as u32) << 16) | (level as u32)], 8)?;
        Ok(())
    }

    pub fn check_badc0ffe_unlocked(&self) -> Result<bool> {
        Ok(self.send_icmd_command(0xf00b, vec![], 0)?.0 == 0)
    }
}