#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LedDeviceInfo {
    pub device_name: String,
    pub function: String,
}

impl LedDeviceInfo {
    pub fn device_id(&self) -> String {
        let Self {
            device_name,
            function,
        } = self;
        format!("{device_name}::{function}")
    }
}
