use tailor_api::{Color, ColorProfile, ProfileInfo};
use tokio::sync::mpsc;
use zbus::{dbus_interface, fdo};

use crate::{profiles::{KEYBOARD_DIR, PROFILE_DIR}, util};

pub struct KeyboardInterface {
    pub color_sender: mpsc::Sender<Color>,
}

#[dbus_interface(name = "com.tux.Tailor.Keyboard")]
impl KeyboardInterface {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ColorProfile>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        util::write_file(KEYBOARD_DIR, name, value.as_bytes()).await
    }

    async fn get_profile(&self, name: &str) -> fdo::Result<String> {
        util::read_file(KEYBOARD_DIR, name).await
    }

    async fn list_profiles(&self) -> fdo::Result<Vec<String>> {
        util::get_profiles(KEYBOARD_DIR).await
    }

    async fn remove_profile(&self, name: &str) -> fdo::Result<()> {
        util::remove_file(KEYBOARD_DIR, name).await
    }

    async fn rename_profile(&self, old_name: &str, new_name: &str) -> fdo::Result<Vec<String>> {
        if self.list_profiles().await?.contains(&new_name.to_string()) {
            Err(fdo::Error::InvalidArgs(format!("File `{old_name}` already exists")))
        } else {
            let profiles = util::get_profiles(PROFILE_DIR).await?;

            for profile in profiles {
                let mut data = util::read_json::<ProfileInfo>(&PROFILE_DIR, &profile).await?;
                if data.keyboard == old_name {
                    data.keyboard = new_name.to_string();
                    util::write_json(&PROFILE_DIR, &profile, &data).await?;
                }
            }

            util::move_file(KEYBOARD_DIR, new_name, old_name).await?;

            self.list_profiles().await
        }
    }

    async fn override_color(&mut self, color: &str) -> fdo::Result<()> {
        let color: Color =
            serde_json::from_str(color).map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        self.color_sender
            .send(color)
            .await
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))
    }
}
