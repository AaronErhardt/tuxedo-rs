use crate::suspend::process_suspend;

use super::{buffer::TemperatureBuffer, FanRuntimeData};

use std::time::Duration;

impl FanRuntimeData {
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn fan_control_loop(&mut self) {
        loop {
            // Add the current temperature to history
            let current_temp = self.update_temp();

            let target_fan_speed = self.profile.calc_target_fan_speed(current_temp);
            let fan_diff = self.fan_speed.abs_diff(target_fan_speed);

            // Make small steps to decrease or increase fan speed.
            // If the target fan speed is below 50%, don't increase the speed at all
            // unless the difference is higher than 3% to avoid frequent speed changes
            // at low temperatures.
            let fan_increment = fan_diff / 4 + (target_fan_speed / 50);

            // Update fan speed
            self.set_speed(if target_fan_speed > self.fan_speed {
                self.fan_speed.saturating_add(fan_increment).min(100)
            } else {
                self.fan_speed.saturating_sub(fan_increment)
            });

            let delay = suitable_delay(&self.temp_history, fan_diff);

            tracing::debug!(
                "Fan {}: Current temperature is {current_temp}°C, fan speed: {}%, target fan speed: {target_fan_speed} \
                fan diff: {fan_diff}, fan increment {fan_increment}, delay: {delay:?}", self.fan_idx, self.fan_speed
            );

            tokio::select! {
                _ = tokio::time::sleep(delay) => {},
                _ = process_suspend(&mut self.suspend_receiver) => {
                    self.fan_speed = self.io.get_fan_speed_percent(0).unwrap();
                }
            }
        }
    }
}

/// Calculate a suitable delay to reduce CPU usage.
fn suitable_delay(temp_buffer: &TemperatureBuffer, fan_diff: u8) -> Duration {
    // How much is the temperature changing?
    let temperature_pressure = temp_buffer.diff_to_min_in_history();

    // How much is the fan speed off from the ideal value?
    let fan_diff_pressure = fan_diff / 2;

    // Calculate an overall pressure value from 0 to 15.
    let pressure = temperature_pressure
        .saturating_add(fan_diff_pressure)
        .min(15);

    // Define a falling exponential function with time constant -1/7.
    // This should yield decent results but the formula might be tuned
    // to perform better.
    // 0  -> 2000ms
    // 15 -> ~230ms
    const TAU: f64 = -1.0 / 7.0;
    let delay = 2000.0 * (pressure as f64 * TAU).exp();
    Duration::from_millis(delay as u64)
}

#[cfg(test)]
mod test {
    use crate::fancontrol::buffer::TemperatureBuffer;

    use super::suitable_delay;

    #[test]
    fn test_suitable_delay() {
        let mut temp_buffer = TemperatureBuffer::new(20);

        // Test with no pressure.
        assert_eq!(suitable_delay(&temp_buffer, 0).as_millis(), 2000);

        // Test with max pressure.
        assert_eq!(suitable_delay(&temp_buffer, 255).as_millis(), 234);

        // Test with pressure 1.
        assert_eq!(suitable_delay(&temp_buffer, 2).as_millis(), 1733);

        // Test with pressure 1 but this time through temperature diff.
        temp_buffer.update(21);
        assert_eq!(suitable_delay(&temp_buffer, 0).as_millis(), 1733);
    }
}
