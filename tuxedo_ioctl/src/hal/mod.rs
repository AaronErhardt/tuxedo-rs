mod clevo;
mod uniwill;

use std::{fs::File, io};

use crate::{config::open_device_file, error::IoctlError, read};

pub trait HardwareDevice: Sized {
    fn init(file: std::fs::File) -> IoctlResult<Self>;
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
    fn set_odm_performance_profile(&self, performance_profile: String) -> IoctlResult<()>;
    fn get_default_odm_performance_profile(&self) -> IoctlResult<String>;
}

pub trait WebcamDevice {
    fn set_webcam(&self, status: bool) -> IoctlResult<()>;
    fn get_webcam(&self) -> IoctlResult<bool>;
}

pub trait TdpDevice {
    fn get_number_tdps(&self) -> IoctlResult<u8>;
    fn get_tdp_descriptors(&self) -> IoctlResult<Vec<String>>;
    fn get_tdp_min(&self, tdp_index: u8) -> IoctlResult<u8>;
    fn get_tdp_max(&self, tdp_index: u8) -> IoctlResult<u8>;
    fn set_tdp(&self, tdp_index: u8, tdp_value: u8) -> IoctlResult<()>;
    fn get_tdp(&self, tdp_index: u8) -> IoctlResult<u8>;
}

type IoctlResult<T> = Result<T, IoctlError>;

#[derive(Debug)]
pub enum HwManufacturer {
    Clevo,
    Uniwill,
}

impl HwManufacturer {
    pub fn new(file: &std::fs::File) -> Result<Self, io::Error> {
        if read::cl::hw_check(&file) == Ok(1) {
            Ok(HwManufacturer::Clevo)
        } else if read::uw::hw_check(&file) == Ok(1) {
            Ok(HwManufacturer::Uniwill)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No Tuxedo driver available",
            ))
        }
    }
}

#[derive(Debug)]
pub struct IoInterface {
    file: File,
    manufacturer: HwManufacturer,
}

impl IoInterface {
    pub fn new() -> Result<Self, std::io::Error> {
        let file = open_device_file()?;

        let manufacturer = HwManufacturer::new(&file)?;

        Ok(Self { file, manufacturer })
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interface() {
        sudo::escalate_if_needed().unwrap();

        let mut io = IoInterface::new().unwrap();

        // Check webcam
        io.set_web_cam_enabled(false).unwrap();
        assert_eq!(io.get_web_cam_enabled().unwrap(), false);

        io.set_web_cam_enabled(true).unwrap();
        assert_eq!(io.get_web_cam_enabled().unwrap(), true);

        // Set performance profile
        io.set_performance_profile(PerformanceProfile::Quiet)
            .unwrap();

        // Get temperatures
        assert!(20 < io.get_fan_temperature(Fan::Fan1).unwrap());
        assert_eq!(
            io.get_fan_temperature(Fan::Fan2).unwrap_err(),
            IoctlError::DevNotAvailable
        );

        // Check fans
        io.set_fan_speed_percent(Fan::Fan1, 100).unwrap();
        assert_eq!(io.get_fan_speed_percent(Fan::Fan1).unwrap(), 100);

        io.set_fan_speed_percent(Fan::Fan1, 50).unwrap();
        assert_eq!(io.get_fan_speed_percent(Fan::Fan1).unwrap(), 50);

        io.set_fan_speed_percent(Fan::Fan1, 10).unwrap();
        assert_eq!(io.get_fan_speed_percent(Fan::Fan1).unwrap(), 10);

        io.set_fan_speed_percent(Fan::Fan1, 100).unwrap();
        assert_eq!(io.get_fan_speed_percent(Fan::Fan1).unwrap(), 100);
    }
}
 */
