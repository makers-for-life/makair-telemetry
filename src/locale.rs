// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use std::convert::TryFrom;
use std::ops::RangeInclusive;

/// An ISO 639-1 language code to be used to choose language for the ControlUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLocale(u16);

impl UiLocale {
    /// Language code as a u16
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    /// Language code as a usize
    pub fn as_usize(&self) -> usize {
        self.0.into()
    }

    /// Allowed value bounds
    pub fn bounds() -> RangeInclusive<usize> {
        RangeInclusive::new(
            Self::try_from("aa").unwrap().as_usize(),
            Self::try_from("zz").unwrap().as_usize(),
        )
    }
}

impl TryFrom<&str> for UiLocale {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 2 {
            let bytes = value.as_bytes();
            let w = ((bytes[0] as u16) << 8) | bytes[1] as u16;
            Ok(UiLocale(w))
        } else {
            Err("language code must be exactly 2 characters, according to ISO 639-1")
        }
    }
}

impl Default for UiLocale {
    fn default() -> Self {
        UiLocale::try_from("en").unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::UiLocale;

    use std::convert::TryFrom;

    #[test]
    fn fr() {
        let expected: u16 = 0x6672;
        assert_eq!(
            UiLocale::try_from("fr").map(|code| code.as_u16()),
            Ok(expected)
        );
    }

    #[test]
    fn empty() {
        assert!(UiLocale::try_from("").is_err())
    }

    #[test]
    fn too_long() {
        assert!(UiLocale::try_from("fra").is_err())
    }
}
