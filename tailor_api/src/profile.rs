#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProfileInfo {
    pub fans: Vec<String>,
    pub leds: Vec<LedProfile>,
    pub performance_profile: Option<String>,
}

impl Default for ProfileInfo {
    fn default() -> Self {
        Self {
            fans: vec!["default".to_owned()],
            leds: Default::default(),
            performance_profile: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LedProfile {
    pub device_name: String,
    pub function: String,
    pub profile: String,
}
