use glob::glob;
use std::io;
use tokio_uring::fs;

use crate::sysfs_util::{r_file, read_to_int, rw_file, write_buffer};

const SYSFS_BACKLIGHT_CLASS: &str = "/sys/class/backlight";

type BacklightResult<T> = Result<T, io::Error>;

#[derive(Debug)]
pub struct BacklightDriver {
    set_brightness_file: fs::File,
    max_brightness_file: fs::File,
    actual_brightness_file: fs::File,
}

impl BacklightDriver {
    pub async fn new() -> BacklightResult<Self> {
        let amd_pattern = format!("{SYSFS_BACKLIGHT_CLASS}/amdgpu_bl*");
        for entry in glob(&amd_pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                let set_backlight_file = rw_file(format!("{}/brightness", path.display())).await?;
                let max_backlight_file =
                    r_file(format!("{}/max_brightness", path.display())).await?;
                let actual_backlight_file =
                    r_file(format!("{}/actual_brightness", path.display())).await?;
                return Ok(Self {
                    set_brightness_file: set_backlight_file,
                    max_brightness_file: max_backlight_file,
                    actual_brightness_file: actual_backlight_file,
                });
            }
        }
        todo!("Unsupported brightness device")
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_backlight_percentage(&mut self) -> BacklightResult<u8> {
        let value = read_to_int(&mut self.actual_brightness_file).await?;
        tracing::debug!("Current backlight brightness is {value}%");
        Ok(value as u8)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_maximum_backlight_raw(&mut self) -> BacklightResult<u8> {
        let value = read_to_int(&mut self.max_brightness_file).await?;
        tracing::debug!("Backlight brightness boundaries are between 0 and {value}");
        Ok(value as u8)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn set_backlight_percentage(
        &mut self,
        brightness_percentage: u8,
    ) -> BacklightResult<()> {
        let max_brightness = self.get_maximum_backlight_raw().await? as f32;
        let mut brightness_raw = (brightness_percentage as f32 * max_brightness / 100.0).round();
        if brightness_raw > max_brightness {
            tracing::warn!("Trying to set the backlight brighness to {brightness_raw}, setting it to the max of {max_brightness}");
            brightness_raw = max_brightness;
        }
        write_buffer(
            &mut self.set_brightness_file,
            brightness_raw.to_string().into_bytes(),
        )
        .await?;
        tracing::debug!("New backlight brightness percentage set to {brightness_percentage}%, raw brightness: {brightness_raw}");
        Ok(())
    }
}
