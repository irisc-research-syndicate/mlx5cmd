#![allow(unused_variables)]

use std::{fmt::Debug, io::Write, path::PathBuf};

use clap::Parser;
use log::{debug, trace};
use mlx5cmd::{cmdif::CmdIf, commands::{EnableHCA, InitHCA, ManagePages, ManagePagesOpMod, QueryHCACap, QueryISSI, QueryPages, QueryPagesOpMod, SetISSI}};
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice, regions::PciRegion};

use mlx5cmd::{
    error::Result, cmdif::vfio::Mlx5CmdIf
};

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long, default_value = "/sys/bus/pci/devices/0000:04:00.0")]
    device: PathBuf,

    output: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = CliArgs::parse();

    let pci_device = VfioPciDevice::open(args.device)?;
    pci_device.reset()?;

    let cmdif = Mlx5CmdIf::new(pci_device)?;
    cmdif.do_command(EnableHCA(()))?;
    cmdif.do_command(QueryISSI(()))?;
    cmdif.do_command(SetISSI { current_issi: 1 })?;

    let mut managed_pages = vec![];

    let query_boot_pages = cmdif.do_command(QueryPages {
        op_mod: QueryPagesOpMod::BootPages,
    })?;

    let mut pages = vec![];
    debug!("Allocating {} boot pages", query_boot_pages.num_pages);
    for _ in 0..query_boot_pages.num_pages {
        let page = cmdif.dma_allocator.alloc(1).unwrap();
        trace!("Allocated {:?} ", page.as_ptr().unwrap());
        pages.push(page.as_ptr().unwrap() as u64);
        managed_pages.push(page);
    }

    cmdif.do_command(ManagePages {
        op_mod: ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_boot_pages.num_pages as u32,
        items: pages,
    })?;

    cmdif.do_command(QueryHCACap { op_mod: 0x0001 })?;

    let query_init_pages = cmdif.do_command(QueryPages {
        op_mod: QueryPagesOpMod::InitPages,
    })?;


    let mut pages = vec![];
    debug!("Allocating {} init pages", query_init_pages.num_pages);
    for _ in 0..query_init_pages.num_pages {
        let page = cmdif.dma_allocator.alloc(1).unwrap();
        trace!("Allocated {:?} ", page.as_ptr().unwrap());
        pages.push(page.as_ptr().unwrap() as u64);
        managed_pages.push(page);
    }
    cmdif.do_command(ManagePages {
        op_mod: ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_init_pages.num_pages as u32,
        items: pages,
    })?;

    cmdif.do_command(InitHCA(()))?;

    let mut output = std::fs::OpenOptions::new().write(true).create(true).open(args.output)?;
    for page in managed_pages {
        let mut content = [0u8; 4096];
        let address = page.as_ptr().unwrap() as u64;
        page.read_bytes(0, &mut content)?;
        output.write(&address.to_be_bytes())?;
        output.write(&content)?;
    }

    Ok(())
}