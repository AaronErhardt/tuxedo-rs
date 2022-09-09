use std::{fmt::Display, io, str::FromStr};

use tailor_api::keyboard::Color;

use crate::{sys_fs_read, sys_fs_write};

use super::sys_fs_type;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum KeyboardState {
    Custom,
    Breathe,
    Cycle,
    Dance,
    Flash,
    RandomColor,
    Tempo,
    Wave,
}

impl Display for KeyboardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Custom => 0,
                Self::Breathe => 1,
                Self::Cycle => 2,
                Self::Dance => 3,
                Self::Flash => 4,
                Self::RandomColor => 5,
                Self::Tempo => 6,
                Self::Wave => 7,
            }
        )
    }
}

impl FromStr for KeyboardState {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Custom),
            "1" => Ok(Self::Breathe),
            "2" => Ok(Self::Cycle),
            "3" => Ok(Self::Dance),
            "4" => Ok(Self::Flash),
            "5" => Ok(Self::RandomColor),
            "6" => Ok(Self::Tempo),
            "7" => Ok(Self::Wave),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected value in the range 0-7, found {s}"),
            )),
        }
    }
}

struct NumBool(bool);

impl Display for NumBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.0 { 1 } else { 0 })
    }
}

impl FromStr for NumBool {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self(false)),
            "1" => Ok(Self(true)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected value in the range 0-1, found {s}"),
            )),
        }
    }
}

sys_fs_type!(KB, RW, Color, ColorLeft, "color_left");
sys_fs_type!(KB, RW, Color, ColorCenter, "color_center");
sys_fs_type!(KB, RW, Color, ColorRight, "color_right");
sys_fs_type!(KB, RW, Color, ColorExtra, "color_extra");
sys_fs_type!(KB, RW, u8, Brightness, "brightness");
sys_fs_type!(KB, RW, NumBool, Mode, "mode");
sys_fs_type!(KB, RW, KeyboardState, State, "state");

/// A type that manages all sysfs files related to
/// keyboard color management.
pub struct KeyboardController {
    color_left: ColorLeft,
    color_right: ColorRight,
    color_center: ColorCenter,
    color_extra: ColorExtra,
    brightness: Brightness,
    mode: Mode,
    state: State,
}

impl KeyboardController {
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

    pub async fn set_color_left(&self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&self.color_left, color).await
    }

    pub async fn get_color_left(&self) -> Result<Color, io::Error> {
        sys_fs_read(&self.color_left).await
    }

    pub async fn set_color_right(&self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&self.color_right, color).await
    }

    pub async fn get_color_right(&self) -> Result<Color, io::Error> {
        sys_fs_read(&self.color_left).await
    }

    pub async fn set_color_center(&self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&self.color_center, color).await
    }

    pub async fn get_color_center(&self) -> Result<Color, io::Error> {
        sys_fs_read(&self.color_left).await
    }

    pub async fn set_color_extra(&self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&self.color_extra, color).await
    }

    pub async fn get_color_extra(&self) -> Result<Color, io::Error> {
        sys_fs_read(&self.color_left).await
    }

    pub async fn set_color_all(&self, color: &Color) -> Result<(), io::Error> {
        let (r0, r1, r2, r3) = futures::join!(
            self.set_color_left(color),
            self.set_color_right(color),
            self.set_color_center(color),
            self.set_color_extra(color)
        );
        r0?;
        r1?;
        r2?;
        r3?;
        Ok(())
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

    use tailor_api::keyboard::Color;

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
