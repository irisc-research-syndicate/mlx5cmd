use std::{fmt::Debug, thread::sleep, time::Duration};

use crate::{
    cqe::CQE,
    error::{Error, Result},
    init::InitSegment,
    mailbox::MailboxAllocator,
    types::{Command, CommandOutput},
};
use deku::DekuContainerRead;
use pci_driver::{
    backends::vfio::VfioPciDevice,
    device::PciDevice,
    regions::{
        AsPciSubregion, BackedByPciSubregion, MappedOwningPciRegion, PciMemoryRegion, PciRegion,
        Permissions,
    },
};

#[allow(dead_code)]
pub struct Mlx5CmdIf<'a> {
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
        let bar0 = pci_device.bar(0).ok_or(Error::Bar0)?;
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

    pub fn iommu_map(&self, iova: u64, length: usize) -> Result<PciMemoryRegion<'a>> {
        iommu_map(&self.pci_device.iommu(), iova, length)
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
        log::debug!("Executing command input={input:02x?} outlen={outlen}");
        let cmd = CQE::backed_by((&self.dma_region).subregion(0x000..0x400));
        let mut mailbox_allocator = MailboxAllocator::new((&self.dma_region).subregion(0x1000..));

        cmd.cmd_type().write(0x07)?;

        cmd.set_input_mb(0)?;
        cmd.set_output_mb(0)?;

        cmd.input_length().write((input.len() as u32).to_be())?;
        for (i, b) in input[..0x10].iter().enumerate() {
            cmd.write_u8(0x10 + i as u64, *b)?;
        }

        let in_mb_vec = mailbox_allocator.build_mailbox(0x00, &input[0x10..])?;
        if let Some(in_mb) = in_mb_vec.first() {
            cmd.set_input_mb(in_mb.as_ptr().unwrap() as u64)?;
        }

        cmd.output_length().write(outlen.to_be())?;
        for (i, b) in [0u8; 0x10].iter().enumerate() {
            cmd.write_u8(0x20 + i as u64, *b)?;
        }

        let out_mb_vec = mailbox_allocator.build_mailbox(0x00, &vec![0u8; outlen as usize])?;
        if let Some(out_mb) = out_mb_vec.first() {
            cmd.set_output_mb(out_mb.as_ptr().unwrap() as u64)?;
        }

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

    pub fn do_command<Cmd: Command + Debug>(&self, cmd: Cmd) -> Result<Cmd::Output> {
        let msg = cmd.to_bytes()?;
        let out = self.exec_command(&msg, cmd.outlen() as u32)?;
        let res = Cmd::Output::from_bytes((&out, 0))?.1;
        log::info!(
            "command={cmd:?} status={status} syndrome={syndrome}",
            status = res.status(),
            syndrome = res.syndrome(),
        );
        log::debug!("output: {res:?}");
        Ok(res)
    }
}

fn iommu_map(
    iommu: &pci_driver::iommu::PciIommu,
    iova: u64,
    length: usize,
) -> Result<PciMemoryRegion<'static>> {
    unsafe {
        let memory = libc::mmap(
            iova as *mut libc::c_void,
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
