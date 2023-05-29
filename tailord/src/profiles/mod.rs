mod fs_util;

use std::{collections::HashMap, path::Component, path::Path};

use crate::{
    fancontrol::profile::FanProfile, performance::PerformanceProfile,
    power_supply::is_power_connected,
};
use tailor_api::{ColorProfile, LedDeviceInfo, LedProfile, ProfileInfo};
use zbus::fdo;

use super::util;

pub const PROFILE_DIR: &str = "/etc/tailord/profiles/";
pub const LED_DIR: &str = "/etc/tailord/led/";
pub const FAN_DIR: &str = "/etc/tailord/fan/";

const ACTIVE_POWER_PROFILE_PATH: &str = "/etc/tailord/power_profile.json";
const ACTIVE_BATTERY_PROFILE_PATH: &str = "/etc/tailord/battery_profile.json";

/// Legacy value, was renamed to led in version 0.3
const OLD_KEYBOARD_DIR: &str = "/etc/tailord/keyboard/";

// Legacy value, was split into power and battery profiles
const OLD_ACTIVE_PROFILE_PATH: &str = "/etc/tailord/active_profile.json";

fn init_paths() {
    // If the old path exist, rename it.
    fs_util::rename(OLD_KEYBOARD_DIR, LED_DIR).ok();

    [PROFILE_DIR, LED_DIR, FAN_DIR].into_iter().for_each(|dir| {
        fs_util::create_dir_all(dir).ok();
    })
}

fn init_active_profiles() {
    fn init_active_profile(name: &str, path: &str) {
        if !Path::new(path).exists() {
            tracing::debug!("Initializing active {} profile.", name);
            let profile_dir = Path::new(PROFILE_DIR);

            // Try carrying the configuration over from older releases
            let target_profile = if let Ok(link) = fs_util::read_link(OLD_ACTIVE_PROFILE_PATH) {
                profile_dir.join(link.components().last().unwrap())
            } else {
                // Use default profile is no link exists yet
                profile_dir.join("default.json")
            };
            fs_util::symlink(target_profile.as_path(), Path::new(path)).ok();
        }
    }
    init_active_profile("battery", ACTIVE_BATTERY_PROFILE_PATH);
    init_active_profile("power", ACTIVE_POWER_PROFILE_PATH);

    // Try removing the old profile link if it still exists
    fs_util::remove_file(OLD_ACTIVE_PROFILE_PATH).ok();
}

fn led_profile_path(name: &str) -> fdo::Result<String> {
    util::normalize_json_path(LED_DIR, name)
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
}

impl Profile {
    pub fn load() -> Self {
        init_paths();
        init_active_profiles();

        let profile_info = Self::get_active_profile_info().unwrap_or_else(|err| {
            tracing::warn!("Failed to load active profile: {err:?}");
            ProfileInfo::default()
        });
        tracing::info!("Loaded profile: {profile_info:?}");

        let mut leds = HashMap::new();
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
                Ok(led_profile) => led_profile,
                Err(err) => {
                    tracing::warn!(
                        "Failed to load led color profile called `{}`: `{}`",
                        info.device_id(),
                        err.to_string(),
                    );
                    ColorProfile::default()
                }
            };
            leds.insert(info, profile);
        }

        let fans = profile_info
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

        Self {
            fans,
            leds,
            performance_profile,
        }
    }

    fn active_profile_path() -> &'static str {
        if is_power_connected() {
            ACTIVE_POWER_PROFILE_PATH
        } else {
            ACTIVE_BATTERY_PROFILE_PATH
        }
    }

    pub async fn set_active_battery_profile_name(name: &str) -> fdo::Result<()> {
        Self::set_active_profile_name_link(name, ACTIVE_BATTERY_PROFILE_PATH).await
    }

    pub async fn set_active_power_profile_name(name: &str) -> fdo::Result<()> {
        Self::set_active_profile_name_link(name, ACTIVE_POWER_PROFILE_PATH).await
    }

    pub async fn set_active_profile_name(name: &str) -> fdo::Result<()> {
        Self::set_active_profile_name_link(name, Self::active_profile_path()).await
    }

    async fn set_active_profile_name_link(name: &str, link: &str) -> fdo::Result<()> {
        if fs_util::exists(util::normalize_json_path(PROFILE_DIR, name)?) {
            fs_util::remove_file(link)?;
            fs_util::symlink(util::normalize_json_path("profiles", name)?, link)?;
            Ok(())
        } else {
            Err(fdo::Error::FileNotFound(format!(
                "Couldn't find profile `{name}`"
            )))
        }
    }

    pub async fn get_active_profile_name() -> fdo::Result<String> {
        Self::get_active_profile_name_from_link(Self::active_profile_path()).await
    }

    async fn get_active_profile_name_from_link(link: &str) -> fdo::Result<String> {
        let link = fs_util::read_link(link)?;
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
        let data = fs_util::read(Self::active_profile_path())?;
        serde_json::from_slice(&data).map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))
    }
}
