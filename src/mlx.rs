use std::{collections::HashMap, fmt::Debug, thread::sleep, time::Duration};

use crate::{
    allocator::{AllocationGuard, Allocator}, commands::{
        access_register::{AccessRegister, AccessRegisterOpMod}, BaseOutputStatus, Command, CommandErrorStatus, ExecShellcode64, ManagePages, ManagePagesOpMod, QueryPages, QueryPagesOpMod
    }, cqe::CQE, error::{Error, Result}, init::InitSegment, mailbox::MailboxAllocator, registers::Register
};
use deku::DekuContainerRead;
use irisc_asm::assemble;
use log::{debug, trace};
use pci_driver::{
    backends::vfio::VfioPciDevice,
    device::PciDevice,
    regions::{
        AsPciSubregion, BackedByPciSubregion, MappedOwningPciRegion, PciMemoryRegion, PciRegion, Permissions
    },
};

use crate::commands::{
    SetISSI, EnableHCA, QueryISSI, QueryHCACap, InitHCA
};

#[allow(dead_code)]
pub struct Mlx5CmdIf {
    pub pci_device: VfioPciDevice,
    pub bar0_region: MappedOwningPciRegion,
    pub dma_allocator: Allocator,
    pub cqe_region: AllocationGuard,
    pub managed_pages: HashMap<u64, AllocationGuard>,
}

const DMA_PAGES: usize = 32768;

impl Mlx5CmdIf {
    pub fn new(pci_device: VfioPciDevice) -> Result<Self> {
        pci_device
            .config()
            .command()
            .bus_master_enable()
            .write(true)?;
        let bar0 = pci_device.bar(0).ok_or(Error::Bar0)?;
        let bar0_region = bar0.map(..bar0.len(), Permissions::ReadWrite)?;
        let dma_region = iommu_map(&pci_device.iommu(), 0x10000000_u64, DMA_PAGES << 12)?;
        let dma_allocator: Allocator = Allocator::new(dma_region, 0x1000);
        let cqe_region: AllocationGuard = dma_allocator.alloc(1).unwrap();
        let cqe_ptr = cqe_region.as_ptr().unwrap() as u64;

        let this = Self {
            pci_device,
            bar0_region,
            dma_allocator,
            cqe_region,
            managed_pages: HashMap::new(),
        };
        this.setup_cmdq_phy_addr(cqe_ptr)?;

        Ok(this)
    }

    pub fn handle_page_request(&mut self, page_type: QueryPagesOpMod) -> Result<()> {
        let query_pages = self.do_command(QueryPages {
            op_mod: page_type,
        })?;

        if query_pages.num_pages > 0 {
            let mut pages = vec![];
            debug!("Allocating {} pages for HCA", query_pages.num_pages);
            for _ in 0..query_pages.num_pages {
                let page = self.dma_allocator.alloc(1).unwrap();
                let page_ptr = page.as_ptr().unwrap() as u64;
                trace!("Allocated {:#x} to HCA", page_ptr);
                pages.push(page_ptr);
                self.managed_pages.insert(page_ptr, page);
            }
            self.do_command(ManagePages {
                op_mod: ManagePagesOpMod::AllocationSuccess,
                input_num_entries: query_pages.num_pages as u32,
                items: pages,
            })?;
        } else if query_pages.num_pages < 0 {
            debug!("Deallocating {:} pages from HCA", -query_pages.num_pages);

            let manage_pages_out = self.do_command(ManagePages {
                op_mod: ManagePagesOpMod::HCAReturnPages,
                input_num_entries: -query_pages.num_pages as u32,
                items: vec![],
            })?;

            for page_ptr in manage_pages_out.items {
                trace!("Deallocated {:#x} from HCA", page_ptr);

                self.managed_pages.remove(&page_ptr);
            }
        }

        Ok(())

    }

    pub fn initialize(&mut self) -> Result<()> {
        self.do_command(EnableHCA(()))?;
        self.do_command(QueryISSI(()))?;
        self.do_command(SetISSI { current_issi: 1 })?;

        self.handle_page_request(QueryPagesOpMod::BootPages)?;

        self.do_command(QueryHCACap { op_mod: 0x0001 })?;

        self.handle_page_request(QueryPagesOpMod::InitPages)?;

        self.do_command(InitHCA(()))?;
        
        Ok(())
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
        log::trace!("Executing command input={input:02x?} outlen={outlen}");
        let cmd = CQE::backed_by(&*self.cqe_region);
        let mailbox_region = self.dma_allocator.alloc(256).unwrap();
        let mut mailbox_allocator = MailboxAllocator::new((&*mailbox_region).subregion(..));

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
        }
        let err = cmd.status().read()? >> 1;
        if err != 0x00 {
            return Err(Error::CmdIf(err));
        }
        log::trace!("Command: {cmd:?}");

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
        log::trace!("Output={output:02x?}");

        Ok(output)
    }

    pub fn do_command<Cmd: Command + Debug>(&self, cmd: Cmd) -> Result<Cmd::Output> {
        log::debug!("Command: {cmd:x?}");
        let msg = cmd.to_bytes()?;
        let out = self.exec_command(&msg, cmd.outlen() as u32)?;
        let base_output = BaseOutputStatus::from_bytes((&out, 0))?.1;
        if base_output.0.status != CommandErrorStatus::Ok {
            return Err(Error::Command {
                status: base_output.0.status,
                syndrome: base_output.0.syndrome,
            });
        }

        let res = Cmd::Output::from_bytes((&out, 0))?.1;
        log::debug!("Output: {res:x?}");
        Ok(res)
    }
}

impl Mlx5CmdIf {
    pub fn read_register<Reg: Register + Debug>(&self, reg: Reg, argument: u32) -> Result<Reg> {
        log::debug!("Reading register {reg:x?}");
        let resp = self.do_command(AccessRegister {
            op_mod: AccessRegisterOpMod::Read,
            argument,
            register_id: Reg::REGISTER_ID,
            register_data: reg.to_bytes()?,
        })?;
        let reg = Reg::from_bytes((&resp.register_data, 0))?.1;
        log::debug!("Register value: {reg:x?}");
        Ok(reg)
    }

    pub fn write_register<Reg: Register + Debug>(&self, reg: Reg, argument: u32) -> Result<Reg> {
        log::debug!("Writing register {reg:x?}");
        let resp = self.do_command(AccessRegister {
            op_mod: AccessRegisterOpMod::Write,
            argument,
            register_id: Reg::REGISTER_ID,
            register_data: reg.to_bytes()?,
        })?;
        let reg = Reg::from_bytes((&resp.register_data, 0))?.1;
        log::debug!("Register value after write {reg:x?}");
        Ok(reg)
    }
}

impl Mlx5CmdIf {
    pub fn run_shellcode(&self, shellcode: &str) -> anyhow::Result<[u64;3]> {
        let (code, _labels) = assemble(0, shellcode)?;
        let mut shellcode = [0u8; 0xa0];
        shellcode[..code.len()].copy_from_slice(&code);
        Ok(self.do_command(ExecShellcode64{
            op_mod: 0,
            args: [0, 0, 0],
            shellcode,
        })?.results)
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
