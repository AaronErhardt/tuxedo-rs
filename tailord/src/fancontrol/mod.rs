use std::{sync::Arc, time::Duration};

use tokio::sync::{broadcast, mpsc};
use tuxedo_ioctl::hal::traits::HardwareDevice;

use crate::suspend::get_suspend_receiver;

use self::{buffer::TemperatureBuffer, profile::FanProfile};

mod buffer;
pub mod profile;
mod runtime;

#[derive(Clone)]
pub struct FanRuntimeHandle {
    pub fan_speed_sender: mpsc::Sender<u8>,
    pub profile_sender: mpsc::Sender<FanProfile>,
}

#[derive(Debug)]
pub struct FanRuntimeData {
    fan_idx: u8,
    /// Stores the temperature history.
    temp_history: TemperatureBuffer,
    /// Percentage of the current fan speed.
    /// This is used to avoid unnecessary updates.
    fan_speed: u8,
    /// Device i/o interface.
    io: Arc<dyn HardwareDevice>,
    /// The configuration.
    profile: FanProfile,
    suspend_receiver: broadcast::Receiver<bool>,
}

pub struct FanRuntime {
    fan_receiver: mpsc::Receiver<FanProfile>,
    fan_speed_receiver: mpsc::Receiver<u8>,
    data: FanRuntimeData,
}

impl FanRuntime {
    // initialize global instance at startup
    pub fn new(
        fan_idx: u8,
        io: Arc<dyn HardwareDevice>,
        profile: FanProfile,
    ) -> (FanRuntimeHandle, FanRuntime) {
        let fan_speed = io.get_fan_speed_percent(fan_idx).unwrap();
        let temp = io.get_fan_temperature(fan_idx).unwrap();
        let temp_history = TemperatureBuffer::new(temp);

        let (fan_sender, fan_receiver) = mpsc::channel(1);
        let (fan_speed_sender, fan_speed_receiver) = mpsc::channel(1);
        let suspend_receiver = get_suspend_receiver();

        (
            FanRuntimeHandle {
                fan_speed_sender,
                profile_sender: fan_sender,
            },
            FanRuntime {
                data: FanRuntimeData {
                    temp_history,
                    fan_speed,
                    io,
                    profile,
                    fan_idx,
                    suspend_receiver,
                },
                fan_receiver,
                fan_speed_receiver,
            },
        )
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                new_config = self.fan_receiver.recv() => {
                    if let Some(config) = new_config {
                        self.data.profile = config;
                    }
                },
                // Override the fan speed for 1s
                override_speed = self.fan_speed_receiver.recv() => {
                    if let Some(mut speed) = override_speed {
                        loop {
                            if let Err(err) = self.data.io.set_fan_speed_percent(self.data.fan_idx, speed) {
                                tracing::error!("Failed to update fan speed: `{}`", err.to_string());
                                break;
                            }
                            tokio::select! {
                                override_speed = self.fan_speed_receiver.recv() => {
                                    if let Some(new_speed) = override_speed {
                                        speed = new_speed
                                    }
                                }
                                _ = tokio::time::sleep(Duration::from_millis(1000)) => break,
                            }
                        }
                    }
                }
                _ = self.data.fan_control_loop() => {},
            }
        }
    }
}

impl FanRuntimeData {
    #[tracing::instrument(level = "trace", skip(self))]
    /// Adds entries to history ring buffer.
    fn update_temp(&mut self) -> u8 {
        match self.io.get_fan_temperature(self.fan_idx) {
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

    #[tracing::instrument(level = "trace", skip(self))]
    fn set_speed(&mut self, new_speed: u8) {
        if self.fan_speed != new_speed {
            self.fan_speed = new_speed;
            if let Err(err) = self.io.set_fan_speed_percent(self.fan_idx, new_speed) {
                tracing::error!("Failed setting new fan speed: `{err}`");
            }
        }
    }
}
