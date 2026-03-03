mod de;
mod en;
mod es;
mod fr;
mod hi;
mod it;
mod ja;
mod ko;
mod nl;
mod pl;
mod pt;
mod tr;
mod uk;
mod vi;
mod zh;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Locale {
    En,
    Uk,
    Es,
    De,
    Fr,
    Pt,
    Ja,
    Ko,
    Zh,
    It,
    Hi,
    Tr,
    Nl,
    Pl,
    Vi,
}

impl Locale {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "en" => Some(Self::En),
            "uk" => Some(Self::Uk),
            "es" => Some(Self::Es),
            "de" => Some(Self::De),
            "fr" => Some(Self::Fr),
            "pt" => Some(Self::Pt),
            "ja" => Some(Self::Ja),
            "ko" => Some(Self::Ko),
            "zh" => Some(Self::Zh),
            "it" => Some(Self::It),
            "hi" => Some(Self::Hi),
            "tr" => Some(Self::Tr),
            "nl" => Some(Self::Nl),
            "pl" => Some(Self::Pl),
            "vi" => Some(Self::Vi),
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
            Self::Pt => "pt",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::Zh => "zh",
            Self::It => "it",
            Self::Hi => "hi",
            Self::Tr => "tr",
            Self::Nl => "nl",
            Self::Pl => "pl",
            Self::Vi => "vi",
        }
    }

    /// Human-readable name for display in settings.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Uk => {
                "\u{0423}\u{043a}\u{0440}\u{0430}\u{0457}\u{043d}\u{0441}\u{044c}\u{043a}\u{0430}"
            }
            Self::Es => "Espa\u{00f1}ol",
            Self::De => "Deutsch",
            Self::Fr => "Fran\u{00e7}ais",
            Self::Pt => "Portugu\u{00ea}s",
            Self::Ja => "\u{65e5}\u{672c}\u{8a9e}",
            Self::Ko => "\u{d55c}\u{ad6d}\u{c5b4}",
            Self::Zh => "\u{4e2d}\u{6587}",
            Self::It => "Italiano",
            Self::Hi => "\u{0939}\u{093f}\u{0928}\u{094d}\u{0926}\u{0940}",
            Self::Tr => "T\u{00fc}rk\u{00e7}e",
            Self::Nl => "Nederlands",
            Self::Pl => "Polski",
            Self::Vi => "Ti\u{1ebf}ng Vi\u{1ec7}t",
        }
    }

    /// All available locales in order.
    pub fn all() -> &'static [Self] {
        &[
            Self::En,
            Self::Uk,
            Self::Es,
            Self::De,
            Self::Fr,
            Self::Pt,
            Self::It,
            Self::Hi,
            Self::Tr,
            Self::Nl,
            Self::Pl,
            Self::Vi,
            Self::Ja,
            Self::Ko,
            Self::Zh,
        ]
    }

    /// Next locale in cycling order (for settings).
    pub fn next(&self) -> Option<Self> {
        let all = Self::all();
        let idx = all.iter().position(|l| l == self)?;
        if idx + 1 < all.len() {
            Some(all[idx + 1])
        } else {
            None // wraps to "auto"
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
            0x16 => Self::Pt, // Portuguese
            0x11 => Self::Ja, // Japanese
            0x12 => Self::Ko, // Korean
            0x04 => Self::Zh, // Chinese
            0x10 => Self::It, // Italian
            0x39 => Self::Hi, // Hindi
            0x1F => Self::Tr, // Turkish
            0x13 => Self::Nl, // Dutch
            0x15 => Self::Pl, // Polish
            0x2A => Self::Vi, // Vietnamese
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
            Locale::Pt => pt::strings(),
            Locale::Ja => ja::strings(),
            Locale::Ko => ko::strings(),
            Locale::Zh => zh::strings(),
            Locale::It => it::strings(),
            Locale::Hi => hi::strings(),
            Locale::Tr => tr::strings(),
            Locale::Nl => nl::strings(),
            Locale::Pl => pl::strings(),
            Locale::Vi => vi::strings(),
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

/// Format the reset timestamp as a local target time.
/// Respects Windows 12h/24h system setting.
/// 24h: "(Thu 14:00)" or "(14:00)". 12h: "(Thu 2:00 PM)" or "(2:00 PM)".
pub fn format_reset_target(resets_at: &str) -> Option<String> {
    use chrono::{DateTime, Local, Utc};
    let reset_utc: DateTime<Utc> = resets_at.parse().ok()?;
    let reset_local = reset_utc.with_timezone(&Local);
    let now_local = Local::now();
    let time_fmt = if is_system_24h() { "%H:%M" } else { "%I:%M %p" };
    if reset_local.date_naive() == now_local.date_naive() {
        Some(format!("({})", reset_local.format(time_fmt)))
    } else {
        Some(format!(
            "({} {})",
            reset_local.format("%a"),
            reset_local.format(time_fmt)
        ))
    }
}

/// Detect Windows 12h vs 24h clock from registry (Control Panel\International\iTime).
/// Returns true for 24h format. Defaults to 24h if registry read fails.
pub fn is_system_24h() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
    };

    const KEY: &str = "Control Panel\\International";
    const VALUE: &str = "iTime";

    let key_wide: Vec<u16> = KEY.encode_utf16().chain(std::iter::once(0)).collect();
    let value_wide: Vec<u16> = VALUE.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_wide.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return true;
        }

        let mut buf = [0u16; 4];
        let mut data_size = (buf.len() * 2) as u32;
        let mut data_type = REG_SZ;

        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_wide.as_ptr()),
            None,
            Some(&mut data_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey);

        if result.is_ok() && data_size >= 2 {
            // "1" = 24h, "0" = 12h
            buf[0] == b'1' as u16
        } else {
            true
        }
    }
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
