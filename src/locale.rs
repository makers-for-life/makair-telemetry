// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use std::convert::TryFrom;
use std::ops::RangeInclusive;

/// An ISO 639-1 language code to be used to choose language for the whole system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-messages",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct Locale(u16);

impl Locale {
    /// Create a locale from a u16
    pub fn try_from_u16(num: u16) -> Option<Self> {
        match Self::try_from(Self(num).to_string().as_str()) {
            Ok(locale) => Some(locale),
            Err(_) => None,
        }
    }

    /// Language code as a u16
    pub fn as_u16(&self) -> u16 {
        self.0
    }

    /// Language code as a usize
    pub fn as_usize(&self) -> usize {
        self.0.into()
    }

    /// Allowed value bounds (this is not really correct/useful)
    pub fn bounds() -> RangeInclusive<usize> {
        RangeInclusive::new(
            Self::try_from("aa").unwrap().as_usize(),
            Self::try_from("zz").unwrap().as_usize(),
        )
    }
}

impl TryFrom<&str> for Locale {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() == 2 {
            let bytes = value.as_bytes();
            let w = ((bytes[0] as u16) << 8) | bytes[1] as u16;
            Ok(Locale(w))
        } else {
            Err("language code must be exactly 2 characters, according to ISO 639-1")
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::try_from("en").unwrap()
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.0.to_be_bytes();
        let str = String::from_utf8_lossy(&bytes);
        f.write_str(&str)
    }
}

#[cfg(test)]
mod tests {
    use super::Locale;

    use proptest::prelude::*;
    use std::convert::TryFrom;

    const FR: u16 = 0x6672;

    fn ui_locale_strategy() -> impl Strategy<Value = Locale> {
        proptest::num::u16::ANY.prop_filter_map("Invalid UI locale code", |code| {
            let ui_locale = Locale(code);
            if ui_locale.to_string().is_ascii() {
                Some(ui_locale)
            } else {
                None
            }
        })
    }

    #[test]
    fn from_str_fr() {
        assert_eq!(Locale::try_from("fr").map(|code| code.as_u16()), Ok(FR));
    }

    #[test]
    fn from_str_empty() {
        assert!(Locale::try_from("").is_err())
    }

    #[test]
    fn from_str_too_long() {
        assert!(Locale::try_from("fra").is_err())
    }

    #[test]
    fn to_str() {
        assert_eq!(Locale(FR).to_string().as_str(), "fr")
    }

    proptest! {
        #[test]
        fn back_and_forth(ui_locale in ui_locale_strategy()) {
            let str = ui_locale.to_string();
            assert_eq!(Locale::try_from(str.as_str()).map(|code| code.as_u16()), Ok(ui_locale.as_u16()))
        }
    }
}
