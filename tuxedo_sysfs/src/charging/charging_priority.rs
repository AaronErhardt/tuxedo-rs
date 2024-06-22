use std::io;

use crate::sysfs_util::{r_file, read_to_string, read_to_string_list, rw_file, write_string};

use super::ChargingPriority;

const CHARGING_PRIORITY_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_prio";
const CHARGING_PRIORITIES_AVAILABLE_PATH: &str =
    "/sys/devices/platform/tuxedo_keyboard/charging_profile/charging_prios_available";

impl ChargingPriority {
    pub async fn new() -> Result<Option<Self>, io::Error> {
        let mut available_charging_priorities_file =
            match r_file(CHARGING_PRIORITIES_AVAILABLE_PATH).await {
                Ok(f) => f,
                Err(_) => return Ok(None),
            };
        let priorities = read_to_string_list(&mut available_charging_priorities_file).await?;

        let priority_file = rw_file(CHARGING_PRIORITY_PATH).await?;

        Ok(Some(ChargingPriority {
            available_charging_priorities: priorities,
            charging_priority_file: priority_file,
        }))
    }

    pub async fn get_charging_priority(&mut self) -> Result<String, io::Error> {
        Ok(read_to_string(&mut self.charging_priority_file)
            .await?
            .trim()
            .to_owned())
    }

    /// charge_battery, performance
    /// src/ng-app/app/charging-settings/charging-settings.component.ts
    /// in TCC
    /// / src/uniwill_keyboard.h in tuxedo-keyboard
    pub async fn set_charging_priority(&mut self, priority: String) -> Result<(), io::Error> {
        write_string(&mut self.charging_priority_file, priority).await
    }
}
