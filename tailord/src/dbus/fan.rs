use tailor_api::fan::FanProfilePoint;
use tokio::sync::mpsc;
use zbus::{dbus_interface, fdo};

use crate::{profiles::FAN_DIR, util};

pub struct FanInterface {
    pub fan_speed_sender: mpsc::Sender<u8>,
}

#[dbus_interface(name = "com.tux.Tailor.Fan")]
impl FanInterface {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<Vec<FanProfilePoint>>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        util::write_file(FAN_DIR, name, value.as_bytes()).await
    }

    async fn get_profile(&self, name: &str) -> fdo::Result<String> {
        util::read_file(FAN_DIR, name).await
    }

    async fn list_profiles(&self) -> fdo::Result<Vec<String>> {
        util::get_profiles(FAN_DIR).await
    }

    async fn remove_fan_profile(&self, name: &str) -> fdo::Result<()> {
        util::remove_file(FAN_DIR, name).await
    }

    async fn override_speed(&mut self, speed: u8) -> fdo::Result<()> {
        self.fan_speed_sender
            .send(speed)
            .await
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))
    }
}
