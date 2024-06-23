mod charge_control;
mod charging_priority;
mod charging_profile;

/// A type that manages all sysfs files related to
/// charging profile options provided by the tuxedo_keyboard driver.
///
/// The charging profile imposes a firmware-enforced limit on the maximum charge of the
/// battery.
pub struct ChargingProfile {
    pub available_charging_profiles: Vec<String>,
    charging_profile_file: tokio_uring::fs::File,
}

/// A type that manages all sysfs files related to
/// charging priority options provided by the tuxedo_keyboard driver.
///
/// The charging priority determines whether charging speed or
/// performance is prioritized when charging over USB-C.
pub struct ChargingPriority {
    pub available_charging_priorities: Vec<String>,
    charging_priority_file: tokio_uring::fs::File,
}

/// A type that manages all sysfs files related to charging start/end thresholds.
pub struct BatteryChargeControl {
    pub name: String,
    pub available_start_thresholds: Option<Vec<u32>>,
    pub available_end_thresholds: Option<Vec<u32>>,
    /// Percentage value between 0-100,
    /// [`Self::available_start_thresholds`] lists further restrictions on accepted values.
    start_threshold_file: tokio_uring::fs::File,
    /// Percentage value between 0-100,
    /// [`Self::available_end_thresholds`] lists further restrictions on accepted values.
    end_threshold_file: tokio_uring::fs::File,
    /// Must be 'Custom' to allow for custom thresholds.
    ///
    /// Possible values listed at <https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-power> (section: /sys/class/power_supply/<supply_name>/charge_type)
    charge_type_file: tokio_uring::fs::File,
}
