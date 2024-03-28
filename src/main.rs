#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::ptr::null_mut;

use pci_driver::regions::{AsPciSubregion, BackedByPciSubregion, PciMemoryRegion, Permissions};
use pci_driver::{device::PciDevice, regions::{structured::{PciRegisterRo, PciRegisterRw}, PciRegion}};
use dbg_hex::dbg_hex;

mod error;

use error::{Error, Result};

use pci_driver::pci_struct;

pci_struct! {
    pub struct InitSegment<'a> : 0x1000 {
        fw_rev_major        @ 0x0000 : PciRegisterRo<'a, u16>,
        fw_rev_minor        @ 0x0002 : PciRegisterRo<'a, u16>,
        fw_rev_subminor     @ 0x0004 : PciRegisterRo<'a, u16>,
        cmd_interface_rev   @ 0x0006 : PciRegisterRo<'a, u16>,
        cmdq_phy_addr_hi    @ 0x0010 : PciRegisterRw<'a, u32>,
        cmdq_phy_addr_lo    @ 0x0014 : PciRegisterRw<'a, u32>,
        cmdq_doorbell       @ 0x0018 : PciRegisterRw<'a, u32>,
        initializing        @ 0x01fc : PciRegisterRo<'a, u32>,
        internal_timer_hi   @ 0x1000 : PciRegisterRo<'a, u32>,
        internal_timer_lo   @ 0x1004 : PciRegisterRo<'a, u32>,
        clear_interrupt     @ 0x100c : PciRegisterRo<'a, u32>,
        health_syndrom      @ 0x1010 : PciRegisterRo<'a, u32>,
    }
}

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
        for x in cmd_data { signature ^= x; }
        self.signature().write(signature)?;
        Ok(())
    }
}

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

pub fn iommu_map(iommu: pci_driver::iommu::PciIommu, iova: u64, length: usize) -> Result<PciMemoryRegion> {
    unsafe {
        let memory = libc::mmap(null_mut(), length, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_ANONYMOUS | libc::MAP_PRIVATE, -1, 0) as *mut u8;
        iommu.map(iova, length, memory, Permissions::ReadWrite)?;
        Ok(PciMemoryRegion::new_raw(memory, length, Permissions::ReadWrite))
    }
}

const QUERY_HCA_CAP : u32 = 0x01000000_u32;
const QUERY_ADAPTER : u32 = 0x01010000_u32;
const INIT_HCA : u32 = 0x01020000_u32;
const TEARDOWN_HCA : u32 = 0x01030000_u32;
const ENABLE_HCA : u32 = 0x01040000_u32;
const DISABLE_HCA : u32 = 0x01050000_u32;

fn main() -> Result<()> {
    let pci_device = pci_driver::backends::vfio::VfioPciDevice::open("/sys/bus/pci/devices/0000:04:00.0")?;
    pci_device.reset()?;
    pci_device.config().command().bus_master_enable().write(true)?;
    let bar0 = pci_device.bar(0).ok_or(Error::Bar0Error)?;
    let bar0_region = bar0.map(..bar0.len(), Permissions::ReadWrite)?;
    let dma_region = iommu_map(pci_device.iommu(), 0x10000000_u64, 0x100000)?;

    let init_segment = InitSegment::backed_by(&bar0_region);
    dbg!(&init_segment);
    init_segment.cmdq_phy_addr_lo().write(0x10000000_u32.to_be())?;
    while init_segment.initializing().read()?.to_be() & 0x80000000 != 0x00000000 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    };

    let in_mb = Mailbox::backed_by((&dma_region).subregion(0x400..0x800));
    let out_mb = Mailbox::backed_by((&dma_region).subregion(0x800..0xc00));

    let cmd = CQE::backed_by(&dma_region);
    cmd.cmd_type().write(0x07)?;
    cmd.input_length().write(0x00000010_u32.to_be())?;
    cmd.input_mb_ptr_hi().write(0x00000000_u32.to_be())?;
    cmd.input_mb_ptr_lo().write(0x00000000_u32.to_be())?;
    cmd.input_mb_ptr_lo().write(0x10000400_u32.to_be())?;

    cmd.cmd_input_inline0().write(ENABLE_HCA.to_be())?;
    cmd.cmd_input_inline1().write(0x00000000_u32.to_be())?;
    cmd.cmd_input_inline2().write(0x00000000_u32.to_be())?;
    cmd.cmd_input_inline3().write(0x00000000_u32.to_be())?;

    cmd.cmd_output_inline0().write(0x00000000_u32.to_be())?;
    cmd.cmd_output_inline1().write(0x00000000_u32.to_be())?;
    cmd.cmd_output_inline2().write(0x00000000_u32.to_be())?;
    cmd.cmd_output_inline3().write(0x00000000_u32.to_be())?;

    cmd.output_length().write(0x00000010_u32.to_be())?;
    cmd.output_mb_ptr_hi().write(0x00000000_u32.to_be())?;
    cmd.output_mb_ptr_lo().write(0x10000800_u32.to_be())?;

    cmd.token().write(0x00)?;
    cmd.status().write(0x01)?;
    cmd.update_signature()?;

    init_segment.cmdq_doorbell().write(0x00000001_u32.to_be())?;

    while cmd.status().read()? & 0x01 != 0x00 {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    dbg!(&cmd);
//    dbg!(&init_segment);

    Ok(())
}
