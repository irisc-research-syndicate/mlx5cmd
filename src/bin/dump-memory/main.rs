#![allow(unused_variables)]

use std::io::Write;
use std::{fmt::Debug, path::PathBuf};

use clap::Parser;
use clap_num::maybe_hex;
use log::debug;
use mlx5cmd::cmdif::CmdIf;
use mlx5cmd::commands::ExecShellcode64;
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice};

use mlx5cmd::{
    error::Result, mlx::Mlx5CmdIf
};

use irisc_asm::assemble;

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long, default_value = "/sys/bus/pci/devices/0000:04:00.0")]
    device: PathBuf,

    #[clap(value_parser=maybe_hex::<u32>)]
    base: u32,

    #[clap(value_parser=maybe_hex::<u32>)]
    length: u32,

    output: PathBuf,
}

const READMEM_SHELLCODE: &'static str = r"
lbl entry
    ld.q r5, r4, 0x08
    ld.q r6, r5, 0x00
    st.q r0, r4, r6, 0x10
    ret.d
";

fn main() -> Result<()> {
    env_logger::init();
    let args = CliArgs::parse();

    let pci_device = VfioPciDevice::open(args.device)?;
    pci_device.reset()?;

    let mut cmdif = Mlx5CmdIf::new(pci_device)?;
    cmdif.initialize()?;

    let (code, labels) = assemble(0x00000000u32, READMEM_SHELLCODE).unwrap();

    let mut output = std::fs::OpenOptions::new().write(true).create(true).open(args.output)?;

    for address in (args.base..args.base+args.length).step_by(8) {
        let mut shellcode = [0u8; 0xa0];
        shellcode[..code.len()].copy_from_slice(&code);

        let exec_output = cmdif.do_command(ExecShellcode64 {
            op_mod: 0x0000,
            args: [address as u64, 0, 0],
            shellcode,
        })?;

        debug!("{:#018x}: {:#018x}", address, exec_output.results[1]);
        output.write(&exec_output.results[1].to_be_bytes())?;
    }

    Ok(())
}