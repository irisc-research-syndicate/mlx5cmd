#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unreachable_code)]

use std::path::Path;
use std::{fmt::Debug, io::Write, path::PathBuf, time::Duration};
use std::os::fd::*;

use clap::Parser;
use irisc_asm::assemble;
use log::{debug, info, trace};
use mlx5cmd::allocator::AllocationGuard;
use mlx5cmd::cmdif::CmdIf;
use mlx5cmd::commands::QueryEQ;
use mlx5cmd::{commands::{create_mkey::{AccessMode, CreateMKey, MKeyContext}, AllocPD, AllocUAR, CreateEQ, EQContext, EnableHCA, ExecShellcode64, InitHCA, ManagePages, ManagePagesOpMod, QueryHCACap, QueryISSI, QueryPages, QueryPagesOpMod, SetISSI}, mtcr::{VCR_CMD_ADDR, VCR_CTRL_ADDR}, registers::mtrc::{MtrcCapReg, MtrcConfReg, MtrcCtrlReg}};
use mlx5cmd::mtcr::{ICmd, AS_CR_SPACE, AS_EXPANSION_ROM, AS_ICMD, AS_ICMD_EXT, AS_NODNIC_INIT_SEG, MTCR};
use pci_driver::regions::PciMemoryRegion;
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice, regions::PciRegion};
use dbg_hex::dbg_hex;

use mlx5cmd::{
    error::Result, cmdif::vfio::VfioCmdIf
};

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long, default_value = "/sys/bus/pci/devices/0000:04:00.0")]
    device: PathBuf,
}

pub fn clear_allocation(allocation: &AllocationGuard) {
    for i in 0..allocation.len() {
        allocation.write_u8(i, 0u8).unwrap();
    }
}

pub fn create_mtt_mkey(cmdif: &VfioCmdIf, pd: u32, key:u8, pages: usize) -> Result<(u32, AllocationGuard)> {
    let memory = cmdif.dma_allocator.alloc(pages).unwrap();
    clear_allocation(&memory);
    let mkey_index = cmdif.do_command(CreateMKey {
        pg_access: false,
        umem_valid: false,
        context: MKeyContext {
            free: false,
            umr_en: false,
            a: false,
            rw: false,
            rr: false,
            lw: true,
            lr: true,
            access_mode: AccessMode::MTT,
            qpn: 0xffffff,
            mkey: key,
            length64: false,
            pd: pd,
            // start_addr: memory.as_ptr().unwrap() as u64,
            // len: memory.len(),
            start_addr: 0x0000,
            len: 0x1000 * pages as u64,
            bsf_octword_size: 0, 
            translation_octword_size: (pages as u32) / 2,
            log_entry_size: 12,
        },
        translation_octwords_actual_size: pages as u32 / 2,
        translation_entries: (0..pages).map(|i| (memory.as_ptr().unwrap() as u64 + 0x1000 * (i as u64)) | 0).collect(),
    })?.mkey_index;

    Ok(((mkey_index << 8) | (key as u32), memory))
}

pub fn create_pa_mkey(cmdif: &VfioCmdIf, pd: u32, key: u8, pages: usize) -> Result<(u32, AllocationGuard)> {
    let memory = cmdif.dma_allocator.alloc(pages).unwrap();
    clear_allocation(&memory);
    let mkey_index = dbg!(cmdif.do_command(CreateMKey {
        pg_access: false,
        umem_valid: false,
        context: MKeyContext {
            free: false,
            umr_en: false,
            a: false,
            rw: false,
            rr: false,
            lw: true,
            lr: true,
            access_mode: AccessMode::PA,
            qpn: 0xffffff,
            mkey: key,
            length64: false,
            pd: pd,
            start_addr: memory.as_ptr().unwrap() as u64,
            len: memory.len(),
            bsf_octword_size: 0,
            translation_octword_size: 0,
            log_entry_size: 0,
        },
        translation_octwords_actual_size: 0,
        translation_entries: vec![],
    }))?.mkey_index;

    Ok(((mkey_index << 8) | (key as u32), memory))
}

