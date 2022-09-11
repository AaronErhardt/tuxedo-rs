use std::path::Component;

use tailor_api::{ColorProfile, ProfileInfo};
use tokio::sync::mpsc;
use zbus::{dbus_interface, fdo};

use crate::{
    fancontrol::profile::FanProfile,
    profiles::{Profile, PROFILE_DIR},
    util,
};

pub struct ProfileInterface {
    pub fan_sender: mpsc::Sender<FanProfile>,
    pub keyboard_sender: mpsc::Sender<ColorProfile>,
}

#[dbus_interface(name = "com.tux.Tailor.Profiles")]
impl ProfileInterface {
    async fn add_profile(&self, name: &str, value: &str) -> fdo::Result<()> {
        // Verify correctness of the file.
        serde_json::from_str::<ProfileInfo>(value)
            .map_err(|err| fdo::Error::InvalidArgs(err.to_string()))?;

        util::write_file(PROFILE_DIR, name, value.as_bytes()).await
    }

    async fn get_profile(&self, name: &str) -> fdo::Result<String> {
        util::read_file(PROFILE_DIR, name).await
    }

    async fn list_profiles(&self) -> fdo::Result<Vec<String>> {
        util::get_profiles(PROFILE_DIR).await
    }

    async fn remove_profile(&self, name: &str) -> fdo::Result<()> {
        util::remove_file(PROFILE_DIR, name).await
    }

    async fn set_active_profile_name(&self, name: &str) -> fdo::Result<()> {
        std::fs::metadata(util::normalize_json_path(PROFILE_DIR, name)?)
            .map_err(|_| fdo::Error::FileNotFound(format!("Couldn't find profile `{name}`")))?;

        let link_path = format!("{PROFILE_DIR}active_profile.json");
        drop(std::fs::remove_file(&link_path));
        std::os::unix::fs::symlink(util::normalize_json_path("", name)?, &link_path)
            .map_err(|err| fdo::Error::IOError(err.to_string()))
    }

    async fn get_active_profile_name(&self) -> fdo::Result<String> {
        let link = std::fs::read_link(&format!("{PROFILE_DIR}active_profile.json"))
            .map_err(|err| fdo::Error::IOError(err.to_string()))?;
        let components: Vec<Component> = link.components().collect();
        if components.len() == 1 {
            if let Component::Normal(name) = components.first().unwrap() {
                if let Some(name) = name.to_str() {
                    return Ok(name.trim_end_matches(".json").to_string());
                }
            }
        }

        Err(fdo::Error::InvalidFileContent(
            "File `active_profile.json` isn't a valid link".to_string(),
        ))
    }

    async fn reload(&mut self) -> fdo::Result<()> {
        let Profile { fan, keyboard } = Profile::reload()?;
        let res1 = self
            .keyboard_sender
            .send(keyboard)
            .await
            .map_err(|e| e.to_string());
        let res2 = self.fan_sender.send(fan).await.map_err(|e| e.to_string());
        res1.and(res2)
            .map_err(|err| fdo::Error::Failed(format!("Internal error: `{err}`")))?;
        Ok(())
    }
}
