#![allow(unused_variables)]

use std::collections::BTreeMap;
use std::{fmt::Debug, path::PathBuf};

use clap::Parser;
#[allow(unused_imports)]
use dbg_hex::dbg_hex;
use mlx5cmd::types::{ManagePagesOpMod, QueryPagesOpMod};
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice};

use mlx5cmd::{
    error::Result, mlx::Mlx5CmdIf
};
use mlx5cmd::types::{
    EnableHCA, ExecShellcode, InitHCA, ManagePages, QueryHCACap, QueryISSI, QueryPages, SetISSI
};

use irisc_asm::assemble_template;
use irisc_asm::utils::{cartesian_product, parse_parameter};

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long, default_value = "/sys/bus/pci/devices/0000:04:00.0")]
    device: PathBuf,

    #[arg(short, long, value_parser = parse_parameter)]
    param: Vec<(String, Vec<u64>)>,

    template: PathBuf,
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

    let out = cmdif.do_command(SetISSI { current_issi: 1 })?;

    let query_boot_pages = cmdif.do_command(QueryPages {
        op_mod: QueryPagesOpMod::BootPages,
    })?;

    let mut available_page = 0x00000000_11000000_u64;
    let mut pages = vec![];
    for _ in 0x00..query_boot_pages.num_pages {
        pages.push(available_page);
        available_page += 0x1000;
    }
    let manage_pages_cmd = ManagePages {
        op_mod: ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_boot_pages.num_pages as u32,
        items: pages,
    };
    cmdif.do_command(manage_pages_cmd)?;

    cmdif.do_command(QueryHCACap { op_mod: 0x0001 })?;

    let query_init_pages = cmdif.do_command(QueryPages {
        op_mod: QueryPagesOpMod::InitPages,
    })?;

    let mut pages = vec![];
    for _ in 0x00..query_init_pages.num_pages {
        pages.push(available_page);
        available_page += 0x1000;
    }
    let manage_pages_cmd = ManagePages {
        op_mod: ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_init_pages.num_pages as u32,
        items: pages,
    };

    cmdif.do_command(manage_pages_cmd)?;
    cmdif.do_command(InitHCA(()))?;

    let template = std::fs::read_to_string(args.template)?;

    for parameters in cartesian_product(args.param).into_iter().map(BTreeMap::from_iter) {
        let (code, labels) = assemble_template(0, &template, &parameters).unwrap();

        let mut shellcode = [0u8; 0xa0];
        shellcode[..code.len()].copy_from_slice(&code);

        dbg_hex!(cmdif.do_command(ExecShellcode {
            op_mod: 0x0000,
            args: [0, 0, 0, 0, 0, 0],
            shellcode,
        })?);
    }

    Ok(())
}
