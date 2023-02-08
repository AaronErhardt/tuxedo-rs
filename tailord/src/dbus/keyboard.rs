use tailor_api::{Color, ColorProfile, ProfileInfo};
use tokio::sync::mpsc;
use zbus::{dbus_interface, fdo};

use crate::{
    profiles::{Profile, KEYBOARD_DIR, PROFILE_DIR},
    util,
};

pub struct KeyboardInterface {
    pub color_sender: mpsc::Sender<Color>,
    pub keyboard_sender: mpsc::Sender<ColorProfile>,
}

#[dbus_interface(name = "com.tux.Tailor.Keyboard")]
impl KeyboardInterface {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ColorProfile>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;
        util::write_file(KEYBOARD_DIR, name, value.as_bytes()).await?;

        // Reload if the keyboard profile is part of the active global profile
        let info = Profile::get_active_profile_info()?;
        if info.keyboard == name {
            let info = Profile::reload()?;
            self.keyboard_sender.send(info.keyboard).await.unwrap();
        }
        Ok(())
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

    async fn rename_profile(&self, from: &str, to: &str) -> fdo::Result<Vec<String>> {
        if self.list_profiles().await?.contains(&to.to_string()) {
            Err(fdo::Error::InvalidArgs(format!(
                "File `{to}` already exists"
            )))
        } else {
            let profiles = util::get_profiles(PROFILE_DIR).await?;

            for profile in profiles {
                let mut data = util::read_json::<ProfileInfo>(PROFILE_DIR, &profile).await?;
                if data.keyboard == from {
                    data.keyboard = to.to_string();
                    util::write_json(PROFILE_DIR, &profile, &data).await?;
                }
            }

            util::move_file(KEYBOARD_DIR, from, to).await?;

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
