#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfileInfo {
    pub fan: String,
    pub keyboard: String,
}

impl Default for ProfileInfo {
    fn default() -> Self {
        Self {
            fan: "default".to_string(),
            keyboard: "default".to_string(),
        }
    }
}
