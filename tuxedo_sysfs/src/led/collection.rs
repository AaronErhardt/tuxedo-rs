use std::{
    io,
    ops::{Index, IndexMut},
};

use tailor_api::Color;

use crate::sysfs_util::{file_exists, read_int_list, read_path_to_string, rw_file};

use super::{Collection, Controller};

const SYSFS_LED_PATH: &str = "/sys/class/leds";
const BRIGHTNESS: &str = "max_brightness";
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

            let function =
                if let Some(function) = file_name.to_str().unwrap_or_default().split(':').last() {
                    function.trim().to_owned()
                } else {
                    tracing::warn!("Badly formatted led device: {:?}", file_name);
                    break;
                };

            let device_name_path = path.join(DEVICE_NAME);
            let device_name = if let Ok(name) = read_path_to_string(device_name_path).await {
                name
            } else {
                let device_modalias_path = path.join(DEVICE_MODALIAS);
                if let Ok(name) = read_path_to_string(device_modalias_path).await {
                    name
                } else {
                    tracing::warn!("Could not find LED device name: {:?}", file_name);
                    break;
                }
            };

            // Check for brightness file
            let brightness_path = path.join(BRIGHTNESS);
            if !file_exists(&brightness_path).await {
                // Not even basic support available -> skip device.
                break;
            }

            // Get maximum brightness
            let max_brightness_path = path.join(MAX_BRIGHTNESS);
            let (brightness_file, max_brightness) =
                if let Ok(mut file) = rw_file(max_brightness_path).await {
                    if let Ok(values) = read_int_list(&mut file).await {
                        (file, values[0])
                    } else {
                        tracing::warn!("Brightness file can't be read: {:?}", file_name);
                        break;
                    }
                } else {
                    // Not even basic support available -> skip device.
                    break;
                };

            if max_brightness < 2 {
                // Not even basic support available -> skip device.
                break;
            }

            let multi_index_path = path.join(MULTI_INDEX);
            let rgb_support = tokio::fs::read_to_string(multi_index_path)
                .await
                .map(|content| content.to_lowercase() == "reg green blue")
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
                            break;
                        }
                    } else {
                        // Should be there for an RGB device
                        tracing::warn!(
                            "RGB device should have multiple intensities: {:?}",
                            file_name
                        );
                        break;
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
                    break;
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

/*
    pub async fn new() -> Result<Self, io::Error> {
        Ok(Self {
            color_left: ColorLeft::new().await?,
            color_right: ColorRight::new().await?,
            color_center: ColorCenter::new().await?,
            color_extra: ColorExtra::new().await?,
            brightness: Brightness::new().await?,
            mode: Mode::new().await?,
            state: State::new().await?,
        })
    }


    pub async fn set_brightness(&self, brightness: u8) -> Result<(), io::Error> {
        sys_fs_write(&self.brightness, &brightness).await
    }

    pub async fn get_brightness(&self) -> Result<u8, io::Error> {
        sys_fs_read(&self.brightness).await
    }

    pub async fn set_mode(&self, mode: bool) -> Result<(), io::Error> {
        sys_fs_write(&self.mode, &NumBool(mode)).await
    }

    pub async fn get_mode(&self) -> Result<bool, io::Error> {
        sys_fs_read(&self.mode).await.map(|res| res.0)
    }

    pub async fn set_state(&self, state: KeyboardState) -> Result<(), io::Error> {
        sys_fs_write(&self.state, &state).await
    }

    pub async fn get_state(&self) -> Result<KeyboardState, io::Error> {
        sys_fs_read(&self.state).await
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use tailor_api::Color;

    use super::KeyboardController;

    #[test]
    fn test_colors() {
        if sudo::check() == sudo::RunningAs::User {
            return;
        }

        tokio_uring::start(async {
            let c = KeyboardController::new().await.unwrap();

            let test_color = Color { r: 255, g: 0, b: 0 };

            c.set_color_left(&test_color).await.unwrap();
            assert_eq!(c.get_color_left().await.unwrap(), test_color);

            let test_color = Color {
                r: 0,
                g: 150,
                b: 180,
            };

            c.set_color_all(&test_color).await.unwrap();
            assert_eq!(c.get_color_left().await.unwrap(), test_color);
            assert_eq!(c.get_color_right().await.unwrap(), test_color);
            assert_eq!(c.get_color_center().await.unwrap(), test_color);
            assert_eq!(c.get_color_extra().await.unwrap(), test_color);

            c.set_brightness(0).await.unwrap();
            assert_eq!(c.get_brightness().await.unwrap(), 0);
        });
    }

    #[test]
    fn test_color_rotation() {
        if sudo::check() == sudo::RunningAs::User {
            return;
        }

        tokio_uring::start(async {
            let c = KeyboardController::new().await.unwrap();

            // Make sure that other tests running in parallel don't interfere
            std::thread::sleep(Duration::from_millis(100));

            c.set_brightness(255).await.unwrap();

            for i in (0..255 * 3).step_by(4) {
                let mut color = Color { r: 0, g: 0, b: 0 };

                let value = i as u8 % 255;
                match i / 255 {
                    0 => color.r = value,
                    1 => color.g = value,
                    2 => color.b = value,
                    _ => unreachable!(),
                }

                let value = 255 - value;
                match i / 255 {
                    0 => color.b = value,
                    1 => color.r = value,
                    2 => color.g = value,
                    _ => unreachable!(),
                }

                c.set_color_left(&color).await.unwrap();
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });
    }
}
 */
