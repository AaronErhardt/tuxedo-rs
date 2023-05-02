use std::{collections::HashMap, path::Component, path::Path};

use crate::{
    backlight::BacklightProfile, fancontrol::profile::FanProfile, performance::PerformanceProfile,
};
use tailor_api::{ColorProfile, LedDeviceInfo, LedProfile, ProfileInfo};
use zbus::fdo;

use super::util;

pub const PROFILE_DIR: &str = "/etc/tailord/profiles/";
pub const KEYBOARD_DIR: &str = "/etc/tailord/keyboard/";
pub const FAN_DIR: &str = "/etc/tailord/fan/";
pub const ACTIVE_PROFILE_PATH: &str = "/etc/tailord/active_profile.json";

fn init_paths() {
    [PROFILE_DIR, KEYBOARD_DIR, FAN_DIR]
        .into_iter()
        .for_each(|dir| {
            std::fs::create_dir_all(dir).ok();
        })
}

fn init_active_profile() {
    if Path::new(ACTIVE_PROFILE_PATH).exists() {
        return;
    }
    tracing::debug!("Initialising active profile.");
    let default_profile = Path::join(Path::new(PROFILE_DIR), "default.json");
    std::os::unix::fs::symlink(default_profile.as_path(), Path::new(ACTIVE_PROFILE_PATH)).ok();
}

fn led_profile_path(name: &str) -> fdo::Result<String> {
    util::normalize_json_path(KEYBOARD_DIR, name)
}

fn fan_path(name: &str) -> fdo::Result<String> {
    util::normalize_json_path(FAN_DIR, name)
}

fn load_led_profile(name: &str) -> fdo::Result<ColorProfile> {
    let color_profile_data = std::fs::read(led_profile_path(name)?)
        .map_err(|err| fdo::Error::IOError(err.to_string()))?;
    serde_json::from_slice(&color_profile_data)
        .map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))
}

fn load_fan_profile(name: &str) -> fdo::Result<FanProfile> {
    FanProfile::load_config(fan_path(name)?)
}

#[derive(Debug)]
pub struct Profile {
    pub fans: Vec<FanProfile>,
    pub leds: HashMap<LedDeviceInfo, ColorProfile>,
    pub performance_profile: Option<PerformanceProfile>,
    pub brightness: BacklightProfile,
}

impl Profile {
    pub fn load() -> Self {
        init_paths();
        init_active_profile();

        let profile_info = Self::get_active_profile_info().unwrap_or_else(|err| {
            tracing::warn!("Failed to load active profile at `{ACTIVE_PROFILE_PATH}`: {err:?}");
            ProfileInfo::default()
        });
        tracing::info!("Loaded profile at `{ACTIVE_PROFILE_PATH}`: {profile_info:?}");

        let mut led = HashMap::new();
        for data in profile_info.leds {
            let LedProfile {
                device_name,
                function,
                profile,
            } = data;
            let info = LedDeviceInfo {
                device_name,
                function,
            };
            let profile = match load_led_profile(&profile) {
                Ok(keyboard) => keyboard,
                Err(err) => {
                    tracing::warn!(
                        "Failed to load keyboard color profile called `{}`: `{}`",
                        info.device_id(),
                        err.to_string(),
                    );
                    ColorProfile::default()
                }
            };
            led.insert(info, profile);
        }

        let fan = profile_info
            .fans
            .iter()
            .map(|fan_profile| match load_fan_profile(fan_profile) {
                Ok(fan) => fan,
                Err(err) => {
                    tracing::error!(
                        "Failed to load fan color profile called `{}`: `{}`",
                        fan_profile,
                        err.to_string(),
                    );
                    FanProfile::default()
                }
            })
            .collect();

        let performance_profile = profile_info
            .performance_profile
            .map(PerformanceProfile::new);

        let brightness = BacklightProfile::new(profile_info.brightness);

        Self {
            fans: fan,
            leds: led,
            performance_profile,
            brightness,
        }
    }

    pub async fn set_active_profile_name(name: &str) -> fdo::Result<()> {
        std::fs::metadata(util::normalize_json_path(PROFILE_DIR, name)?)
            .map_err(|_| fdo::Error::FileNotFound(format!("Couldn't find profile `{name}`")))?;

        std::fs::remove_file(ACTIVE_PROFILE_PATH)
            .map_err(|err| fdo::Error::IOError(err.to_string()))?;
        std::os::unix::fs::symlink(
            util::normalize_json_path("profiles", name)?,
            ACTIVE_PROFILE_PATH,
        )
        .map_err(|err| fdo::Error::IOError(err.to_string()))
    }

    pub async fn get_active_profile_name() -> fdo::Result<String> {
        let link = std::fs::read_link(ACTIVE_PROFILE_PATH)
            .map_err(|err| fdo::Error::IOError(err.to_string()))?;
        let components: Vec<Component> = link.components().collect();
        if components.len() > 0 {
            if let Component::Normal(name) = components.last().unwrap() {
                if let Some(name) = name.to_str() {
                    return Ok(name.trim_end_matches(".json").to_string());
                }
            }
        }

        Err(fdo::Error::InvalidFileContent(
            "The active profile isn't set correctly".to_string(),
        ))
    }

    pub fn get_active_profile_info() -> fdo::Result<ProfileInfo> {
        let data = std::fs::read(ACTIVE_PROFILE_PATH)
            .map_err(|err| fdo::Error::IOError(err.to_string()))?;
        serde_json::from_slice(&data).map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))
    }
}
