#![allow(unused_variables)]

use std::collections::BTreeMap;
use std::io::Write;
use std::time::SystemTime;
use std::{fmt::Debug, path::PathBuf};

use clap::Parser;
use mlx5cmd::cmdif::CmdIf;
use mlx5cmd::commands::ExecShellcode64;
use pci_driver::{backends::vfio::VfioPciDevice, device::PciDevice};

use mlx5cmd::{
    error::Result, cmdif::vfio::VfioCmdIf
};

use irisc_asm::assemble_template;
use irisc_asm::utils::{cartesian_product, parse_parameter};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;


#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long, default_value = "/sys/bus/pci/devices/0000:04:00.0")]
    device: PathBuf,

    #[arg(long, default_value = "data")]
    outdir: PathBuf,

    #[arg(long, default_value_t = 16*1024*1024)]
    output_buffer: usize,

    #[arg(short, long, value_parser = parse_parameter)]
    param: Vec<(String, Vec<u64>)>,


    template: PathBuf,
    output: Option<PathBuf>,
}


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Experiment {
    parameters: BTreeMap<String, Vec<u64>>,
    arguments: [Vec<u32>; 6],
    template: String,
    timestamp: u64,
}


#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct ExperimentData {
    #[serde_as(as = "serde_with::hex::Hex")]
    shellcode: Vec<u8>,
    parameters: BTreeMap<String, u64>,
    arguments: [u64; 3],
    results: [u64; 3],
    execution_time: u128,
    assembly_time: u128,
    total_time: u128,
}


fn main() -> Result<()> {
    env_logger::init();
    let args = CliArgs::parse();

    let pci_device = VfioPciDevice::open(args.device)?;
    pci_device.reset()?;

    let mut cmdif = VfioCmdIf::new(pci_device)?;
    cmdif.initialize()?;

    let template = std::fs::read_to_string(&args.template)?;
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let output_path = args.output.unwrap_or_else(|| {
        std::fs::create_dir_all(&args.outdir).unwrap();
        args.outdir.join(args.template.with_extension(format!("{}.json", timestamp)))
    });
    let mut output = std::io::BufWriter::with_capacity(
        args.output_buffer,
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_path)?
    );

    fn output_json(output: &mut impl Write, value: &impl Serialize) -> Result<usize> {
        Ok(output.write(format!("{}\n", serde_json::to_string(value).unwrap()).as_bytes())?)
    }

    output_json(&mut output,&Experiment {
        parameters: BTreeMap::from_iter(args.param.clone()),
        arguments: Default::default(),
        template: template.clone(),
        timestamp: timestamp,
    })?;

    let mut iteration_start_time = SystemTime::now();

    for (i, parameters) in cartesian_product(args.param).into_iter().map(BTreeMap::from_iter).enumerate() {
        if i & 0x3ff == 0 {
            dbg!(i);
            dbg!(&parameters);
        }
        let assembly_start_time = SystemTime::now();

        let (code, labels) = assemble_template(0, &template, &parameters).unwrap();

        let assembly_time = assembly_start_time.elapsed().unwrap().as_nanos();

        let mut shellcode = [0u8; 0xa0];
        shellcode[..code.len()].copy_from_slice(&code);

        let execution_start_time = std::time::SystemTime::now();

        let exec_output = cmdif.do_command(ExecShellcode64 {
            op_mod: 0x0000,
            args: [0, 0, 0],
            shellcode,
        })?;

        let execution_time = execution_start_time.elapsed().unwrap().as_nanos();

        let total_time = iteration_start_time.elapsed().unwrap().as_nanos();

        iteration_start_time = SystemTime::now();

        output_json(&mut output, &ExperimentData {
            parameters: parameters,
            shellcode: code,
            arguments: [0; 3],
            results: exec_output.results,
            execution_time,
            assembly_time,
            total_time,
        })?;
    }

    output.flush()?;

    Ok(())
}
