use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ioerror")]
    IoError(#[from] std::io::Error),

    #[error("Bar0 not found")]
    Bar0Error,

    #[error("Cmdif {0}")]
    CmdIf(u8),

    #[error("Could not serialize command")]
    DekuError(#[from] deku::error::DekuError),
}

pub type Result<T> = std::result::Result<T, Error>;
