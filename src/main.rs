#![allow(unused_variables)]

use std::{fmt::Debug, io::Write, path::PathBuf};

use clap::Parser;
#[allow(unused_imports)]
use dbg_hex::dbg_hex;
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice};

use crate::{
    error::Result, mlx::Mlx5CmdIf
};
use crate::types::{
    create_mkey::{AccessMode, CreateMKey, MKeyContext}, AllocPD, EnableHCA, ExecShellcode, InitHCA, ManagePages, QueryHCACap, QueryISSI, QueryPages, SetISSI
};
use crate::registers::mtrc::{MtrcCapReg, MtrcConfReg};

pub mod cqe;
pub mod error;
pub mod init;
pub mod mailbox;
pub mod mlx;
pub mod types;
pub mod registers;

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
    cmdif.do_command(EnableHCA(()))?;
    cmdif.do_command(QueryISSI(()))?;

    let out = cmdif.do_command(SetISSI { current_issi: 1 })?;

    let query_boot_pages = cmdif.do_command(QueryPages {
        op_mod: types::QueryPagesOpMod::BootPages,
    })?;

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

    cmdif.do_command(QueryHCACap { op_mod: 0x0001 })?;

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

    //let mtrc_cap_reg = dbg!(cmdif.read_register(MtrcCapReg::default(), 0)?);

    //let mut stdbs = vec![];

    //struct StringDB {
    //    num: u8,
    //    address: u32,
    //    data: Vec<u8>,
    //}

    //for stdb_num in 0..mtrc_cap_reg.num_string_db {
    //    let mut stdb = StringDB {
    //        num: stdb_num,
    //        address: mtrc_cap_reg.string_db_param[stdb_num as usize].address,
    //        data: vec![],
    //    };

    //    for offset in (0..mtrc_cap_reg.string_db_param[stdb_num as usize].size).step_by(64) {
    //        let stdb_reg = cmdif.read_register(
    //            MtrcStdbReg {
    //                index: stdb_num,
    //                size: 64,
    //                offset,
    //                data: [0u8; 64],
    //            },
    //            0,
    //        )?;

    //        stdb.data.extend_from_slice(&stdb_reg.data);
    //    }
    //    let mut stdb_file =
    //        std::fs::File::create(format!("stdb.{}.{:#010x}", stdb.num, stdb.address))?;
    //    stdb_file.write_all(&stdb.data)?;

    //    stdbs.push(stdb);
    //}
    let trace_pd = dbg!(cmdif.do_command(AllocPD{})?).pd;

    let trace_mkey_index = dbg_hex!(cmdif.do_command(CreateMKey {
        pg_access: false,
        umem_valid: false,
        context: MKeyContext {
            free: false,
            umr_en: false,
            rw: false,
            rr: false,
            lw: true,
            lr: true,
            access_mode: AccessMode::MTT,
            qpn: 0xffffff,
            mkey: 0x0c,
            length64: false,
            pd: trace_pd,
            start_addr: 0x00000000_10800000,
            len: 0x00000000_00002000,
            translation_octword_size: 1,
            log_entry_size: 12,
        },
        translation_octwords_actual_size: 1,
        translation_entries: vec![
            0x00000000_10800000, 0x000000000_10801000
        ],
    })?).mkey_index;

    dbg_hex!(cmdif.write_register(MtrcCapReg {
        trace_owner: true,
        ..Default::default()
    }, 0)?);

    dbg_hex!(cmdif.write_register(MtrcConfReg {
        trace_mode: 1,
        trace_mkey: (trace_mkey_index << 8) | 0x0c,
        log_trace_buffer_size: 6,
    }, 0))?;

    //let mut shellcode = [0u8; 0xa0];
    //shellcode[..SHELLCODE.len()].copy_from_slice(SHELLCODE);

    //dbg_hex!(cmdif.do_command(ExecShellcode {
    //    op_mod: 0x0000,
    //    args: [0, 0, 0, 0, 0, 0],
    //    shellcode,
    //})?);

    Ok(())
}
