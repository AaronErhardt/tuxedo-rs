use std::fs::File;

use crate::{config::open_device_file, error::IoctlError, read, write};

pub const MAX_FAN_SPEED: u8 = 0xff;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Fan {
    Fan1,
    Fan2,
    Fan3,
}

impl Fan {
    pub fn try_from_u8(num: u8) -> Option<Self> {
        match num {
            0 => Some(Self::Fan1),
            1 => Some(Self::Fan2),
            2 => Some(Self::Fan3),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PerformanceProfile {
    Quiet,
    Powersave,
    Entertainment,
    Performance,
}

impl Default for PerformanceProfile {
    fn default() -> Self {
        Self::Performance
    }
}

impl PerformanceProfile {
    fn as_clevo_arg(&self) -> u32 {
        match self {
            PerformanceProfile::Quiet => 0x00,
            PerformanceProfile::Powersave => 0x01,
            PerformanceProfile::Entertainment => 0x02,
            PerformanceProfile::Performance => 0x03,
        }
    }
}

#[derive(Debug)]
enum Hw {
    Clevo,
    Uniwill,
}

#[derive(Debug)]
pub struct IoInterface {
    file: File,
    hw: Hw,
}

impl IoInterface {
    pub fn new() -> Result<Self, std::io::Error> {
        let file = open_device_file()?;

        if let Ok(value) = read::cl_webcam_sw(&file) {
            if value == 1 {
                return Ok(Self {
                    file,
                    hw: Hw::Clevo,
                });
            }
        }
        if let Ok(value) = read::hwcheck_uw(&file) {
            if value == 1 {
                return Ok(Self {
                    file,
                    hw: Hw::Uniwill,
                });
            }
        }
        todo!("Platform not supported yet");
    }

    fn set_fan_speed_percent_clevo(
        &mut self,
        fan: Fan,
        fan_speed_percent: u8,
    ) -> Result<(), IoctlError> {
        let mut fan_speed_raw: [u8; 3] = [0; 3];

        let fan_speed_percent = fan_speed_percent.clamp(0, 100);

        for (i, fan_speed) in fan_speed_raw.iter_mut().enumerate() {
            let selected_fan = Fan::try_from_u8(i as u8).unwrap();
            if selected_fan == fan {
                *fan_speed =
                    (fan_speed_percent as f64 * MAX_FAN_SPEED as f64 / 100.0).round() as u8;
            } else {
                *fan_speed = self.read_fanspeed_raw(fan)?;
            }
        }

        let mut argument: u32 = fan_speed_raw[0] as u32;
        argument |= (fan_speed_raw[1] as u32) << 0x08;
        argument |= (fan_speed_raw[2] as u32) << 0x10;
        write::cl_fanspeed(&self.file, argument)?;
        Ok(())
    }

    fn set_fan_speed_percent_uniwill(
        &mut self,
        fan: Fan,
        fan_speed_percent: u8,
    ) -> Result<(), IoctlError> {
        let fan_speed_raw =
            (fan_speed_percent as f64 * MAX_FAN_SPEED as f64 / 100.0).round() as u32;
        match fan {
            Fan::Fan1 => write::uw_fanspeed(&self.file, fan_speed_raw),
            Fan::Fan2 => write::uw_fanspeed2(&self.file, fan_speed_raw),
            Fan::Fan3 => Err(IoctlError::DevNotAvailable),
        }
    }

    /// Set the fan speed in percent from 0 to 100.
    /// Values above 100 will be clamped to 100.
    pub fn set_fan_speed_percent(
        &mut self,
        fan: Fan,
        fan_speed_percent: u8,
    ) -> Result<(), IoctlError> {
        match self.hw {
            Hw::Clevo => self.set_fan_speed_percent_clevo(fan, fan_speed_percent)?,
            Hw::Uniwill => self.set_fan_speed_percent_uniwill(fan, fan_speed_percent)?,
        }
        Ok(())
    }

    pub fn get_fan_speed_percent(&self, fan: Fan) -> Result<u8, IoctlError> {
        let fan_speed_raw = self.read_fanspeed_raw(fan)?;
        Ok(((fan_speed_raw as f64 / MAX_FAN_SPEED as f64) * 100.0).round() as u8)
    }

    fn read_fanspeed_raw(&self, fan: Fan) -> Result<u8, IoctlError> {
        self.read_faninfo_raw(fan)
            .map(|value| (value & 0xFF).try_into().unwrap())
    }

    fn read_faninfo_raw_1(&self) -> Result<u32, IoctlError> {
        match self.hw {
            Hw::Clevo => read::cl_faninfo1(&self.file),
            Hw::Uniwill => read::uw_fan_temp(&self.file),
        }
    }

    fn read_faninfo_raw_2(&self) -> Result<u32, IoctlError> {
        match self.hw {
            Hw::Clevo => read::cl_faninfo2(&self.file),
            Hw::Uniwill => read::uw_fan_temp2(&self.file),
        }
    }
    fn read_faninfo_raw_3(&self) -> Result<u32, IoctlError> {
        match self.hw {
            Hw::Clevo => read::cl_faninfo3(&self.file),
            Hw::Uniwill => Err(IoctlError::DevNotAvailable),
        }
    }

    fn read_faninfo_raw(&self, fan: Fan) -> Result<u32, IoctlError> {
        match fan {
            Fan::Fan1 => self.read_faninfo_raw_1(),
            Fan::Fan2 => self.read_faninfo_raw_2(),
            Fan::Fan3 => self.read_faninfo_raw_3(),
        }
    }

    pub fn set_fans_auto(&self) -> Result<(), IoctlError> {
        write::cl_fanauto(&self.file, 0xF)
    }

    // I'm not sure if that works though...
    pub fn set_fans_manual(&self) -> Result<(), IoctlError> {
        write::cl_fanauto(&self.file, 0x0)
    }

    pub fn set_web_cam_enabled(&self, status: bool) -> Result<(), IoctlError> {
        write::cl_webcam_sw(&self.file, u32::from(status))
    }

    pub fn get_web_cam_enabled(&self) -> Result<bool, IoctlError> {
        read::cl_webcam_sw(&self.file).map(|val| val != 0)
    }

    pub fn set_performance_profile(&self, profile: PerformanceProfile) -> Result<(), IoctlError> {
        write::cl_perf_profile(&self.file, profile.as_clevo_arg())
    }

    pub fn get_fan_temperature(&self, fan: Fan) -> Result<u8, IoctlError> {
        let fan_info_raw = self.read_faninfo_raw(fan)?;

        // Explicitly use temp2 since it's more consistently implemented
        //int fanTemp1 = (int8_t) ((fanInfo >> 0x08) & 0xff);
        // let fan_temp_2: u8 = ((fan_info_raw >> 0x10) & 0xff).try_into().unwrap();

        // If a fan is not available a low value is read out
        if fan_info_raw <= 1 {
            Err(IoctlError::DevNotAvailable)
        } else {
            Ok(fan_info_raw as u8)
        }
    }
}

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
