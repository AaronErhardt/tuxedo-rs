use std::io;

use crate::sysfs_util::{r_file, read_to_string, read_to_string_list, rw_file};

use super::ChargingProfile;

const CHARGING_PROFILE_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profile";
const CHARGING_PROFILES_AVAILABLE_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profiles_available";

impl ChargingProfile {
    pub async fn new() -> Result<Option<Self>, io::Error> {
        let mut available_charging_profiles_file =
            match r_file(CHARGING_PROFILES_AVAILABLE_PATH).await {
                Ok(f) => f,
                Err(_) => return Ok(None),
            };
        let available_charging_profiles =
            read_to_string_list(&mut available_charging_profiles_file).await?;
        let charging_profile_file = rw_file(CHARGING_PROFILE_PATH).await?;

        Ok(Some(ChargingProfile {
            available_charging_profiles,
            charging_profile_file,
        }))
    }

    pub async fn get_charging_profile(&mut self) -> Result<String, io::Error> {
        Ok(read_to_string(&mut self.charging_profile_file)
            .await?
            .trim()
            .to_owned())
    }
}
