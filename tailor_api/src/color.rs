use atoi::FromRadix16;
use std::{fmt::Display, io, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ColorPoint {
    pub color: Color,
    pub transition: ColorTransition,
    /// Transition time in ms.
    pub transition_time: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ColorTransition {
    None,
    Linear,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ColorProfile {
    None,
    Single(Color),
    Multiple(Vec<ColorPoint>),
}

impl Default for ColorProfile {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn sysfs_rgb_string(&self, max_brightness: u32) -> String {
        let Color { r, g, b } = *self;
        if max_brightness == 255 {
            format!("{r} {g} {b}")
        } else {
            let max = max_brightness as f32;
            let scale = |value: u8| -> u32 { (value as f32 / 255.0 * max).clamp(0.0, max) as u32 };
            format!("{} {} {}", scale(r), scale(g), scale(b))
        }
    }

    pub fn sysfs_monochrome_string(&self, max_brightness: u32) -> String {
        let Color { r, g, b } = *self;
        let average = [r, g, b].into_iter().map(u16::from).sum::<u16>() / 3;
        if max_brightness == 255 {
            average.to_string()
        } else {
            let max = max_brightness as f32;
            let value = (average as f32 / 255.0 * max).clamp(0.0, max) as u32;
            value.to_string()
        }
    }

    pub fn from_sysfs_rgb_value(values: [u32; 3], max_brightness: u32) -> Self {
        if max_brightness == 255 {
            let values: Vec<u8> = values
                .into_iter()
                .map(u8::try_from)
                .map(Result::unwrap_or_default)
                .collect();
            Self {
                r: values[0],
                g: values[1],
                b: values[2],
            }
        } else {
            let max = max_brightness as f32;
            let scale = |value: u32| -> u8 { (value as f32 / max * 255.0).clamp(0.0, 255.0) as u8 };
            Self {
                r: scale(values[0]),
                g: scale(values[1]),
                b: scale(values[2]),
            }
        }
    }
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

#[cfg(test)]
mod test {
    use crate::color::Color;
    use std::str::FromStr;

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
}