pub fn create_eq(cmdif: &VfioCmdIf, log_eq_size: u8, event_bitmask: u64) -> Result<(u32, u8, AllocationGuard)> {
    let eq_length = 1usize << log_eq_size;
    let eq_size = eq_length * 0x40usize;
    let pages = (eq_size + 0xfff) >> 12;

    let eq_memory = cmdif.dma_allocator.alloc(pages).unwrap();
    clear_allocation(&eq_memory);
    for i in 0..eq_length {
        eq_memory.write_u8(((0x40 * i) + 0x3f) as u64, 0x01)?;
    }

    let eq_uar = cmdif.do_command(AllocUAR{})?.uar;
    let eq_num = cmdif.do_command(CreateEQ {
        ctx: EQContext {
            status: 0,
            ec: false,
            oi: false,
            st: 0,
            log_eq_size: log_eq_size,
            uar_page: eq_uar,
            intr: 0,
            log_page_size: log_eq_size - 6,
            consumer_counter: 0,
            producer_counter: 0
        },
        event_bitmask: event_bitmask,
        pas: (0..pages).map(|page| eq_memory.as_ptr().unwrap() as u64 + (page as u64) * 0x1000).collect()
    })?.eq;
    Ok((eq_uar, eq_num, eq_memory))
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = CliArgs::parse();

    let pci_device = VfioPciDevice::open(args.device)?;
    pci_device.reset()?;
    let mut cmdif = VfioCmdIf::new(pci_device)?;

    {
        let mtcr = MTCR::open_from_cmdif(&cmdif)?;

        // 0xbadcoffe unlock
        mtcr.change_address_space(AS_CR_SPACE)?;
        mtcr.write_dword(0x23f0, 0xbadc0ffe)?;

        let icmd = ICmd::from_mtcr(mtcr)?;
        dbg!(icmd.set_itrace(0xffffffff, 1, 0))?;
        dbg!(icmd.check_badc0ffe_unlocked())?;
    }

    cmdif.initialize()?;

    //let interrupts = cmdif.pci_device.interrupts();
    //let intr_evfds = (0..interrupts.msi_x().max()).map(|_| Ok(eventfd::EventFD::new(0, eventfd::EfdFlags::EFD_CLOEXEC)?) ).collect::<Result<Vec<_>>>()?;
    //dbg!(interrupts.msi_x().enable(&intr_evfds.iter().map(|evfd| evfd.as_raw_fd()).collect::<Vec<_>>()))?;

    //let (pagerequest_eq_uar, pagereqeust_eq, pagerequest_eq_mem) = create_eq(&cmdif, 6, 1 << 0x0b)?;

    let trace_pd = cmdif.do_command(AllocPD{})?.pd;
    //let (trace_mkey, trace_mem) = create_mtt_mkey(&cmdif, trace_pd, 0x00, 2)?;
    let (trace_mkey, trace_mem) = create_mtt_mkey(&cmdif, trace_pd, 0x42, 4)?;

    // cmdif.read_register(MtrcCapReg::default(), 0)?;

    cmdif.write_register(MtrcCapReg {
        trace_owner: true,
        ..Default::default()
    }, 0)?;

    cmdif.write_register(MtrcConfReg {
        trace_mode: 1,
        trace_mkey: trace_mkey,
        log_trace_buffer_size: 2,
    }, 0)?;

    cmdif.write_register(MtrcCtrlReg {
        trace_status: 1,
        arm_event: true,
        modify_field_select: 1,
        ..Default::default()
    }, 0)?;

    cmdif.run_shellcode(r"
        lbl entry
            set64 r5, 0x23f0
            set64 r6, 0
            set64 r7, 0

        lbl test
            ld.d r5, r5, 0x00
            addi r6, r1, 0

        lbl result
            st.q r0, r4, r5, 0x08
            st.q r0, r4, r6, 0x10
            st.q r0, r4, r7, 0x18

        lbl exit
            ret.d
    ")?;

    cmdif.handle_page_request(QueryPagesOpMod::RegularPages)?;
    cmdif.handle_page_request(QueryPagesOpMod::RegularPages)?;

    fn write_region<'a, P: AsRef<Path>>(path: P, region: PciMemoryRegion<'a>) -> Result<()> {
        let mut buffer = vec![0u8; region.len() as usize];
        region.read_bytes(0, &mut buffer)?;
        std::fs::write(path, buffer)?;
        Ok(())
    }

    write_region("trace_buffer", *trace_mem)?;
//    write_region("pagerequest_eq_buffer", *pagerequest_eq_mem)?;

//    write_region("dma_region", cmdif.dma_allocator.0.lock().unwrap().memory)?;

    Ok(())
}