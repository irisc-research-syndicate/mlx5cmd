use thiserror::Error;

use crate::commands::CommandErrorStatus;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ioerror")]
    Io(#[from] std::io::Error),

    #[error("Bar0 not found")]
    Bar0,

    #[error("Cmdif {0}")]
    CmdIf(u8),

    #[error("Could not serialize command")]
    Deku(#[from] deku::error::DekuError),

    #[error("Command error: status={status} syndrome={syndrome}")]
    Command {
        status: CommandErrorStatus,
        syndrome: u32,
    },

    #[error("Out of memory")]
    OutOfMemory,
}
pub type Result<T> = std::result::Result<T, Error>;
