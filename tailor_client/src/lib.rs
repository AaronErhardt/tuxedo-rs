#![deny(unreachable_pub, rust_2018_idioms)]

mod dbus;
mod error;

pub use error::ClientError;
use tailor_api::{Color, ColorProfile, FanProfilePoint, LedDeviceInfo, ProfileInfo};
use zbus::Connection;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Clone)]
pub struct TailorConnection<'a> {
    profiles: dbus::ProfilesProxy<'a>,
    led: dbus::LedProxy<'a>,
    fan: dbus::FanProxy<'a>,
    performance: dbus::PerformanceProxy<'a>,
}

impl<'a> TailorConnection<'a> {
    pub async fn new() -> Result<TailorConnection<'a>, zbus::Error> {
        let connection = Connection::system().await?;

        let profiles = dbus::ProfilesProxy::new(&connection).await?;
        let keyboard = dbus::LedProxy::new(&connection).await?;
        let fan = dbus::FanProxy::new(&connection).await?;
        let performance = dbus::PerformanceProxy::new(&connection).await?;

        Ok(Self {
            profiles,
            led: keyboard,
            fan,
            performance,
        })
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_led_profile(&self, name: &str, profile: &ColorProfile) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.led.add_profile(name, &value).await?)
    }

    pub async fn get_led_profile(&self, name: &str) -> ClientResult<ColorProfile> {
        let profile_data = self.led.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_led_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.led.list_profiles().await?)
    }

    pub async fn copy_led_profile(&self, from: &str, to: &str) -> ClientResult<()> {
        let profile = self.get_led_profile(from).await?;
        self.add_led_profile(to, &profile).await
    }

    pub async fn rename_led_profile(&self, from: &str, to: &str) -> ClientResult<Vec<String>> {
        Ok(self.led.rename_profile(from, to).await?)
    }

    pub async fn remove_led_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.led.remove_profile(name).await?)
    }

    pub async fn override_led_colors(&self, color: &Color) -> ClientResult<()> {
        let value = serde_json::to_string(color)?;
        Ok(self.led.override_color(&value).await?)
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_fan_profile(
        &self,
        name: &str,
        profile: &[FanProfilePoint],
    ) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.fan.add_profile(name, &value).await?)
    }

    pub async fn get_fan_profile(&self, name: &str) -> ClientResult<Vec<FanProfilePoint>> {
        let profile_data = self.fan.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_fan_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.fan.list_profiles().await?)
    }

    pub async fn copy_fan_profile(&self, from: &str, to: &str) -> ClientResult<()> {
        let profile = self.get_fan_profile(from).await?;
        self.add_fan_profile(to, &profile).await
    }

    pub async fn rename_fan_profile(&self, from: &str, to: &str) -> ClientResult<Vec<String>> {
        Ok(self.fan.rename_profile(from, to).await?)
    }

    pub async fn remove_fan_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.fan.remove_profile(name).await?)
    }

    pub async fn override_fan_speed(&self, fan_idx: u8, speed: u8) -> ClientResult<()> {
        Ok(self.fan.override_speed(fan_idx, speed).await?)
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_global_profile(&self, name: &str, profile: &ProfileInfo) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.profiles.add_profile(name, &value).await?)
    }

    pub async fn get_global_profile(&self, name: &str) -> ClientResult<ProfileInfo> {
        let profile_data = self.profiles.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_global_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.profiles.list_profiles().await?)
    }

    pub async fn copy_global_profile(&self, from: &str, to: &str) -> ClientResult<()> {
        let profile = self.get_global_profile(from).await?;
        self.add_global_profile(to, &profile).await
    }

    pub async fn rename_global_profile(&self, from: &str, to: &str) -> ClientResult<Vec<String>> {
        Ok(self.profiles.rename_profile(from, to).await?)
    }

    pub async fn remove_global_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.profiles.remove_profile(name).await?)
    }

    pub async fn get_active_global_profile_name(&self) -> ClientResult<String> {
        Ok(self.profiles.get_active_profile_name().await?)
    }

    pub async fn get_available_performance_profile_names(&self) -> ClientResult<Vec<String>> {
        Ok(self
            .profiles
            .get_available_performance_profile_names()
            .await?)
    }

    pub async fn set_active_global_profile_name(&self, name: &str) -> ClientResult<()> {
        Ok(self.profiles.set_active_profile_name(name).await?)
    }

    pub async fn get_number_of_fans(&self) -> ClientResult<u8> {
        Ok(self.profiles.get_number_of_fans().await?)
    }

    pub async fn get_led_devices(&self) -> ClientResult<Vec<LedDeviceInfo>> {
        let data = self.profiles.get_led_devices().await?;
        Ok(serde_json::from_str(&data)?)
    }

    pub async fn reload(&self) -> ClientResult<()> {
        Ok(self.profiles.reload().await?)
    }
}

impl<'a> TailorConnection<'a> {
    /// Temporarily override the performance profile. Please note that this will not survive a
    /// restart as the performance profile is handled by the overall profile configuration.
    pub async fn set_profile(&self, name: &str, value: &str) -> ClientResult<()> {
        Ok(self.performance.set_profile(name, value).await?)
    }

    /// Read the current performance profile.
    pub async fn get_profile(&self, name: &str) -> ClientResult<String> {
        Ok(self.performance.get_profile(name).await?)
    }

    /// Read the list of supported performance profiles.
    pub async fn list_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.performance.list_profiles().await?)
    }
}
