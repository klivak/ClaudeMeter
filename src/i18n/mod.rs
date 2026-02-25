mod de;
mod en;
mod es;
mod fr;
mod uk;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locale {
    En,
    Uk,
    Es,
    De,
    Fr,
}

impl Locale {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "en" => Some(Self::En),
            "uk" => Some(Self::Uk),
            "es" => Some(Self::Es),
            "de" => Some(Self::De),
            "fr" => Some(Self::Fr),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Uk => "uk",
            Self::Es => "es",
            Self::De => "de",
            Self::Fr => "fr",
        }
    }

    /// Detect locale from Windows UI language (LANGID).
    pub fn detect_from_windows() -> Self {
        use windows::Win32::Globalization::GetUserDefaultUILanguage;
        let lang_id = unsafe { GetUserDefaultUILanguage() };
        // Primary language ID is the low 10 bits
        let primary = lang_id & 0x3FF;
        match primary {
            0x22 => Self::Uk, // Ukrainian
            0x0A => Self::Es, // Spanish
            0x07 => Self::De, // German
            0x0C => Self::Fr, // French
            _ => Self::En,
        }
    }
}

pub struct I18n {
    #[allow(dead_code)]
    locale: Locale,
    strings: HashMap<&'static str, &'static str>,
    fallback: HashMap<&'static str, &'static str>,
}

impl I18n {
    pub fn new(locale: Locale) -> Self {
        let strings = match locale {
            Locale::En => en::strings(),
            Locale::Uk => uk::strings(),
            Locale::Es => es::strings(),
            Locale::De => de::strings(),
            Locale::Fr => fr::strings(),
        };
        let fallback = en::strings();
        Self {
            locale,
            strings,
            fallback,
        }
    }

    pub fn from_config(language: &str) -> Self {
        let locale = if language == "auto" {
            Locale::detect_from_windows()
        } else {
            Locale::from_str(language).unwrap_or(Locale::En)
        };
        Self::new(locale)
    }

    /// Translate a key. Falls back to English, then returns the key itself.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings
            .get(key)
            .copied()
            .or_else(|| self.fallback.get(key).copied())
            .unwrap_or(key)
    }

    #[allow(dead_code)]
    pub fn locale(&self) -> Locale {
        self.locale
    }
}

/// Format a duration in seconds into a human-readable string.
/// e.g. 3661 → "1h 1m", 45 → "45s", 90000 → "1d 1h"
pub fn format_duration(seconds: i64) -> String {
    if seconds <= 0 {
        return "now".to_string();
    }
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else if mins > 0 {
        format!("{}m", mins)
    } else {
        format!("{}s", secs)
    }
}

/// Calculate seconds until a reset timestamp.
pub fn seconds_until(resets_at: &str) -> Option<i64> {
    use chrono::{DateTime, Utc};
    let reset: DateTime<Utc> = resets_at.parse().ok()?;
    let now = Utc::now();
    let diff = reset.signed_duration_since(now).num_seconds();
    Some(diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "now");
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(90000), "1d 1h");
    }

    #[test]
    fn test_fallback() {
        let i18n = I18n::new(Locale::En);
        assert_eq!(i18n.t("Plan"), "Plan");
        assert_eq!(i18n.t("nonexistent_key_xyz"), "nonexistent_key_xyz");
    }

    #[test]
    fn test_ukrainian() {
        let i18n = I18n::new(Locale::Uk);
        assert_eq!(i18n.t("Plan"), "План");
        assert_eq!(i18n.t("Pro"), "Pro"); // same in all languages
    }
}
