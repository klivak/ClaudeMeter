use crate::theme::ResolvedTheme;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

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

/// Convert a COLORREF (0x00BBGGRR) to D2D1_COLOR_F (r,g,b,a as f32 in 0.0..1.0)
pub fn colorref_to_d2d(cr: ColorRef) -> D2D1_COLOR_F {
    let val = cr.0;
    D2D1_COLOR_F {
        r: (val & 0xFF) as f32 / 255.0,
        g: ((val >> 8) & 0xFF) as f32 / 255.0,
        b: ((val >> 16) & 0xFF) as f32 / 255.0,
        a: 1.0,
    }
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
    pub hover: ColorRef,
    pub border: ColorRef,
}

impl ThemeColors {
    pub fn for_theme(theme: ResolvedTheme) -> Self {
        match theme {
            ResolvedTheme::Dark => Self::dark(),
            ResolvedTheme::Light => Self::light(),
        }
    }

    /// Apply custom color overrides from config.
    pub fn with_overrides(mut self, custom: &crate::config::CustomColors) -> Self {
        fn apply(target: &mut ColorRef, value: &Option<String>) {
            if let Some(h) = value {
                let h = h.trim_start_matches('#');
                if h.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&h[0..2], 16),
                        u8::from_str_radix(&h[2..4], 16),
                        u8::from_str_radix(&h[4..6], 16),
                    ) {
                        *target = super::colors::rgb(r, g, b);
                    }
                }
            }
        }
        apply(&mut self.background, &custom.background);
        apply(&mut self.surface, &custom.surface);
        apply(&mut self.text_primary, &custom.text_primary);
        apply(&mut self.text_secondary, &custom.text_secondary);
        apply(&mut self.progress_bg, &custom.progress_bg);
        apply(&mut self.green, &custom.green);
        apply(&mut self.yellow, &custom.yellow);
        apply(&mut self.red, &custom.red);
        apply(&mut self.accent, &custom.accent);
        apply(&mut self.separator, &custom.separator);
        apply(&mut self.hover, &custom.hover);
        apply(&mut self.border, &custom.border);
        self
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
            hover: hex("3b3c50"),
            border: hex("45475a"),
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
            hover: hex("ced3dd"),
            border: hex("9ca0b0"),
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

/// Lighten a D2D1_COLOR_F by mixing towards white.
/// `amount` in 0.0..1.0 (0 = unchanged, 1 = white).
pub fn lighten_d2d(c: &D2D1_COLOR_F, amount: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: c.r + (1.0 - c.r) * amount,
        g: c.g + (1.0 - c.g) * amount,
        b: c.b + (1.0 - c.b) * amount,
        a: c.a,
    }
}
