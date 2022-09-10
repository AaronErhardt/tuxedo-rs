use crate::fancontrol::profile::FanProfile;
use tailor_api::keyboard::ColorProfile;
use zbus::fdo;

use self::util::normalize_json_path;

pub mod util;

pub const PROFILE_DIR: &str = "/etc/tailord/profiles/";
pub const KEYBOARD_DIR: &str = "/etc/tailord/keyboard/";
pub const FAN_DIR: &str = "/etc/tailord/fan/";
pub const ACTIVE_PROFILE_PATH: &str = "/etc/tailord/profiles/active_profile.json";

fn init_paths() {
    [PROFILE_DIR, KEYBOARD_DIR, FAN_DIR]
        .into_iter()
        .for_each(|dir| {
            drop(std::fs::create_dir_all(dir));
        })
}

#[derive(Debug, Default)]
pub struct Profile {
    pub fan: FanProfile,
    pub keyboard: ColorProfile,
}

#[derive(Debug, serde::Deserialize)]
pub struct ProfileInfo {
    fan: String,
    keyboard: String,
}

impl Default for ProfileInfo {
    fn default() -> Self {
        Self {
            fan: "default".to_string(),
            keyboard: "default".to_string(),
        }
    }
}

impl ProfileInfo {
    fn keyboard_path(&self) -> fdo::Result<String> {
        normalize_json_path(KEYBOARD_DIR, &self.keyboard)
    }

    fn fan_path(&self) -> fdo::Result<String> {
        normalize_json_path(FAN_DIR, &self.fan)
    }

    fn load_keyboard_profile(&self) -> fdo::Result<ColorProfile> {
        let color_profile_data = std::fs::read(&self.keyboard_path()?)
            .map_err(|err| fdo::Error::IOError(err.to_string()))?;
        serde_json::from_slice(&color_profile_data)
            .map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))
    }

    fn load_fan_profile(&self) -> fdo::Result<FanProfile> {
        FanProfile::load_config(&self.fan_path()?)
    }
}

impl Profile {
    pub fn load() -> Self {
        init_paths();

        let profile_info = if let Some(profile_info) = std::fs::read(ACTIVE_PROFILE_PATH)
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
        {
            profile_info
        } else {
            tracing::error!("Failed to load active profile at `{ACTIVE_PROFILE_PATH}`");
            ProfileInfo::default()
        };

        let keyboard = match profile_info.load_keyboard_profile() {
            Ok(keyboard) => keyboard,
            Err(err) => {
                tracing::error!(
                    "Failed to load keyboard color profile called `{}`: `{}`",
                    profile_info.keyboard,
                    err.to_string(),
                );
                ColorProfile::default()
            }
        };

        let fan = match profile_info.load_fan_profile() {
            Ok(fan) => fan,
            Err(err) => {
                tracing::error!(
                    "Failed to load fan color profile called `{}`: `{}`",
                    profile_info.fan,
                    err.to_string(),
                );
                FanProfile::default()
            }
        };

        Self { fan, keyboard }
    }

    pub fn reload() -> fdo::Result<Self> {
        let profile_info_data = std::fs::read(ACTIVE_PROFILE_PATH).map_err(zbus::Error::Io)?;
        let profile_info: ProfileInfo = serde_json::from_slice(&profile_info_data)
            .map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))?;

        let keyboard = profile_info.load_keyboard_profile()?;
        let fan = profile_info.load_fan_profile()?;

        Ok(Self { fan, keyboard })
    }
}
