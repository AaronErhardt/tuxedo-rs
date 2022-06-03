use std::{fmt::Display, io, str::FromStr};

use crate::{sys_fs_read, sys_fs_write, SysFsType};

use super::sys_fs_type;

use atoi::FromRadix16;

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl FromStr for Color {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 6 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Incorrect length for 3x8-bit hexadecimal value",
            ))
        } else {
            let r = u8::from_radix_16(s[0..2].as_bytes());
            let g = u8::from_radix_16(s[2..4].as_bytes());
            let b = u8::from_radix_16(s[4..6].as_bytes());

            if r.1 == 2 && g.1 == 2 && b.1 == 2 {
                Ok(Self {
                    r: r.0,
                    g: g.0,
                    b: b.0,
                })
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Incorrect value (not [0-9] or [A-F]) in hexadecimal value",
                ))
            }
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

pub struct KeyboardController {
    color_left: ColorLeft,
    color_right: ColorRight,
    color_center: ColorCenter,
    color_extra: ColorLeft,
    brightness: Brightness,
    mode: Mode,
    state: State,
}

impl KeyboardController {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            color_left: ColorLeft::open_file()?,
            color_right: ColorRight::open_file()?,
            color_center: ColorCenter::open_file()?,
            color_extra: ColorLeft::open_file()?,
            brightness: Brightness::open_file()?,
            mode: Mode::open_file()?,
            state: State::open_file()?,
        })
    }

    pub fn set_color_left(&mut self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&mut self.color_left, color)
    }

    pub fn get_color_left(&mut self) -> Result<Color, io::Error> {
        sys_fs_read(&mut self.color_left)
    }

    pub fn set_color_right(&mut self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&mut self.color_right, color)
    }

    pub fn get_color_right(&mut self) -> Result<Color, io::Error> {
        sys_fs_read(&mut self.color_left)
    }

    pub fn set_color_center(&mut self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&mut self.color_center, color)
    }

    pub fn get_color_center(&mut self) -> Result<Color, io::Error> {
        sys_fs_read(&mut self.color_left)
    }

    pub fn set_color_extra(&mut self, color: &Color) -> Result<(), io::Error> {
        sys_fs_write(&mut self.color_extra, color)
    }

    pub fn get_color_extra(&mut self) -> Result<Color, io::Error> {
        sys_fs_read(&mut self.color_left)
    }

    pub fn set_color_all(&mut self, color: &Color) -> Result<(), io::Error> {
        self.set_color_left(color)?;
        self.set_color_right(color)?;
        self.set_color_center(color)?;
        self.set_color_extra(color)?;
        Ok(())
    }

    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), io::Error> {
        sys_fs_write(&mut self.brightness, &brightness)
    }

    pub fn get_brightness(&mut self) -> Result<u8, io::Error> {
        sys_fs_read(&mut self.brightness)
    }

    pub fn set_mode(&mut self, mode: bool) -> Result<(), io::Error> {
        sys_fs_write(&mut self.mode, &NumBool(mode))
    }

    pub fn get_mode(&mut self) -> Result<bool, io::Error> {
        sys_fs_read(&mut self.mode).map(|res| res.0)
    }

    pub fn set_state(&mut self, state: KeyboardState) -> Result<(), io::Error> {
        sys_fs_write(&mut self.state, &state)
    }

    pub fn get_state(&mut self) -> Result<KeyboardState, io::Error> {
        sys_fs_read(&mut self.state)
    }
}

#[cfg(test)]
mod test {
    use std::{str::FromStr, time::Duration};

    use super::{Color, KeyboardController};

    #[test]
    fn color_from_string() {
        let string = "000Fac";
        let color = Color::from_str(string).unwrap();
        assert_eq!(
            color,
            Color {
                r: 0x00,
                g: 0x0F,
                b: 0xAC,
            }
        );
        assert_eq!(color.to_string()[2..], string.to_ascii_uppercase());

        Color::from_str("F00FF").unwrap_err();
        Color::from_str("F").unwrap_err();
        Color::from_str("INVLD!").unwrap_err();
    }

    #[test]
    fn test_colors() {
        sudo::escalate_if_needed().unwrap();

        let mut c = KeyboardController::new().unwrap();

        let test_color = Color { r: 255, g: 0, b: 0 };

        c.set_color_left(&test_color).unwrap();
        assert_eq!(c.get_color_left().unwrap(), test_color);

        let test_color = Color {
            r: 0,
            g: 150,
            b: 180,
        };

        c.set_color_all(&test_color).unwrap();
        assert_eq!(c.get_color_left().unwrap(), test_color);
        assert_eq!(c.get_color_right().unwrap(), test_color);
        assert_eq!(c.get_color_center().unwrap(), test_color);
        assert_eq!(c.get_color_extra().unwrap(), test_color);

        c.set_brightness(0).unwrap();
        assert_eq!(c.get_brightness().unwrap(), 0);
    }

    #[test]
    fn test_color_rotation() {
        sudo::escalate_if_needed().unwrap();

        let mut c = KeyboardController::new().unwrap();

        // Make sure that other tests running in parallel don't interfere
        std::thread::sleep(Duration::from_millis(100));

        c.set_brightness(255).unwrap();

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

            c.set_color_left(&color).unwrap();
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
