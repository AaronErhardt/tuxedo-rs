use std::path::Path;

use tailor_api::FanProfilePoint;
use zbus::fdo;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct FanProfile {
    inner: Vec<FanProfilePoint>,
}

impl FanProfile {
    pub fn load_config(file_name: impl AsRef<Path>) -> fdo::Result<Self> {
        let file_name = file_name.as_ref();
        let content =
            std::fs::read(file_name).map_err(|err| fdo::Error::IOError(err.to_string()))?;
        let mut inner: Vec<FanProfilePoint> = serde_json::from_slice(&content)
            .map_err(|err| fdo::Error::InvalidFileContent(err.to_string()))?;

        if inner.is_empty() {
            return Err(fdo::Error::FileNotFound("Empty configuration".to_string()));
        }

        // Make sure the temperature is increasing with each point.
        let is_sorted = inner
            .iter()
            .try_fold(-1, |prev_value, new_value| {
                let new_value = i16::from(new_value.temp);
                // If the previous value is smaller than the next, return `None` -> not sorted
                if prev_value < new_value {
                    Some(new_value)
                } else {
                    None
                }
            })
            .is_some();

        if !is_sorted {
            tracing::warn!("Temperature in temperature profile isn't increasing: `{file_name:?}`");
            inner.sort_by(|first, second| first.temp.cmp(&second.temp));
        }

        // Make sure that the fan speed is increasing along with the temperature.
        let prev_speed = 100;
        for value in inner.iter_mut().rev() {
            // Cap value to 100
            if value.fan > 100 {
                tracing::warn!("Fan speed can't be larger than 100%: `{file_name:?}`");
                value.fan = 100;
            }

            if value.fan > prev_speed {
                value.fan = prev_speed;
                tracing::warn!(
                    "Fan speed isn't increasing along with the temperature: `{file_name:?}`"
                );
            }
        }

        // Make sure some minimum fan speed is kept:
        // From 50°C the fan speed should ramp up to
        // 100% at 100°C.
        for value in &mut inner {
            let min_speed = match value.temp {
                0..=50 => 0,
                51..=255 => (value.temp - 50).saturating_mul(2).min(100),
            };
            if min_speed > value.fan {
                let invalid_fan_value = value.fan;
                value.fan = min_speed;
                tracing::warn!(
                    "Fan speed {}% at {}°C is too low. Falling back to {min_speed}%: `{file_name:?}`",
                    invalid_fan_value,
                    value.temp
                );
            }
        }

        // Make sure that 100% fan speed will be reached
        if inner.last().unwrap().fan < 100 {
            tracing::warn!(
                "Fan speed 100% is never reached. Set speed to 100% at 100°C: `{file_name:?}`"
            );
            inner.push(FanProfilePoint {
                temp: 100,
                fan: 100,
            })
        }

        Ok(Self { inner })
    }

    // Use the temp profile in the configuration to calculate the
    // corresponding fan speed.
    pub fn calc_target_fan_speed(&self, current_temp: u8) -> u8 {
        // Find the first item that has a greater or equal temperature.
        let position = self.inner.iter().position(|p| p.temp >= current_temp);

        if let Some(position) = position {
            let profile_point = &self.inner[position];

            // If the profile point fits exact or it's the first element,
            // directly use its temperature.
            if profile_point.temp == current_temp || position == 0 {
                profile_point.fan
            } else {
                let prev_point = &self.inner[position - 1];

                // Interpolate with a linear slope between those two points.
                // Use u16 to make sure the multiplication doesn't overflow.
                let temp_diff = (profile_point.temp - prev_point.temp) as u16;
                let curr_temp_diff = (current_temp - prev_point.temp) as u16;
                let fan_diff = (profile_point.fan - prev_point.fan) as u16;

                prev_point.fan + (fan_diff * curr_temp_diff / temp_diff) as u8
            }
        } else {
            // The temperature is higher than anything in the list.
            100
        }
    }
}

impl Default for FanProfile {
    fn default() -> Self {
        Self {
            inner: vec![
                FanProfilePoint { temp: 25, fan: 0 },
                FanProfilePoint { temp: 30, fan: 10 },
                FanProfilePoint { temp: 40, fan: 22 },
                FanProfilePoint { temp: 50, fan: 35 },
                FanProfilePoint { temp: 60, fan: 45 },
                FanProfilePoint { temp: 70, fan: 62 },
                FanProfilePoint { temp: 80, fan: 75 },
                FanProfilePoint { temp: 90, fan: 100 },
            ],
        }
    }
}
