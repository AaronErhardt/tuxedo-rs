use thiserror::Error;
use zbus::fdo;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Bus response error: `{0}`")]
    BusError(#[from] fdo::Error),
    #[error("Serialization error: `{0}`")]
    Serialization(#[from] serde_json::Error),
}