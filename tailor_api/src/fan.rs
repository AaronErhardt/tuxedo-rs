#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FanProfilePoint {
    pub temp: u8,
    pub fan: u8,
}
