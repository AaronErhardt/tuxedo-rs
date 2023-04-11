use tailor_api::{LedDeviceInfo, ProfileInfo};
use zbus::{dbus_interface, fdo};

use crate::{
    fancontrol::FanRuntimeHandle,
    led::LedRuntimeHandle,
    performance::PerformanceProfileRuntimeHandle,
    profiles::{Profile, PROFILE_DIR},
    util,
};

pub struct ProfileInterface {
    pub fan_handles: Vec<FanRuntimeHandle>,
    pub led_handles: Vec<LedRuntimeHandle>,
    pub performance_profile_handle: Option<PerformanceProfileRuntimeHandle>,
}

impl ProfileInterface {
    fn performance_profile_handle(&self) -> fdo::Result<&PerformanceProfileRuntimeHandle> {
        self.performance_profile_handle
            .as_ref()
            .ok_or(fdo::Error::Failed(
                "No performance profile handler available".to_string(),
            ))
    }

    fn performance_profile_handle_mut(
        &mut self,
    ) -> fdo::Result<&mut PerformanceProfileRuntimeHandle> {
        self.performance_profile_handle
            .as_mut()
            .ok_or(fdo::Error::Failed(
                "No performance profile handler available".to_string(),
            ))
    }
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

    async fn set_active_performance_profile_name(&mut self, name: &str) -> fdo::Result<()> {
        self.performance_profile_handle()?
            .profile_sender
            .send(name.to_string())
            .await
            .map_err(|err| fdo::Error::Failed(err.to_string()))?;
        Ok(self
            .performance_profile_handle_mut()?
            .set_active_performance_profile(name))
    }

    async fn get_active_performance_profile_name(&self) -> fdo::Result<String> {
        Ok(self
            .performance_profile_handle()?
            .get_active_performance_profile()
            .to_string())
    }

    async fn get_available_performance_profile_names(&self) -> fdo::Result<Vec<String>> {
        self.performance_profile_handle()?
            .get_availables_performance_profiles()
            .map_err(|err| fdo::Error::Failed(err.to_string()))
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
        let Profile {
            fans,
            leds,
            performance_profile,
        } = Profile::load();

        for (idx, fan_handle) in self.fan_handles.iter().enumerate() {
            let profile = fans.get(idx).cloned().unwrap_or_default();
            fan_handle
                .profile_sender
                .send(profile)
                .await
                .map_err(|err| fdo::Error::Failed(err.to_string()))?;
        }

        for led_handle in &self.led_handles {
            let profile = leds.get(&led_handle.info).cloned().unwrap_or_default();
            led_handle
                .profile_sender
                .send(profile)
                .await
                .map_err(|err| fdo::Error::Failed(err.to_string()))?;
        }

        if let Some(perf_handle) = &self.performance_profile_handle {
            if let Some(performance_profile) = performance_profile {
                perf_handle
                    .profile_sender
                    .send(performance_profile.to_string())
                    .await
                    .map_err(|err| fdo::Error::Failed(err.to_string()))?;
            }
        }

        Ok(())
    }
}
