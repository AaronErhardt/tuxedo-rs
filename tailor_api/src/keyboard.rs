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
    use crate::keyboard::Color;
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
