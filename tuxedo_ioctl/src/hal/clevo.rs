use crate::{error::IoctlError, read, write};

use super::traits::{HardwareDevice, WebcamDevice};
use super::IoctlResult;

pub const MAX_FAN_SPEED: u8 = 0xff;

const PERF_PROF_QUIET: &str = "quiet";
const PERF_PROF_POWERSAVE: &str = "power_saving";
const PERF_PROF_PERFORMANCE: &str = "performance";
const PERF_PROF_ENTERTAINMENT: &str = "entertainment";

const PERF_PROFILE_MAP: [(&str, u8); 4] = [
    (PERF_PROF_QUIET, 0x00),
    (PERF_PROF_POWERSAVE, 0x01),
    (PERF_PROF_PERFORMANCE, 0x02),
    (PERF_PROF_ENTERTAINMENT, 0x03),
];

#[derive(Debug)]
pub struct ClevoHardware {
    file: std::fs::File,
    num_of_fans: u8,
}

impl ClevoHardware {
    pub fn init(file: std::fs::File) -> IoctlResult<Self> {
        if read::cl::hw_check(&file)? == 1 {
            let mut this = Self {
                file,
                num_of_fans: 0,
            };

            // Only show actually available fans
            while this.get_fan_temperature(this.num_of_fans).is_ok() {
                this.num_of_fans += 1;
            }

            Ok(this)
        } else {
            Err(IoctlError::DevNotAvailable)
        }
    }
}

impl HardwareDevice for ClevoHardware {
    fn device_interface_id_str(&self) -> IoctlResult<String> {
        read::cl::hw_interface_id(&self.file)
    }

    fn device_model_id_str(&self) -> IoctlResult<String> {
        Err(IoctlError::NotAvailable)
    }

    fn set_enable_mode_set(&self, _enabled: bool) -> IoctlResult<()> {
        // Not implemented for clevo for now...
        Ok(())
    }

    fn get_number_fans(&self) -> u8 {
        self.num_of_fans
    }

    fn set_fans_auto(&self) -> IoctlResult<()> {
        write::cl::fan_auto(&self.file, 0xF)
    }

    fn set_fan_speed_percent(&self, fan: u8, fan_speed_percent: u8) -> IoctlResult<()> {
        let mut fan_speed_raw: [u8; 3] = [0; 3];

        let fan_speed_percent = fan_speed_percent.clamp(0, 100);

        for (i, fan_speed) in fan_speed_raw.iter_mut().enumerate() {
            let selected_fan = i as u8;
            if selected_fan == fan {
                *fan_speed = (fan_speed_percent as f64 * 0xFF as f64 / 100.0).round() as u8;
            } else {
                *fan_speed = self.read_fanspeed_raw(selected_fan)?;
            }
        }

        let mut argument: i32 = fan_speed_raw[0] as i32;
        argument |= (fan_speed_raw[1] as i32) << 0x08;
        argument |= (fan_speed_raw[2] as i32) << 0x10;
        write::cl::fan_speed(&self.file, argument)?;
        Ok(())
    }

    fn get_fan_speed_percent(&self, fan: u8) -> IoctlResult<u8> {
        let fan_speed_raw = self.read_fanspeed_raw(fan)?;
        Ok(((fan_speed_raw as f64 / MAX_FAN_SPEED as f64) * 100.0).round() as u8)
    }

    fn get_fan_temperature(&self, fan: u8) -> IoctlResult<u8> {
        let fan_info_raw = self.read_faninfo_raw(fan)?;

        // Explicitly use temp2 since it's more consistently implemented
        // int fanTemp1 = (int8_t) ((fanInfo >> 0x08) & 0xff);
        let fan_temp_2: u8 = ((fan_info_raw >> 0x10) & 0xff).try_into().unwrap();

        // If a fan is not available a low value is read out
        if fan_temp_2 <= 1 {
            Err(IoctlError::DevNotAvailable)
        } else {
            Ok(fan_temp_2)
        }
    }

    fn get_fans_min_speed(&self) -> IoctlResult<u8> {
        Ok(20)
    }

    fn get_fans_off_available(&self) -> IoctlResult<bool> {
        Ok(true)
    }

    fn get_available_odm_performance_profiles(&self) -> IoctlResult<Vec<String>> {
        Ok(vec![
            PERF_PROF_QUIET.into(),
            PERF_PROF_POWERSAVE.into(),
            PERF_PROF_ENTERTAINMENT.into(),
            PERF_PROF_PERFORMANCE.into(),
        ])
    }

    fn set_odm_performance_profile(&self, performance_profile: &str) -> IoctlResult<()> {
        if let Some((_, id)) = PERF_PROFILE_MAP
            .iter()
            .find(|(name, _)| name == &performance_profile)
        {
            write::cl::perf_profile(&self.file, *id as i32)
        } else {
            Err(IoctlError::InvalidArgs)
        }
    }

    fn get_default_odm_performance_profile(&self) -> IoctlResult<String> {
        Ok(PERF_PROF_PERFORMANCE.to_owned())
    }
}

impl WebcamDevice for ClevoHardware {
    fn set_webcam(&self, status: bool) -> IoctlResult<()> {
        write::cl::webcam_sw(&self.file, i32::from(status))
    }

    fn get_webcam(&self) -> IoctlResult<bool> {
        read::cl::webcam_sw(&self.file).map(|val| val != 0)
    }
}

// Helper methods
impl ClevoHardware {
    fn read_fanspeed_raw(&self, fan: u8) -> Result<u8, IoctlError> {
        self.read_faninfo_raw(fan)
            .map(|value| (value & 0xFF).try_into().unwrap())
    }

    fn read_faninfo_raw(&self, fan: u8) -> Result<i32, IoctlError> {
        match fan {
            0 => read::cl::fan_info_0(&self.file),
            1 => read::cl::fan_info_1(&self.file),
            2 => read::cl::fan_info_2(&self.file),
            _ => Err(IoctlError::DevNotAvailable),
        }
    }
}
