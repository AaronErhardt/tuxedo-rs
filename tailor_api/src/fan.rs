#[derive(serde::Deserialize, Debug)]
pub struct FanProfilePoint {
    pub temp: u8,
    pub fan: u8,
}
