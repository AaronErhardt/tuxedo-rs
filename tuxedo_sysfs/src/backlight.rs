use glob::glob;
use std::{
    io,
    path::{Path, PathBuf},
};
use tokio_uring::fs;

use crate::sysfs_util::{read_to_int, rw_file, write_buffer};

const SYSFS_BACKLIGHT_CLASS: &str = "/sys/class/backlight";

type BacklightResult<T> = Result<T, io::Error>;

#[non_exhaustive]
pub struct GpuDriver {
    gpus: Vec<PathBuf>,
}

impl GpuDriver {
    fn new() -> Self {
        let amd_pattern = format!("{SYSFS_BACKLIGHT_CLASS}/amdgpu_bl*");
        let mut amd_gpus = vec![];
        for entry in glob(&amd_pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                amd_gpus.push(path.to_path_buf());
            }
        }
        GpuDriver { gpus: amd_gpus }
    }

    fn get_number_of_backlights(&self) -> u8 {
        self.gpus.len() as u8
    }
}

pub struct BacklightDriver {
    paths: Vec<String>,
}

impl BacklightDriver {
    pub fn new() -> Self {
        let amd_pattern = format!("{SYSFS_BACKLIGHT_CLASS}/amdgpu_bl*");
        let mut amd_gpus = vec![];
        for entry in glob(&amd_pattern).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                amd_gpus.push(path.display().to_string());
            }
        }
        Self { paths: amd_gpus }
    }

    async fn file(&self, number: usize, subsystem: &str) -> BacklightResult<fs::File> {
        match self.paths.get(number) {
            Some(path) => rw_file(format!("{path}/{subsystem}")).await,
            None => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unable to set backlight {number}. Available number of backlights {}",
                    self.paths.len()
                ),
            )),
        }
    }

    fn get_number_of_backlights(&self) -> usize {
        self.paths.len()
    }

    async fn get_current_backlight(&self, number: usize) -> BacklightResult<u8> {
        let value = read_to_int(&mut self.file(number, "actual_brightness").await?).await?;
        Ok(value as u8)
    }

    async fn get_maximum_backlight(&self, number: usize) -> BacklightResult<u8> {
        let value = read_to_int(&mut self.file(number, "max_brightness").await?).await?;
        Ok(value as u8)
    }

    async fn set_backlight(&self, number: usize, value: u8) -> BacklightResult<()> {
        let current_brightness = self.get_current_backlight(number).await?;
        let max_brightness = self.get_maximum_backlight(number).await?;
        let brightness_raw = value * max_brightness / 100;
        write_buffer(
            &mut self.file(number, "brightness").await?,
            brightness_raw.to_string().into_bytes(),
        )
        .await?;
        Ok(())
    }
}
