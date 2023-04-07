use std::string::FromUtf8Error;

use nix::errno::Errno;

#[derive(thiserror::Error, Debug)]
pub enum IoctlError {
    #[error("Parsing to UTF8 failed")]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Read(#[from] Errno),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Device not available")]
    DevNotAvailable,
    #[error("Invalid args")]
    InvalidArgs,
    #[error("Feature not available")]
    NotAvailable,
}
