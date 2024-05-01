use std::io;

use crate::sysfs_util::{
    r_file, read_path_to_string_list, read_to_string, read_to_string_list, rw_file,
};

use super::ChargingProfile;

const CHARGING_PROFILE_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profile";
const CHARGING_PROFILES_AVAILABLE_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_profiles_available";
const CHARGING_PRIORITY_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_prio";
const CHARGING_PRIORITIES_AVAILABLE_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_prios_available";

impl ChargingProfile {
    // TODO: should this return Option<Self, io::Error> in case the whole charging_profile thing is unavailable?
    pub async fn new() -> Result<Self, io::Error> {
        let available_charging_profiles =
            read_path_to_string_list(CHARGING_PROFILES_AVAILABLE_PATH).await?;
        let charging_profile_file = rw_file(CHARGING_PROFILE_PATH).await?;

        let (available_charging_priorities, charging_priority_file) =
            if let Ok(mut available_charging_priorities_file) =
                r_file(CHARGING_PRIORITIES_AVAILABLE_PATH).await
            {
                let priorities =
                    read_to_string_list(&mut available_charging_priorities_file).await?;

                let priority_file = rw_file(CHARGING_PRIORITY_PATH).await?;
                (Some(priorities), Some(priority_file))
            } else {
                (None, None)
            };

        Ok(ChargingProfile {
            available_charging_profiles,
            charging_profile_file,
            available_charging_priorities,
            charging_priority_file,
        })
    }

    pub async fn get_charging_profile(&mut self) -> Result<String, io::Error> {
        Ok(read_to_string(&mut self.charging_profile_file)
            .await?
            .trim()
            .to_owned())
    }

    // TODO: should this return Result<Option> or Option<Result>?
    pub async fn get_charging_priority(&mut self) -> Result<Option<String>, io::Error> {
        if let Some(charging_priority_file) = &mut self.charging_priority_file {
            Ok(Some(
                read_to_string(charging_priority_file)
                    .await?
                    .trim()
                    .to_owned(),
            ))
        } else {
            Ok(None)
        }
    }

    // TODO: set_prio: how does this error in case prio is unavailable?
}
