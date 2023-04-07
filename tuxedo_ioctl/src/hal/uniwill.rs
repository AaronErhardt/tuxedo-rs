use crate::{error::IoctlError, read, write};

use super::traits::{HardwareDevice, TdpDevice};
use super::IoctlResult;

const MAX_FAN_SPEED: u8 = 0xc8;

const PERF_PROF_BALANCED: &str = "power_save";
const PERF_PROF_ENTHUSIAST: &str = "enthusiast";
const PERF_PROF_OVERBOOST: &str = "overboost";

const PERF_PROFILE_MAP: [(&str, u8); 3] = [
    (PERF_PROF_BALANCED, 0x01),
    (PERF_PROF_ENTHUSIAST, 0x02),
    (PERF_PROF_OVERBOOST, 0x03),
];

#[derive(Debug)]
pub struct UniwillHardware {
    file: std::fs::File,
    num_of_fans: u8,
}

impl UniwillHardware {
    pub fn init(file: std::fs::File) -> IoctlResult<Self> {
        if read::uw::hw_check(&file)? == 1 {
            Ok(Self {
                file,
                num_of_fans: 2,
            })
        } else {
            Err(IoctlError::DevNotAvailable)
        }
    }
}

impl HardwareDevice for UniwillHardware {
    fn device_interface_id_str(&self) -> IoctlResult<String> {
        read::uw::hw_interface_id(&self.file)
    }

    fn device_model_id_str(&self) -> IoctlResult<String> {
        read::uw::model_id(&self.file).map(|id| id.to_string())
    }

    fn set_enable_mode_set(&self, enabled: bool) -> IoctlResult<()> {
        write::uw::mode_enable(&self.file, i32::from(enabled))
    }

    fn get_number_fans(&self) -> u8 {
        self.num_of_fans
    }

    fn set_fans_auto(&self) -> IoctlResult<()> {
        write::uw::fan_auto(&self.file, 0)
    }

    fn set_fan_speed_percent(&self, fan: u8, fan_speed_percent: u8) -> IoctlResult<()> {
        let fan_speed_raw =
            (MAX_FAN_SPEED as f64 * fan_speed_percent as f64 / 100.0).round() as i32;

        match fan {
            0 => write::uw::fan_speed_0(&self.file, fan_speed_raw),
            1 => write::uw::fan_speed_1(&self.file, fan_speed_raw),
            _ => Err(IoctlError::DevNotAvailable),
        }
    }

    fn get_fan_speed_percent(&self, fan: u8) -> IoctlResult<u8> {
        let fan_speed_raw = match fan {
            0 => read::uw::fan_speed_0(&self.file),
            1 => read::uw::fan_speed_1(&self.file),
            _ => Err(IoctlError::DevNotAvailable),
        }?;

        Ok((fan_speed_raw as f64 * 100.0 / MAX_FAN_SPEED as f64).round() as u8)
    }

    fn get_fan_temperature(&self, fan: u8) -> IoctlResult<u8> {
        let temp = match fan {
            0 => read::uw::fan_temp_0(&self.file),
            1 => read::uw::fan_temp_1(&self.file),
            _ => Err(IoctlError::DevNotAvailable),
        }?;

        // Also use known set value (0x00) from tccwmi to detect no temp/fan
        if temp == 0 {
            Err(IoctlError::DevNotAvailable)
        } else {
            Ok(temp as u8)
        }
    }

    fn get_fans_min_speed(&self) -> IoctlResult<u8> {
        let speed = read::uw::fans_min_speed(&self.file)?;
        Ok(u8::try_from(speed).unwrap_or_default())
    }

    fn get_fans_off_available(&self) -> IoctlResult<bool> {
        read::uw::fans_off_available(&self.file).map(|res| res == 1)
    }

    fn get_available_odm_performance_profiles(&self) -> IoctlResult<Vec<String>> {
        let available_profs = read::uw::profs_available(&self.file)?;
        Ok(match available_profs {
            2 => {
                vec![PERF_PROF_BALANCED.into(), PERF_PROF_ENTHUSIAST.into()]
            }
            3 => {
                vec![
                    PERF_PROF_BALANCED.into(),
                    PERF_PROF_ENTHUSIAST.into(),
                    PERF_PROF_OVERBOOST.into(),
                ]
            }
            _ => {
                return Err(IoctlError::DevNotAvailable);
            }
        })
    }

    fn set_odm_performance_profile(&self, performance_profile: &str) -> IoctlResult<()> {
        if let Some((_, id)) = PERF_PROFILE_MAP
            .iter()
            .find(|(name, _)| name == &performance_profile)
        {
            write::uw::perf_profile(&self.file, *id as i32)
        } else {
            Err(IoctlError::InvalidArgs)
        }
    }

    fn get_default_odm_performance_profile(&self) -> IoctlResult<String> {
        let available_profiles = read::uw::profs_available(&self.file)?;
        if available_profiles > 0 {
            let nr_tdps = self.get_number_tdps()?;
            Ok(if nr_tdps > 0 {
                // LEDs only case (default to LEDs off)
                PERF_PROF_OVERBOOST
            } else {
                PERF_PROF_ENTHUSIAST
            }
            .to_owned())
        } else {
            Err(IoctlError::DevNotAvailable)
        }
    }
}

impl TdpDevice for UniwillHardware {
    fn get_number_tdps(&self) -> IoctlResult<u8> {
        // Check return status of getters to figure out how many
        // TDPs are configurable
        for i in (0..=2).rev() {
            if let Ok(result) = self.get_tdp(i) {
                if result >= 0 {
                    return Ok(i + 1);
                }
            }
        }

        Err(IoctlError::DevNotAvailable)
    }

    fn get_tdp_descriptors(&self) -> IoctlResult<Vec<String>> {
        let mut tdp_descriptors = Vec::new();
        let tdps = self.get_number_tdps()?;

        if tdps >= 1 {
            tdp_descriptors.push("pl1".to_owned());
        }
        if tdps >= 2 {
            tdp_descriptors.push("pl2".to_owned());
        }
        if tdps >= 3 {
            tdp_descriptors.push("pl4".to_owned());
        }

        Ok(tdp_descriptors)
    }

    fn get_tdp_min(&self, tdp_index: u8) -> IoctlResult<i32> {
        match tdp_index {
            0 => read::uw::tdp_min_0(&self.file),
            1 => read::uw::tdp_min_1(&self.file),
            2 => read::uw::tdp_min_2(&self.file),
            _ => Err(IoctlError::DevNotAvailable),
        }
    }

    fn get_tdp_max(&self, tdp_index: u8) -> IoctlResult<i32> {
        match tdp_index {
            0 => read::uw::tdp_max_0(&self.file),
            1 => read::uw::tdp_max_1(&self.file),
            2 => read::uw::tdp_max_2(&self.file),
            _ => Err(IoctlError::DevNotAvailable),
        }
    }

    fn set_tdp(&self, tdp_index: u8, tdp_value: i32) -> IoctlResult<()> {
        match tdp_index {
            0 => write::uw::tdp_0(&self.file, tdp_value),
            1 => write::uw::tdp_1(&self.file, tdp_value),
            2 => write::uw::tdp_2(&self.file, tdp_value),
            _ => Err(IoctlError::DevNotAvailable),
        }
    }

    fn get_tdp(&self, tdp_index: u8) -> IoctlResult<i32> {
        match tdp_index {
            0 => read::uw::tdp_0(&self.file),
            1 => read::uw::tdp_1(&self.file),
            2 => read::uw::tdp_2(&self.file),
            _ => Err(IoctlError::DevNotAvailable),
        }
    }
}
