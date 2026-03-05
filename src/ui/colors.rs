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

/// Pleasant gradient colors for progress bars (green → amber → coral)
pub const GRADIENT_LOW: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 0.18,
    g: 0.80,
    b: 0.44,
    a: 1.0,
}; // #2ecc71
pub const GRADIENT_MID: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 0.95,
    g: 0.77,
    b: 0.06,
    a: 1.0,
}; // #f1c40f
pub const GRADIENT_HIGH: D2D1_COLOR_F = D2D1_COLOR_F {
    r: 0.91,
    g: 0.30,
    b: 0.24,
    a: 1.0,
}; // #e74c3c

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
    /// True when custom color overrides are applied (disables gradient progress bars)
    pub has_overrides: bool,
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
        let mut any_override = false;
        fn apply(target: &mut ColorRef, value: &Option<String>, changed: &mut bool) {
            if let Some(h) = value {
                let h = h.trim_start_matches('#');
                if h.len() == 6 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        u8::from_str_radix(&h[0..2], 16),
                        u8::from_str_radix(&h[2..4], 16),
                        u8::from_str_radix(&h[4..6], 16),
                    ) {
                        *target = super::colors::rgb(r, g, b);
                        *changed = true;
                    }
                }
            }
        }
        apply(&mut self.background, &custom.background, &mut any_override);
        apply(&mut self.surface, &custom.surface, &mut any_override);
        apply(
            &mut self.text_primary,
            &custom.text_primary,
            &mut any_override,
        );
        apply(
            &mut self.text_secondary,
            &custom.text_secondary,
            &mut any_override,
        );
        apply(
            &mut self.progress_bg,
            &custom.progress_bg,
            &mut any_override,
        );
        apply(&mut self.green, &custom.green, &mut any_override);
        apply(&mut self.yellow, &custom.yellow, &mut any_override);
        apply(&mut self.red, &custom.red, &mut any_override);
        apply(&mut self.accent, &custom.accent, &mut any_override);
        apply(&mut self.separator, &custom.separator, &mut any_override);
        apply(&mut self.hover, &custom.hover, &mut any_override);
        apply(&mut self.border, &custom.border, &mut any_override);
        if any_override {
            self.has_overrides = true;
        }
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
            has_overrides: false,
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
            has_overrides: false,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CustomColors;

    #[test]
    fn test_rgb_byte_order() {
        let c = rgb(0xFF, 0x00, 0x00); // red
        assert_eq!(c.0 & 0xFF, 0xFF); // R in low byte
        assert_eq!((c.0 >> 8) & 0xFF, 0x00); // G
        assert_eq!((c.0 >> 16) & 0xFF, 0x00); // B
    }

    #[test]
    fn test_hex_to_rgb_parsing() {
        assert_eq!(hex_to_rgb("#1e1e2e"), (0x1e, 0x1e, 0x2e));
        assert_eq!(hex_to_rgb("ff8800"), (0xff, 0x88, 0x00));
        assert_eq!(hex_to_rgb("#000000"), (0, 0, 0));
        assert_eq!(hex_to_rgb("#ffffff"), (255, 255, 255));
    }

    #[test]
    fn test_colorref_to_d2d_conversion() {
        let cr = rgb(255, 0, 0); // pure red
        let d2d = colorref_to_d2d(cr);
        assert!((d2d.r - 1.0).abs() < 0.01);
        assert!(d2d.g.abs() < 0.01);
        assert!(d2d.b.abs() < 0.01);
        assert!((d2d.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_progress_color_thresholds() {
        let colors = ThemeColors::for_theme(ResolvedTheme::Dark);
        // < 50% → green
        assert_eq!(colors.progress_color(0.0), colors.green);
        assert_eq!(colors.progress_color(49.9), colors.green);
        // 50-79% → yellow
        assert_eq!(colors.progress_color(50.0), colors.yellow);
        assert_eq!(colors.progress_color(79.9), colors.yellow);
        // >= 80% → red
        assert_eq!(colors.progress_color(80.0), colors.red);
        assert_eq!(colors.progress_color(100.0), colors.red);
    }

    #[test]
    fn test_dark_light_themes_differ() {
        let dark = ThemeColors::for_theme(ResolvedTheme::Dark);
        let light = ThemeColors::for_theme(ResolvedTheme::Light);
        assert_ne!(dark.background.0, light.background.0);
        assert_ne!(dark.text_primary.0, light.text_primary.0);
        assert_ne!(dark.accent.0, light.accent.0);
    }

    #[test]
    fn test_has_overrides_default_false() {
        let dark = ThemeColors::for_theme(ResolvedTheme::Dark);
        assert!(!dark.has_overrides);
        let light = ThemeColors::for_theme(ResolvedTheme::Light);
        assert!(!light.has_overrides);
    }

    #[test]
    fn test_with_overrides_sets_flag() {
        let colors = ThemeColors::for_theme(ResolvedTheme::Dark);
        let custom = CustomColors {
            background: Some("#ff0000".to_string()),
            ..Default::default()
        };
        let overridden = colors.with_overrides(&custom);
        assert!(overridden.has_overrides);
        assert_eq!(overridden.background, rgb(255, 0, 0));
    }

    #[test]
    fn test_with_overrides_no_changes_keeps_false() {
        let colors = ThemeColors::for_theme(ResolvedTheme::Dark);
        let custom = CustomColors::default(); // all None
        let result = colors.with_overrides(&custom);
        assert!(!result.has_overrides);
    }

    #[test]
    fn test_with_overrides_invalid_hex_ignored() {
        let colors = ThemeColors::for_theme(ResolvedTheme::Dark);
        let original_bg = colors.background;
        let custom = CustomColors {
            background: Some("xyz".to_string()), // invalid
            ..Default::default()
        };
        let result = colors.with_overrides(&custom);
        assert_eq!(result.background, original_bg);
        assert!(!result.has_overrides);
    }

    #[test]
    fn test_gradient_constants_valid_range() {
        for c in [&GRADIENT_LOW, &GRADIENT_MID, &GRADIENT_HIGH] {
            assert!((0.0..=1.0).contains(&c.r));
            assert!((0.0..=1.0).contains(&c.g));
            assert!((0.0..=1.0).contains(&c.b));
            assert!((c.a - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_lighten_d2d() {
        let c = D2D1_COLOR_F {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };
        let lightened = lighten_d2d(&c, 0.5);
        assert!((lightened.r - 0.75).abs() < 0.01);
        // amount=1.0 → white
        let white = lighten_d2d(&c, 1.0);
        assert!((white.r - 1.0).abs() < 0.01);
        // amount=0.0 → unchanged
        let same = lighten_d2d(&c, 0.0);
        assert!((same.r - 0.5).abs() < 0.01);
    }
}
