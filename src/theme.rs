use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER, KEY_READ, REG_DWORD,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
    Auto,
}

impl ThemeMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => Self::Dark,
            "light" => Self::Light,
            _ => Self::Auto,
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::Auto => "auto",
        }
    }
}

/// Resolve the effective theme (Dark or Light) accounting for Auto mode.
pub fn resolve_theme(mode: ThemeMode) -> ResolvedTheme {
    match mode {
        ThemeMode::Dark => ResolvedTheme::Dark,
        ThemeMode::Light => ResolvedTheme::Light,
        ThemeMode::Auto => {
            if is_system_light_theme() {
                ResolvedTheme::Light
            } else {
                ResolvedTheme::Dark
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResolvedTheme {
    Dark,
    Light,
}

/// Read Windows registry to determine system theme.
/// Returns true if Windows is in light mode.
pub fn is_system_light_theme() -> bool {
    const KEY: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    const VALUE: &str = "AppsUseLightTheme";

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
            return false; // Default dark
        }

        let mut data: u32 = 0;
        let mut data_size = std::mem::size_of::<u32>() as u32;
        let mut data_type = REG_DWORD;

        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_wide.as_ptr()),
            None,
            Some(&mut data_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );

        let _ = RegCloseKey(hkey).ok();

        if result.is_ok() {
            data != 0
        } else {
            false
        }
    }
}
