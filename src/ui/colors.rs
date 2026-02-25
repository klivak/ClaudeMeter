use crate::theme::ResolvedTheme;
use windows::Win32::Foundation::COLORREF;

pub type ColorRef = COLORREF;

pub fn rgb(r: u8, g: u8, b: u8) -> ColorRef {
    COLORREF((b as u32) << 16 | (g as u32) << 8 | r as u32)
}

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn hex(hex: &str) -> ColorRef {
    let (r, g, b) = hex_to_rgb(hex);
    rgb(r, g, b)
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub background: ColorRef,
    pub surface: ColorRef,
    pub text_primary: ColorRef,
    pub text_secondary: ColorRef,
    pub progress_bg: ColorRef,
    pub green: ColorRef,
    pub yellow: ColorRef,
    pub red: ColorRef,
    pub accent: ColorRef,
    pub separator: ColorRef,
}

impl ThemeColors {
    pub fn for_theme(theme: ResolvedTheme) -> Self {
        match theme {
            ResolvedTheme::Dark => Self::dark(),
            ResolvedTheme::Light => Self::light(),
        }
    }

    fn dark() -> Self {
        Self {
            background: hex("1e1e2e"),
            surface: hex("313244"),
            text_primary: hex("cdd6f4"),
            text_secondary: hex("a6adc8"),
            progress_bg: hex("45475a"),
            green: hex("40a02b"),
            yellow: hex("df8e1d"),
            red: hex("d20f39"),
            accent: hex("89b4fa"),
            separator: hex("45475a"),
        }
    }

    fn light() -> Self {
        Self {
            background: hex("eff1f5"),
            surface: hex("dce0e8"),
            text_primary: hex("4c4f69"),
            text_secondary: hex("6c6f85"),
            progress_bg: hex("bcc0cc"),
            green: hex("40a02b"),
            yellow: hex("df8e1d"),
            red: hex("d20f39"),
            accent: hex("1e66f5"),
            separator: hex("bcc0cc"),
        }
    }

    pub fn progress_color(&self, utilization: f64) -> ColorRef {
        if utilization >= 80.0 {
            self.red
        } else if utilization >= 50.0 {
            self.yellow
        } else {
            self.green
        }
    }
}
