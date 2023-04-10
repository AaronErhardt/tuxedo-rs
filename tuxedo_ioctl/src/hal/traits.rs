use std::fmt::Debug;

use super::IoctlResult;

pub trait HardwareDevice: Debug {
    fn device_interface_id_str(&self) -> IoctlResult<String>;
    fn device_model_id_str(&self) -> IoctlResult<String>;
    fn set_enable_mode_set(&self, enabled: bool) -> IoctlResult<()>;

    // Get the amount of available fans
    fn get_number_fans(&self) -> u8;

    fn set_fans_auto(&self) -> IoctlResult<()>;

    /// Set the fan speed in percent from 0 to 100.
    /// Values above 100 will be clamped to 100.
    fn set_fan_speed_percent(&self, fan: u8, fan_speed_percent: u8) -> IoctlResult<()>;

    /// Get the fan speed value in percent from 0 to 100.
    fn get_fan_speed_percent(&self, fan: u8) -> IoctlResult<u8>;

    /// Get the fan temperature in °C
    fn get_fan_temperature(&self, fan: u8) -> IoctlResult<u8>;

    /// Get the minimum supported speed of the fans
    fn get_fans_min_speed(&self) -> IoctlResult<u8>;
    fn get_fans_off_available(&self) -> IoctlResult<bool>;
    fn get_available_odm_performance_profiles(&self) -> IoctlResult<Vec<String>>;
    fn set_odm_performance_profile(&self, performance_profile: &str) -> IoctlResult<()>;
    fn get_default_odm_performance_profile(&self) -> IoctlResult<String>;
}

pub trait WebcamDevice: Debug {
    fn set_webcam(&self, status: bool) -> IoctlResult<()>;
    fn get_webcam(&self) -> IoctlResult<bool>;
}

pub trait TdpDevice: Debug {
    fn get_number_tdps(&self) -> IoctlResult<u8>;
    fn get_tdp_descriptors(&self) -> IoctlResult<Vec<String>>;
    fn get_tdp_min(&self, tdp_index: u8) -> IoctlResult<i32>;
    fn get_tdp_max(&self, tdp_index: u8) -> IoctlResult<i32>;
    fn set_tdp(&self, tdp_index: u8, tdp_value: i32) -> IoctlResult<()>;
    fn get_tdp(&self, tdp_index: u8) -> IoctlResult<i32>;
}
