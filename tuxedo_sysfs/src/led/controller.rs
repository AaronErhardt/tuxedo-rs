use std::io;

use tailor_api::Color;

use crate::sysfs_util::{read_int_list, write_string};

use super::{Controller, ControllerMode};

impl Controller {
    pub async fn new_rgb(
        max_brightness: u32,
        device_name: String,
        function: String,
        mut brightness_file: tokio_uring::fs::File,
        intensities_file: tokio_uring::fs::File,
    ) -> Result<Self, io::Error> {
        // Set brightness to 100% so the individual intensities represent their colors without additional scaling.
        write_string(&mut brightness_file, max_brightness.to_string()).await?;

        Ok(Self {
            max_brightness,
            device_name,
            function,
            brightness_file,
            intensities_file: Some(intensities_file),
        })
    }

    pub async fn new_monochrome(
        max_brightness: u32,
        device_name: String,
        function: String,
        brightness_file: tokio_uring::fs::File,
    ) -> Result<Self, io::Error> {
        Ok(Self {
            max_brightness,
            device_name,
            function,
            brightness_file,
            intensities_file: None,
        })
    }

    pub async fn set_color(&mut self, color: &Color) -> Result<(), io::Error> {
        let Self {
            max_brightness,
            brightness_file,
            intensities_file,
            ..
        } = self;

        if let Some(intensities) = intensities_file {
             write_string(intensities, color.sysfs_rgb_string(*max_brightness)).await
        } else {
             write_string(
                 brightness_file,
                 color.sysfs_monochrome_string(*max_brightness),
             )
             .await
        }
    }

    pub async fn get_color(&mut self) -> Result<Color, io::Error> {
        let Self {
            max_brightness,
            brightness_file,
            intensities_file,
            ..
        } = self;

        if let Some(intensities) = intensities_file {
            let values = read_int_list(intensities).await?;
            let values: [u32; 3] = values.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid number of values")
            })?;
            Ok(Color::from_sysfs_rgb_value(values, *max_brightness))
        } else {
            let value = read_int_list(brightness_file).await?[0];
            Ok(Color::from_sysfs_rgb_value(
                [value, value, value],
                *max_brightness,
            ))
        }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    pub fn function(&self) -> &str {
        &self.function
    }

    pub fn mode(&self) -> ControllerMode {
        if self.intensities_file.is_some() {
            ControllerMode::Rgb
        } else {
            ControllerMode::Monochrome
        }
    }
}
