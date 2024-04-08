#[derive(
    Default, Debug, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash,
)]
#[non_exhaustive]
pub enum LedControllerMode {
    #[default]
    Rgb,
    Monochrome,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LedDeviceInfo {
    pub device_name: String,
    pub function: String,
    pub mode: LedControllerMode,
}

impl LedDeviceInfo {
    pub fn device_id(&self) -> String {
        let Self {
            device_name,
            function,
            mode: _mode,
        } = self;
        format!("{device_name}::{function}")
    }
}
