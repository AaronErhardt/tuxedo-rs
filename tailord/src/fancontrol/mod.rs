use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tuxedo_ioctl::high_level::{Fan, IoInterface};

use self::{buffer::TemperatureBuffer, profile::FanProfile};

mod buffer;
pub mod profile;
mod runtime;

#[derive(Debug)]
pub struct FanRuntime {
    /// Stores the temperature history.
    temp_history: TemperatureBuffer,
    /// Percentage of the current fan speed.
    /// This is used to avoid unnecessary updates.
    fan_speed: u8,
    /// Device i/o interface.
    io: IoInterface,
    /// The configuration.
    profile: FanProfile,
    suspend_receiver: broadcast::Receiver<bool>,
}

impl FanRuntime {
    // initialize global instance at startup
    pub fn new(profile: FanProfile, suspend_receiver: broadcast::Receiver<bool>) -> FanRuntime {
        let io = IoInterface::new().unwrap();
        let fan_speed = io.get_fan_speed_percent(Fan::Fan1).unwrap();
        let temp = io.get_fan_temperature(Fan::Fan1).unwrap();
        let temp_history = TemperatureBuffer::new(temp);

        io.set_fans_manual().unwrap();

        FanRuntime {
            temp_history,
            fan_speed,
            io,
            profile,
            suspend_receiver,
        }
    }

    pub async fn run(
        mut self,
        mut fan_receiver: mpsc::Receiver<FanProfile>,
        mut fan_speed_receiver: mpsc::Receiver<u8>,
    ) {
        loop {
            tokio::select! {
                new_config = fan_receiver.recv() => {
                    if let Some(config) = new_config {
                        self.profile = config;
                    }
                },
                // Override the fan speed for 1s
                override_speed = fan_speed_receiver.recv() => {
                    if let Some(mut speed) = override_speed {
                        loop {
                            if let Err(err) = self.io.set_fan_speed_percent(Fan::Fan1, speed) {
                                tracing::error!("Failed to update fan speed: `{}`", err.to_string());
                                break;
                            }
                            tokio::select! {
                                override_speed = fan_speed_receiver.recv() => {
                                    if let Some(new_speed) = override_speed {
                                        speed = new_speed
                                    }
                                }
                                _ = tokio::time::sleep(Duration::from_millis(1000)) => break,
                            }
                        }
                    }
                }
                _ = self.fan_control_loop() => {},
            }
        }
    }

    /// Adds entries to history ring buffer.
    fn update_temp(&mut self) -> u8 {
        match self.io.get_fan_temperature(Fan::Fan1) {
            Ok(temp) => {
                self.temp_history.update(temp);
                temp
            }
            Err(err) => {
                tracing::error!("Failed reading the current temperature: `{err}`");
                self.temp_history.get_latest()
            }
        }
    }

    fn set_speed(&mut self, new_speed: u8) {
        if self.fan_speed != new_speed {
            self.fan_speed = new_speed;
            if let Err(err) = self.io.set_fan_speed_percent(Fan::Fan1, new_speed) {
                tracing::error!("Failed setting new fan speed: `{err}`");
            }
        }
    }
}
