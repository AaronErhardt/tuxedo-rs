use thiserror::Error;
use zbus::fdo;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Bus response error")]
    BusError(#[from] fdo::Error),
    #[error("Serialization error")]
    Serialization(#[from] serde_json::Error),
}
