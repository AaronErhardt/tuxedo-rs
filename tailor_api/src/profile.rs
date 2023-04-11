#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ProfileInfo {
    pub fans: Vec<String>,
    pub leds: Vec<LedProfile>,
    pub performance_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LedProfile {
    pub device_name: String,
    pub function: String,
    pub profile: String,
}
