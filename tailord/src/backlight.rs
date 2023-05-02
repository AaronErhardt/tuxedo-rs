use std::ops::Deref;

use tokio::sync::mpsc;
use tuxedo_sysfs::backlight::BacklightDriver;

#[derive(Debug)]
pub struct BacklightProfile(u8);

impl Deref for BacklightProfile {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BacklightProfile {
    pub fn new(value: u8) -> Self {
        Self(value)
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct BacklightRuntimeHandle {
    pub backlight_percentage_sender: mpsc::Sender<u8>,
    pub actual_backlight_percentage: u8,
    pub max_backlight_raw: u8,
}

#[allow(unused)]
pub struct BacklightRuntime {
    backlight_percentage_receiver: mpsc::Receiver<u8>,
    /// Device i/o interface.
    controller: BacklightDriver,
}

impl BacklightRuntime {
    #[tracing::instrument(skip(controller))]
    pub async fn new(
        mut controller: BacklightDriver,
        actual_backlight_percentage: u8,
    ) -> (BacklightRuntimeHandle, BacklightRuntime) {
        let (backlight_percentage_sender, backlight_percentage_receiver) = mpsc::channel(1);
        let max_backlight_raw = controller.get_maximum_backlight_raw().await.unwrap();
        controller
            .set_backlight_percentage(actual_backlight_percentage)
            .await
            .unwrap();
        (
            BacklightRuntimeHandle {
                backlight_percentage_sender,
                actual_backlight_percentage,
                max_backlight_raw,
            },
            BacklightRuntime {
                backlight_percentage_receiver,
                controller,
            },
        )
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(mut self) {
        loop {
            if let Some(value) = self.backlight_percentage_receiver.recv().await {
                tracing::info!("Setting backlight to {value}%");
                self.controller
                    .set_backlight_percentage(value)
                    .await
                    .unwrap();
            } else {
                tracing::warn!(
                    "Stopping runtime, the performance profile channel sender has probably dropped"
                );
                break;
            }
        }
    }
}
