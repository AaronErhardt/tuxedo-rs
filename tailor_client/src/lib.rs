#![deny(unreachable_pub, rust_2018_idioms)]

mod dbus;
mod error;

pub use error::ClientError;
use tailor_api::{Color, ColorProfile, FanProfilePoint, ProfileInfo};
use zbus::Connection;

pub type ClientResult<T> = Result<T, ClientError>;

#[derive(Debug, Clone)]
pub struct TailorConnection<'a> {
    profiles: dbus::ProfilesProxy<'a>,
    keyboard: dbus::KeyboardProxy<'a>,
    fan: dbus::FanProxy<'a>,
}

impl<'a> TailorConnection<'a> {
    pub async fn new() -> Result<TailorConnection<'a>, zbus::Error> {
        let connection = Connection::system().await?;

        let profiles = dbus::ProfilesProxy::new(&connection).await?;
        let keyboard = dbus::KeyboardProxy::new(&connection).await?;
        let fan = dbus::FanProxy::new(&connection).await?;

        Ok(Self {
            profiles,
            keyboard,
            fan,
        })
    }
}

impl<'a> TailorConnection<'a> {
    pub async fn add_keyboard_profile(
        &self,
        name: &str,
        profile: &ColorProfile,
    ) -> ClientResult<()> {
        let value = serde_json::to_string(profile)?;
        Ok(self.keyboard.add_profile(name, &value).await?)
    }

    pub async fn get_keyboard_profile(&self, name: &str) -> ClientResult<ColorProfile> {
        let profile_data = self.keyboard.get_profile(name).await?;
        Ok(serde_json::from_str(&profile_data)?)
    }

    pub async fn list_keyboard_profiles(&self) -> ClientResult<Vec<String>> {
        Ok(self.keyboard.list_profiles().await?)
    }

    pub async fn copy_keyboard_profile(&self, from: &str, to: &str) -> ClientResult<()> {
        let profile = self.get_keyboard_profile(from).await?;
        self.add_keyboard_profile(to, &profile).await
    }

    pub async fn rename_keyboard_profile(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> ClientResult<Vec<String>> {
        Ok(self.keyboard.rename_profile(old_name, new_name).await?)
    }

    pub async fn remove_keyboard_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.keyboard.remove_profile(name).await?)
    }

    pub async fn override_keyboard_color(&self, color: &Color) -> ClientResult<()> {
        let value = serde_json::to_string(color)?;
        Ok(self.keyboard.override_color(&value).await?)
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

    pub async fn rename_fan_profile(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> ClientResult<Vec<String>> {
        Ok(self.fan.rename_profile(old_name, new_name).await?)
    }

    pub async fn remove_fan_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.fan.remove_profile(name).await?)
    }

    pub async fn override_fan_speed(&self, speed: u8) -> ClientResult<()> {
        Ok(self.fan.override_speed(speed).await?)
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

    pub async fn rename_global_profile(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> ClientResult<Vec<String>> {
        Ok(self.profiles.rename_profile(old_name, new_name).await?)
    }

    pub async fn remove_global_profile(&self, name: &str) -> ClientResult<()> {
        Ok(self.profiles.remove_profile(name).await?)
    }

    pub async fn get_active_global_profile_name(&self) -> ClientResult<String> {
        Ok(self.profiles.get_active_profile_name().await?)
    }

    pub async fn set_active_global_profile_name(&self, name: &str) -> ClientResult<()> {
        Ok(self.profiles.set_active_profile_name(name).await?)
    }

    pub async fn reload(&self) -> ClientResult<()> {
        Ok(self.profiles.reload().await?)
    }
}
