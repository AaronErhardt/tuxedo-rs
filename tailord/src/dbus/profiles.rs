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

    async fn rename_profile(&mut self, old_name: &str, new_name: &str) -> fdo::Result<Vec<String>> {
        if self.list_profiles().await?.contains(&new_name.to_string()) {
            Err(fdo::Error::InvalidArgs(format!(
                "File `{old_name}` already exists"
            )))
        } else {
            util::move_file(PROFILE_DIR, old_name, new_name).await?;

            if self.get_active_profile_name().await? == old_name {
                self.set_active_profile_name(new_name).await?;
                self.reload().await?;
            }

            self.list_profiles().await
        }
    }

    async fn set_active_profile_name(&self, name: &str) -> fdo::Result<()> {
        Profile::set_active_profile_name(name).await
    }

    async fn get_active_profile_name(&self) -> fdo::Result<String> {
        Profile::get_active_profile_name().await
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
