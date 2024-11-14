use std::path::PathBuf;

use clap::Parser;

use mlx5cmd::cmdif::CmdIf;
use mlx5cmd::registers::flash::MFBA;
use mlx5cmd::registers::flash::MFPA;
use mlx5cmd::cmdif::vfio::VfioCmdIf;

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(short, long, default_value = "/sys/bus/pci/devices/0000:04:00.0")]
    device: PathBuf,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = CliArgs::parse();

    let cmdif = VfioCmdIf::open_from_sysfs(&args.device, true, true)?;

    let mut flash_data = vec![];

    log::info!("{:x?}", cmdif.read_register(MFPA::default(), 0)?);
    for address in (0..16*1024*1024).step_by(0x40) {
        if address & 0xffff == 0 {
            log::info!("{:x}", address);
        }
        let chunk = cmdif.read_register(MFBA {
            size: 0x40,
            address: address,
            ..MFBA::default()
        }, 0)?.data;
        flash_data.extend_from_slice(&chunk);
    }

    std::fs::write("flash_data", flash_data)?;

    Ok(())
}