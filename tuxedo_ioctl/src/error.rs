use std::string::FromUtf8Error;

use nix::errno::Errno;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum IoctlError {
    #[error("Parsing to UTF8 failed")]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    Read(#[from] Errno),
    #[error("Device not available")]
    DevNotAvailable,
}
