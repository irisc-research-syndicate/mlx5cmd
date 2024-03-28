use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {

    #[error("ioerror")]
    IoError(#[from] std::io::Error),

    #[error("Bar0 not found")]
    Bar0Error
}

pub type Result<T> = std::result::Result<T, Error>;
