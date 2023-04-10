use std::{
    io,
    ops::{Index, IndexMut},
};

use tailor_api::Color;

use crate::sysfs_util::{r_file, read_int_list, read_path_to_string, rw_file};

use super::{Collection, Controller};

const SYSFS_LED_PATH: &str = "/sys/class/leds";
const BRIGHTNESS: &str = "brightness";
const MAX_BRIGHTNESS: &str = "max_brightness";
const MULTI_INDEX: &str = "multi_index";
const MULTI_INTENSITIES: &str = "multi_intensity";
const DEVICE_NAME: &str = "device/name";
const DEVICE_MODALIAS: &str = "device/modalias";

impl Collection {
    pub async fn new() -> Result<Self, io::Error> {
        let mut controllers = Vec::new();

        let mut dirs = tokio::fs::read_dir(SYSFS_LED_PATH).await?;
        while let Some(dir) = dirs.next_entry().await? {
            let path = dir.path();
            let file_name = path
                .file_name()
                .expect("The sysfs path must have a last segment");
            let file_name_str = file_name.to_str().unwrap_or_default();

            if file_name_str.contains("mmc") {
                // Not a useful device, skip it.
                continue;
            }

            let function = if let Some(function) = file_name_str.split(':').last() {
                function.trim().to_owned()
            } else {
                tracing::warn!("Badly formatted led device: {:?}", file_name);
                continue;
            };

            let device_name_path = path.join(DEVICE_NAME);
            let device_name = if let Ok(name) = read_path_to_string(device_name_path).await {
                name.trim().to_owned()
            } else {
                let device_modalias_path = path.join(DEVICE_MODALIAS);
                if let Ok(name) = read_path_to_string(device_modalias_path).await {
                    name.trim().to_owned()
                } else {
                    tracing::warn!("Could not find LED device name: {:?}", file_name);
                    continue;
                }
            };

            // Check for brightness file
            let brightness_path = path.join(BRIGHTNESS);
            let brightness_file = if let Ok(file) = rw_file(brightness_path).await {
                file
            } else {
                // Not even basic support available -> skip device.
                continue;
            };

            // Get maximum brightness
            let max_brightness_path = path.join(MAX_BRIGHTNESS);
            let max_brightness = if let Ok(mut file) = r_file(max_brightness_path).await {
                if let Ok(values) = read_int_list(&mut file).await {
                    values[0]
                } else {
                    tracing::warn!("Brightness file can't be read: {:?}", file_name);
                    continue;
                }
            } else {
                // Not even basic support available -> skip device.
                continue;
            };

            if max_brightness < 2 {
                // Not even basic support available -> skip device.
                continue;
            }

            let multi_index_path = path.join(MULTI_INDEX);
            let rgb_support = tokio::fs::read_to_string(multi_index_path)
                .await
                .map(|content| content.trim().to_lowercase() == "red green blue")
                .unwrap_or_default();

            if rgb_support {
                // Get intensities
                let intensities_path = path.join(MULTI_INTENSITIES);
                let (intensities_file, intensities) =
                    if let Ok(mut file) = rw_file(intensities_path).await {
                        if let Ok(values) = read_int_list(&mut file).await {
                            (file, values)
                        } else {
                            tracing::warn!("Intensities file can't be read: {:?}", file_name);
                            continue;
                        }
                    } else {
                        // Should be there for an RGB device
                        tracing::warn!(
                            "RGB device should have multiple intensities: {:?}",
                            file_name
                        );
                        continue;
                    };

                if intensities.len() == 3 {
                    // Push controller with RGB capabilities
                    controllers.push(
                        Controller::new_rgb(
                            max_brightness,
                            device_name,
                            function,
                            brightness_file,
                            intensities_file,
                        )
                        .await?,
                    );
                } else {
                    // Should be 3 for an RGB device
                    tracing::warn!("RGB device should have 3 intensities: {:?}", file_name);
                    continue;
                }
            } else {
                // Push controller with monochrome capabilities
                controllers.push(
                    Controller::new_monochrome(
                        max_brightness,
                        device_name,
                        function,
                        brightness_file,
                    )
                    .await?,
                );
            }
        }

        Ok(Self { controllers })
    }

    pub async fn set_color_all(&mut self, color: &Color) -> Result<(), io::Error> {
        for controller in &mut self.controllers {
            controller.set_color(color).await?;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.controllers.is_empty()
    }

    pub fn len(&self) -> usize {
        self.controllers.len()
    }

    pub fn get(&self, index: usize) -> Option<&Controller> {
        self.controllers.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Controller> {
        self.controllers.get_mut(index)
    }

    pub fn into_inner(self) -> Vec<Controller> {
        self.controllers
    }
}

impl Index<usize> for Collection {
    type Output = Controller;

    fn index(&self, index: usize) -> &Self::Output {
        &self.controllers[index]
    }
}

impl IndexMut<usize> for Collection {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.controllers[index]
    }
}

#[cfg(test)]
mod test {
    use tailor_api::Color;

    use super::Collection;

    #[test]
    fn test_colors() {
        sudo::escalate_if_needed().unwrap();

        tracing_subscriber::fmt::init();

        tokio_uring::start(async {
            let collection = Collection::new().await.unwrap();
            let mut led_controller = collection.into_inner().pop().unwrap();

            let test_color = Color {
                r: 255,
                g: 100,
                b: 0,
            };

            led_controller.set_color(&test_color).await.unwrap();
            assert_eq!(led_controller.get_color().await.unwrap(), test_color);
        });
    }
}
