use tailor_api::ProfileInfo;
use zbus::{dbus_interface, fdo};

use crate::{
    fancontrol::FanRuntimeHandle,
    led::LedRuntimeHandle,
    profiles::{LedDeviceInfo, Profile, PROFILE_DIR},
    util,
};

pub struct ProfileInterface {
    pub fan_handles: Vec<FanRuntimeHandle>,
    pub led_handles: Vec<LedRuntimeHandle>,
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

    async fn rename_profile(&mut self, from: &str, to: &str) -> fdo::Result<Vec<String>> {
        if self.list_profiles().await?.contains(&to.to_string()) {
            Err(fdo::Error::InvalidArgs(format!(
                "File `{to}` already exists"
            )))
        } else {
            util::move_file(PROFILE_DIR, from, to).await?;

            if self.get_active_profile_name().await? == from {
                self.set_active_profile_name(to).await?;
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

    async fn get_number_of_fans(&self) -> fdo::Result<u8> {
        Ok(self.fan_handles.len() as u8)
    }

    async fn get_led_devices(&self) -> fdo::Result<String> {
        let devices: Vec<LedDeviceInfo> = self
            .led_handles
            .iter()
            .map(|handle| handle.info.clone())
            .collect();
        Ok(serde_json::to_string(&devices).unwrap())
    }

    async fn reload(&mut self) -> fdo::Result<()> {
        let Profile { fan, led } = Profile::load();

        for (idx, fan_handle) in self.fan_handles.iter().enumerate() {
            let profile = fan.get(idx).cloned().unwrap_or_default();
            fan_handle
                .profile_sender
                .send(profile)
                .await
                .map_err(|err| fdo::Error::Failed(err.to_string()))?;
        }

        for led_handle in &self.led_handles {
            let profile = led.get(&led_handle.info).cloned().unwrap_or_default();
            led_handle
                .profile_sender
                .send(profile)
                .await
                .map_err(|err| fdo::Error::Failed(err.to_string()))?;
        }

        Ok(())
    }
}
