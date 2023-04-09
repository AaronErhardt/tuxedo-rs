use tailor_api::{Color, ColorProfile};
use tokio::sync::mpsc;
use tuxedo_sysfs::led::Controller;

use crate::profiles::LedDeviceInfo;

pub mod runtime;

pub struct LedRuntime {
    data: LedRuntimeData,
    profile_receiver: mpsc::Receiver<ColorProfile>,
    color_receiver: mpsc::Receiver<Color>,
}

pub struct LedRuntimeData {
    pub controller: Controller,
    pub profile: ColorProfile,
}

#[derive(Clone)]
pub struct LedRuntimeHandle {
    pub info: LedDeviceInfo,
    pub profile_sender: mpsc::Sender<ColorProfile>,
    pub color_sender: mpsc::Sender<Color>,
}

impl LedRuntime {
    pub fn new(data: LedRuntimeData) -> (LedRuntimeHandle, Self) {
        let (profile_sender, profile_receiver) = mpsc::channel(1);
        let (color_sender, color_receiver) = mpsc::channel(1);

        (
            LedRuntimeHandle {
                info: LedDeviceInfo {
                    device_name: data.controller.device_name.clone(),
                    function: data.controller.function.clone(),
                },
                profile_sender,
                color_sender,
            },
            Self {
                data,
                profile_receiver,
                color_receiver,
            },
        )
    }
}
