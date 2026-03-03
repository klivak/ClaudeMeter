/// Rendering helpers for the popup window using Direct2D + DirectWrite.
///
/// Replaces the legacy GDI rendering with hardware-accelerated Direct2D:
/// - ID2D1HwndRenderTarget for all drawing (auto double-buffered)
/// - IDWriteTextFormat for high-quality ClearType text
/// - Antialiased rounded rectangles, ellipses, lines
///
/// All coordinates are in DIPs (device-independent pixels, 1 DIP = 1/96 inch).
use crate::i18n::{format_duration, format_reset_target, is_system_24h, seconds_until, I18n};
use crate::providers::claude::{format_metric_name, UsageResponse};
use crate::ui::colors::{colorref_to_d2d, lighten_d2d, ThemeColors};
use std::collections::HashMap;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct2D::Common::{
    D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_GRADIENT_STOP, D2D1_PIXEL_FORMAT,
    D2D_POINT_2F, D2D_RECT_F, D2D_SIZE_U,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1CreateFactory, ID2D1Factory, ID2D1HwndRenderTarget, D2D1_DRAW_TEXT_OPTIONS_NONE,
    D2D1_ELLIPSE, D2D1_EXTEND_MODE_CLAMP, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_GAMMA_2_2,
    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
    D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES, D2D1_ROUNDED_RECT,
};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED,
    DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_BOLD,
    DWRITE_FONT_WEIGHT_REGULAR, DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
    DWRITE_PARAGRAPH_ALIGNMENT_NEAR, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING,
    DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_WORD_WRAPPING_WRAP,
};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, ReleaseDC, LOGPIXELSX};

pub const POPUP_WIDTH: i32 = 380;
pub const HEADER_HEIGHT: i32 = 40;
pub const PADDING: i32 = 16;
pub const METRIC_LABEL_H: i32 = 22;
pub const PROGRESS_H: i32 = 14;
pub const RESET_LABEL_H: i32 = 18;
pub const SECTION_GAP: i32 = 14;
pub const ITEM_GAP: i32 = 8;
pub const SEPARATOR_H: i32 = 1;
pub const FOOTER_H: i32 = 38;

// --- HoveredElement enum ---

#[derive(Debug, Clone, PartialEq)]
pub enum HoveredElement {
    None,
    SettingsButton,
    CloseButton,
    RefreshButton,
    InstallButton,
    ChatGptLink,
    StatusLink,
    BackButton,
    SettingRow(usize),
    ChartBar(usize),
}

/// Draw accessibility overlay pattern on a progress bar fill.
/// - Green (<50%): fine dots
/// - Yellow (50-79%): diagonal stripes ///
/// - Red (>=80%): cross-hatch
unsafe fn draw_accessibility_pattern(
    rt: &ID2D1HwndRenderTarget,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    utilization: f64,
) {
    if right - left < 2.0 {
        return;
    }
    let pattern_color = D2D1_COLOR_F {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.35,
    };
    let Ok(brush) = rt.CreateSolidColorBrush(&pattern_color as *const _, None) else {
        return;
    };
    // Push clip to keep patterns inside the bar
    rt.PushAxisAlignedClip(
        &D2D_RECT_F {
            left,
            top,
            right,
            bottom,
        },
        windows::Win32::Graphics::Direct2D::D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
    );
    let h = bottom - top;
    if utilization >= 80.0 {
        // Cross-hatch: two sets of diagonal lines
        let spacing = h * 0.6;
        let span = right - left + h;
        let n = (span / spacing) as i32 + 1;
        for i in 0..n {
            let offset = left - h + i as f32 * spacing;
            rt.DrawLine(
                D2D_POINT_2F {
                    x: offset,
                    y: bottom,
                },
                D2D_POINT_2F {
                    x: offset + h,
                    y: top,
                },
                &brush,
                1.0,
                None,
            );
            rt.DrawLine(
                D2D_POINT_2F { x: offset, y: top },
                D2D_POINT_2F {
                    x: offset + h,
                    y: bottom,
                },
                &brush,
                1.0,
                None,
            );
        }
    } else if utilization >= 50.0 {
        // Diagonal stripes
        let spacing = h * 0.8;
        let span = right - left + h;
        let n = (span / spacing) as i32 + 1;
        for i in 0..n {
            let offset = left - h + i as f32 * spacing;
            rt.DrawLine(
                D2D_POINT_2F {
                    x: offset,
                    y: bottom,
                },
                D2D_POINT_2F {
                    x: offset + h,
                    y: top,
                },
                &brush,
                1.0,
                None,
            );
        }
    } else {
        // Fine dots for green
        let dot_spacing = h * 0.7;
        let mut dx = left + dot_spacing / 2.0;
        let mut row = 0;
        while dx < right {
            let cy = top + h / 2.0 + if row % 2 == 0 { -h * 0.15 } else { h * 0.15 };
            rt.FillEllipse(
                &D2D1_ELLIPSE {
                    point: D2D_POINT_2F { x: dx, y: cy },
                    radiusX: 1.0,
                    radiusY: 1.0,
                },
                &brush,
            );
            dx += dot_spacing;
            row += 1;
        }
    }
    rt.PopAxisAlignedClip();
}

// --- Text format cache key ---

#[derive(Hash, Eq, PartialEq, Clone)]
struct TextFormatKey {
    size_pt: i32,
    bold: bool,
    h_align: u8, // 0=leading, 1=trailing, 2=center
    v_align: u8, // 0=near, 1=center
}

// --- D2DResources ---

pub struct D2DResources {
    pub factory: ID2D1Factory,
    pub dwrite_factory: IDWriteFactory,
    pub render_target: Option<ID2D1HwndRenderTarget>,
    text_formats: HashMap<TextFormatKey, IDWriteTextFormat>,
}

impl D2DResources {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
                .map_err(|e| format!("D2D1CreateFactory failed: {e}"))?;

            let dwrite_factory: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)
                .map_err(|e| format!("DWriteCreateFactory failed: {e}"))?;

            Ok(Self {
                factory,
                dwrite_factory,
                render_target: None,
                text_formats: HashMap::new(),
            })
        }
    }

    pub fn ensure_render_target(&mut self, hwnd: HWND) -> Result<(), String> {
        if self.render_target.is_some() {
            return Ok(());
        }
        unsafe {
            let mut rc = RECT::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rc);

            let size = D2D_SIZE_U {
                width: (rc.right - rc.left).max(1) as u32,
                height: (rc.bottom - rc.top).max(1) as u32,
            };

            let pixel_format = D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            };

            let rt_props = D2D1_RENDER_TARGET_PROPERTIES {
                pixelFormat: pixel_format,
                dpiX: 96.0,
                dpiY: 96.0,
                ..Default::default()
            };

            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: size,
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let rt = self
                .factory
                .CreateHwndRenderTarget(&rt_props, &hwnd_props)
                .map_err(|e| format!("CreateHwndRenderTarget failed: {e}"))?;

            self.render_target = Some(rt);
        }
        Ok(())
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if let Some(rt) = self.render_target.as_ref() {
            let size = D2D_SIZE_U {
                width: w.max(1),
                height: h.max(1),
            };
            unsafe {
                let _ = rt.Resize(&size);
            }
        }
    }

    pub fn discard_render_target(&mut self) {
        self.render_target = None;
    }

    /// Release all GPU/COM resources to reclaim memory when popup is hidden.
    pub fn release(&mut self) {
        self.render_target = None;
        self.text_formats.clear();
    }

    fn get_text_format(
        &mut self,
        size_pt: i32,
        bold: bool,
        h_align: u8,
        v_align: u8,
    ) -> &IDWriteTextFormat {
        let key = TextFormatKey {
            size_pt,
            bold,
            h_align,
            v_align,
        };
        if !self.text_formats.contains_key(&key) {
            let weight = if bold {
                DWRITE_FONT_WEIGHT_BOLD
            } else {
                DWRITE_FONT_WEIGHT_REGULAR
            };
            // Convert pt to DIPs: 1pt = 96/72 DIPs
            let font_size = size_pt as f32 * 96.0 / 72.0;
            let font_name = wide("Segoe UI");
            let locale = wide("en-us");
            let format = unsafe {
                self.dwrite_factory
                    .CreateTextFormat(
                        PCWSTR(font_name.as_ptr()),
                        None,
                        weight,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        font_size,
                        PCWSTR(locale.as_ptr()),
                    )
                    .expect("CreateTextFormat failed")
            };
            // Set alignment
            unsafe {
                let h = match h_align {
                    1 => DWRITE_TEXT_ALIGNMENT_TRAILING,
                    2 => DWRITE_TEXT_ALIGNMENT_CENTER,
                    _ => DWRITE_TEXT_ALIGNMENT_LEADING,
                };
                let v = match v_align {
                    1 => DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                    _ => DWRITE_PARAGRAPH_ALIGNMENT_NEAR,
                };
                let _ = format.SetTextAlignment(h);
                let _ = format.SetParagraphAlignment(v);
            }
            self.text_formats.insert(key.clone(), format);
        }
        self.text_formats.get(&key).unwrap()
    }

    /// Get a text format for word-wrapping text
    fn get_text_format_wrap(&mut self, size_pt: i32, bold: bool) -> IDWriteTextFormat {
        let weight = if bold {
            DWRITE_FONT_WEIGHT_BOLD
        } else {
            DWRITE_FONT_WEIGHT_REGULAR
        };
        let font_size = size_pt as f32 * 96.0 / 72.0;
        let font_name = wide("Segoe UI");
        let locale = wide("en-us");
        let format = unsafe {
            self.dwrite_factory
                .CreateTextFormat(
                    PCWSTR(font_name.as_ptr()),
                    None,
                    weight,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    font_size,
                    PCWSTR(locale.as_ptr()),
                )
                .expect("CreateTextFormat failed")
        };
        unsafe {
            let _ = format.SetWordWrapping(DWRITE_WORD_WRAPPING_WRAP);
        }
        format
    }
}

// --- PopupRenderer ---

pub struct PopupRenderer {
    pub dpi_scale: f32,
}

impl PopupRenderer {
    pub fn new(hwnd: HWND) -> Self {
        let dpi_scale = unsafe {
            let hdc = GetDC(hwnd);
            let dpi = GetDeviceCaps(hdc, LOGPIXELSX);
            ReleaseDC(hwnd, hdc);
            dpi as f32 / 96.0
        };
        Self { dpi_scale }
    }

    fn scale(&self, px: i32) -> i32 {
        (px as f32 * self.dpi_scale) as i32
    }

    fn sf(&self, px: i32) -> f32 {
        px as f32 * self.dpi_scale
    }

    /// Calculate the total height needed for the popup in full mode.
    /// Must exactly mirror the layout in draw() to prevent clipping.
    pub fn calculate_height(
        &self,
        usage: &Option<UsageResponse>,
        show_chatgpt: bool,
        compact: bool,
    ) -> i32 {
        if compact {
            let metric_count = usage
                .as_ref()
                .map(|u| u.all_metrics().len())
                .unwrap_or(1)
                .max(1) as i32;
            return self.scale(
                HEADER_HEIGHT
                    + SEPARATOR_H
                    + PADDING
                    + metric_count * (16 + 8 + ITEM_GAP)
                    + PADDING
                    + SEPARATOR_H
                    + FOOTER_H,
            );
        }

        let mut h = 0;
        h += HEADER_HEIGHT;
        h += SEPARATOR_H;
        h += PADDING;

        match usage {
            None => {
                h += 28 + 70 + 28 + 8;
            }
            Some(u) => {
                h += 24;
                let metric_count = u.all_metrics().len() as i32;
                h += metric_count
                    * (METRIC_LABEL_H + 4 + PROGRESS_H + 4 + RESET_LABEL_H + SECTION_GAP);
            }
        }

        if show_chatgpt {
            h += SEPARATOR_H + 8 + 24 + 55 + 28;
        }

        h += SEPARATOR_H + PADDING;
        h += 22 + 70 + 4 + 14;
        h += PADDING;
        h += SEPARATOR_H + FOOTER_H;

        self.scale(h)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        d2d: &mut D2DResources,
        rect: &RECT,
        usage: &Option<UsageResponse>,
        last_updated: &str,
        show_chatgpt: bool,
        compact: bool,
        colors: &ThemeColors,
        i18n: &I18n,
        chart_data: &[f64],
        reset_lines: &[f64],
        last_error: &Option<String>,
        hovered: &HoveredElement,
        anim_values: &[f64],
        settings_rect: &mut RECT,
        close_rect: &mut RECT,
        refresh_rect: &mut RECT,
        install_rect: &mut RECT,
        chatgpt_link_rect: &mut RECT,
        status_link_rect: &mut RECT,
        chart_rect_out: &mut RECT,
        chart_bar_count_out: &mut usize,
    ) {
        let Some(rt) = d2d.render_target.clone() else {
            return;
        };
        let w = (rect.right - rect.left) as f32;

        unsafe {
            let mut y = 0.0f32;

            // Header
            y = self.draw_header(
                &rt,
                d2d,
                w,
                y,
                colors,
                i18n,
                hovered,
                settings_rect,
                close_rect,
            );

            // Separator
            y = self.draw_separator(&rt, w, y, colors);

            y += self.sf(PADDING);

            if compact {
                y = self.draw_compact_metrics(&rt, d2d, w, y, usage, colors);
            } else {
                match usage {
                    None => {
                        y = self.draw_not_detected(
                            &rt,
                            d2d,
                            w,
                            y,
                            colors,
                            i18n,
                            last_error,
                            install_rect,
                        );
                    }
                    Some(u) => {
                        y = self.draw_claude_section(
                            &rt,
                            d2d,
                            w,
                            y,
                            u,
                            colors,
                            i18n,
                            anim_values,
                            hovered,
                            status_link_rect,
                        );
                    }
                }

                if show_chatgpt {
                    y = self.draw_separator(&rt, w, y, colors);
                    y += self.sf(8);
                    y = self.draw_chatgpt_section(&rt, d2d, w, y, colors, i18n, chatgpt_link_rect);
                }

                // History chart
                y = self.draw_separator(&rt, w, y, colors);
                y += self.sf(PADDING);
                y = self.draw_chart(
                    &rt,
                    d2d,
                    w,
                    y,
                    chart_data,
                    reset_lines,
                    colors,
                    i18n,
                    hovered,
                    chart_rect_out,
                    chart_bar_count_out,
                );
                y += self.sf(PADDING);
            }

            // Footer
            self.draw_separator(&rt, w, y, colors);
            self.draw_footer(
                &rt,
                d2d,
                w,
                y + self.sf(SEPARATOR_H),
                last_updated,
                colors,
                i18n,
                hovered,
                refresh_rect,
                status_link_rect,
            );

            // 1px border
            self.draw_border(&rt, w, (rect.bottom - rect.top) as f32, colors);
        }
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_header(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        y: f32,
        colors: &ThemeColors,
        i18n: &I18n,
        hovered: &HoveredElement,
        settings_rect: &mut RECT,
        close_rect: &mut RECT,
    ) -> f32 {
        let h = self.sf(HEADER_HEIGHT);
        let pad = self.sf(PADDING);

        // Header background
        let surface_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.surface) as *const _, None)
            .unwrap();
        rt.FillRectangle(
            &D2D_RECT_F {
                left: 0.0,
                top: y,
                right: w,
                bottom: y + h,
            },
            &surface_brush,
        );

        // Title "ClaudeMeter"
        let title_text = wide(i18n.t("ClaudeMeter"));
        let title_format = d2d.get_text_format(14, true, 0, 1).clone();
        let title_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &title_text[..title_text.len() - 1],
            &title_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - self.sf(80),
                bottom: y + h,
            },
            &title_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        // ⚙ button
        let btn_size = self.sf(28);
        let btn_y = y + (h - btn_size) / 2.0;
        let settings_r = D2D_RECT_F {
            left: w - self.sf(64),
            top: btn_y,
            right: w - self.sf(36),
            bottom: btn_y + btn_size,
        };
        *settings_rect = to_win32_rect(&settings_r);

        // Hover highlight for settings button
        if matches!(hovered, HoveredElement::SettingsButton) {
            let hover_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.hover) as *const _, None)
                .unwrap();
            rt.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: settings_r,
                    radiusX: 4.0,
                    radiusY: 4.0,
                },
                &hover_brush,
            );
        }

        self.draw_text_centered(
            rt,
            d2d,
            "\u{2699}",
            settings_r,
            colors.text_secondary,
            14,
            false,
        );

        // × button
        let close_r = D2D_RECT_F {
            left: w - self.sf(32),
            top: btn_y,
            right: w - self.sf(4),
            bottom: btn_y + btn_size,
        };
        *close_rect = to_win32_rect(&close_r);

        // Hover highlight for close button
        if matches!(hovered, HoveredElement::CloseButton) {
            let hover_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.hover) as *const _, None)
                .unwrap();
            rt.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: close_r,
                    radiusX: 4.0,
                    radiusY: 4.0,
                },
                &hover_brush,
            );
        }

        self.draw_text_centered(
            rt,
            d2d,
            "\u{00D7}",
            close_r,
            colors.text_secondary,
            14,
            false,
        );

        y + h
    }

    unsafe fn draw_separator(
        &self,
        rt: &ID2D1HwndRenderTarget,
        w: f32,
        y: f32,
        colors: &ThemeColors,
    ) -> f32 {
        let brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.separator) as *const _, None)
            .unwrap();
        rt.DrawLine(
            D2D_POINT_2F { x: 0.0, y },
            D2D_POINT_2F { x: w, y },
            &brush,
            1.0,
            None,
        );
        y + self.sf(SEPARATOR_H)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_claude_section(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        mut y: f32,
        usage: &UsageResponse,
        colors: &ThemeColors,
        i18n: &I18n,
        anim_values: &[f64],
        hovered: &HoveredElement,
        status_link_rect: &mut RECT,
    ) -> f32 {
        let pad = self.sf(PADDING);

        // Section header: "☁ CLAUDE · Pro plan"
        let detected = usage.detected_plan();
        let plan = i18n.t(&detected);
        let header_str = format!(
            "\u{2601} {} \u{00B7} {} {}",
            i18n.t("CLAUDE"),
            i18n.t("Plan"),
            plan
        );

        let header_text = wide(&header_str);
        let format = d2d.get_text_format(12, true, 0, 1).clone();
        let brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &header_text[..header_text.len() - 1],
            &format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad - self.sf(60),
                bottom: y + self.sf(20),
            },
            &brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        // "Status ↗" link (right-aligned on header line)
        let status_str = format!("{} \u{2197}", i18n.t("Status"));
        let status_text = wide(&status_str);
        let status_format = d2d.get_text_format(10, false, 1, 1).clone();
        let is_status_hovered = matches!(hovered, HoveredElement::StatusLink);
        let status_color = if is_status_hovered {
            lighten_d2d(&colorref_to_d2d(colors.accent), 0.3)
        } else {
            colorref_to_d2d(colors.accent)
        };
        let status_brush = rt
            .CreateSolidColorBrush(&status_color as *const _, None)
            .unwrap();
        let sr = D2D_RECT_F {
            left: w - pad - self.sf(56),
            top: y,
            right: w - pad,
            bottom: y + self.sf(20),
        };
        *status_link_rect = to_win32_rect(&sr);
        rt.DrawText(
            &status_text[..status_text.len() - 1],
            &status_format,
            &sr,
            &status_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        y += self.sf(24);

        for (i, (key, metric)) in usage.all_metrics().iter().enumerate() {
            let util = if i < anim_values.len() {
                anim_values[i]
            } else {
                metric.utilization
            };
            y = self.draw_metric(
                rt,
                d2d,
                w,
                y,
                key,
                util,
                metric.resets_at.as_deref(),
                colors,
                i18n,
            );
            y += self.sf(SECTION_GAP);
        }

        y
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_metric(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        mut y: f32,
        key: &str,
        utilization: f64,
        resets_at: Option<&str>,
        colors: &ThemeColors,
        i18n: &I18n,
    ) -> f32 {
        let pad = self.sf(PADDING);
        let content_w = w - pad * 2.0;
        let bar_h = self.sf(PROGRESS_H);
        let radius = bar_h / 2.0;

        // Label + percentage on same line
        let metric_name_str = format_metric_name(key);
        let display_name = i18n.t(&metric_name_str);
        let pct_str = format!("{:.0}%", utilization);

        // Label (left)
        let label_text = wide(display_name);
        let label_format = d2d.get_text_format(12, false, 0, 1).clone();
        let label_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &label_text[..label_text.len() - 1],
            &label_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad - self.sf(50),
                bottom: y + self.sf(METRIC_LABEL_H),
            },
            &label_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        // Percentage (right, bold, colored)
        let pct_text = wide(&pct_str);
        let pct_format = d2d.get_text_format(12, true, 1, 1).clone();
        let pct_color = colorref_to_d2d(colors.progress_color(utilization));
        let pct_brush = rt
            .CreateSolidColorBrush(&pct_color as *const _, None)
            .unwrap();
        rt.DrawText(
            &pct_text[..pct_text.len() - 1],
            &pct_format,
            &D2D_RECT_F {
                left: w - pad - self.sf(50),
                top: y,
                right: w - pad,
                bottom: y + self.sf(METRIC_LABEL_H),
            },
            &pct_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        y += self.sf(METRIC_LABEL_H + 4);

        // Progress bar background (rounded)
        let bg_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.progress_bg) as *const _, None)
            .unwrap();
        rt.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: pad,
                    top: y,
                    right: pad + content_w,
                    bottom: y + bar_h,
                },
                radiusX: radius,
                radiusY: radius,
            },
            &bg_brush,
        );

        // Progress bar fill (rounded, gradient)
        let fill_w = (content_w * utilization as f32 / 100.0)
            .max(0.0)
            .min(content_w);
        if fill_w > 0.5 {
            let fill_color = colorref_to_d2d(colors.progress_color(utilization));
            let light_color = lighten_d2d(&fill_color, 0.35);
            let stops = [
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: fill_color,
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: light_color,
                },
            ];
            let fill_rect = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: pad,
                    top: y,
                    right: pad + fill_w,
                    bottom: y + bar_h,
                },
                radiusX: radius.min(fill_w / 2.0),
                radiusY: radius,
            };
            if let Ok(stop_coll) =
                rt.CreateGradientStopCollection(&stops, D2D1_GAMMA_2_2, D2D1_EXTEND_MODE_CLAMP)
            {
                let grad_props = D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                    startPoint: D2D_POINT_2F { x: pad, y: 0.0 },
                    endPoint: D2D_POINT_2F {
                        x: pad + fill_w,
                        y: 0.0,
                    },
                };
                if let Ok(grad_brush) = rt.CreateLinearGradientBrush(&grad_props, None, &stop_coll)
                {
                    rt.FillRoundedRectangle(&fill_rect, &grad_brush);
                }
            }
            // Accessibility overlay pattern
            if crate::APP_STATE
                .as_ref()
                .is_some_and(|s| s.config_mgr.config.accessibility_patterns)
            {
                draw_accessibility_pattern(rt, pad, y, pad + fill_w, y + bar_h, utilization);
            }
        }

        y += bar_h + self.sf(4);

        // Reset time
        if let Some(reset_str) = resets_at {
            let reset_text = if let Some(secs) = seconds_until(reset_str) {
                if secs > 0 {
                    let duration = format_duration(secs);
                    let target = format_reset_target(reset_str).unwrap_or_default();
                    format!("{} {} {}", i18n.t("resets in"), duration, target)
                } else {
                    "resetting soon".to_string()
                }
            } else {
                String::new()
            };

            if !reset_text.is_empty() {
                let text = wide(&reset_text);
                let format = d2d.get_text_format(10, false, 0, 1).clone();
                let brush = rt
                    .CreateSolidColorBrush(
                        &colorref_to_d2d(colors.text_secondary) as *const _,
                        None,
                    )
                    .unwrap();
                rt.DrawText(
                    &text[..text.len() - 1],
                    &format,
                    &D2D_RECT_F {
                        left: pad,
                        top: y,
                        right: w - pad,
                        bottom: y + self.sf(RESET_LABEL_H),
                    },
                    &brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
                y += self.sf(RESET_LABEL_H);
            }
        }

        y
    }

    unsafe fn draw_compact_metrics(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        mut y: f32,
        usage: &Option<UsageResponse>,
        colors: &ThemeColors,
    ) -> f32 {
        let pad = self.sf(PADDING);
        let content_w = w - pad * 2.0;

        let metrics: Vec<(String, f64)> = match usage {
            Some(u) => u
                .all_metrics()
                .iter()
                .map(|(k, m)| (format_metric_name(k), m.utilization))
                .collect(),
            None => vec![("No data".to_string(), 0.0)],
        };

        for (name, utilization) in &metrics {
            // Label
            let label_text = wide(name);
            let label_format = d2d.get_text_format(11, false, 0, 1).clone();
            let label_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
                .unwrap();
            rt.DrawText(
                &label_text[..label_text.len() - 1],
                &label_format,
                &D2D_RECT_F {
                    left: pad,
                    top: y,
                    right: w - pad - self.sf(35),
                    bottom: y + self.sf(16),
                },
                &label_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );

            // Percentage (right)
            let pct = format!("{:.0}%", utilization);
            let pct_text = wide(&pct);
            let pct_format = d2d.get_text_format(11, false, 1, 1).clone();
            let pct_color = colorref_to_d2d(colors.progress_color(*utilization));
            let pct_brush = rt
                .CreateSolidColorBrush(&pct_color as *const _, None)
                .unwrap();
            rt.DrawText(
                &pct_text[..pct_text.len() - 1],
                &pct_format,
                &D2D_RECT_F {
                    left: w - pad - self.sf(35),
                    top: y,
                    right: w - pad,
                    bottom: y + self.sf(16),
                },
                &pct_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
            y += self.sf(16);

            // Progress bar (flat)
            let bar_h = self.sf(8);
            let bg_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.progress_bg) as *const _, None)
                .unwrap();
            rt.FillRectangle(
                &D2D_RECT_F {
                    left: pad,
                    top: y,
                    right: pad + content_w,
                    bottom: y + bar_h,
                },
                &bg_brush,
            );
            let fill_w = (content_w * *utilization as f32 / 100.0)
                .max(0.0)
                .min(content_w);
            if fill_w > 0.5 {
                let fill_color = colorref_to_d2d(colors.progress_color(*utilization));
                let fill_brush = rt
                    .CreateSolidColorBrush(&fill_color as *const _, None)
                    .unwrap();
                rt.FillRectangle(
                    &D2D_RECT_F {
                        left: pad,
                        top: y,
                        right: pad + fill_w,
                        bottom: y + bar_h,
                    },
                    &fill_brush,
                );
                // Accessibility overlay pattern
                if crate::APP_STATE
                    .as_ref()
                    .is_some_and(|s| s.config_mgr.config.accessibility_patterns)
                {
                    draw_accessibility_pattern(rt, pad, y, pad + fill_w, y + bar_h, *utilization);
                }
            }
            y += bar_h + self.sf(ITEM_GAP);
        }

        y
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_not_detected(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        mut y: f32,
        colors: &ThemeColors,
        i18n: &I18n,
        last_error: &Option<String>,
        install_rect: &mut RECT,
    ) -> f32 {
        let pad = self.sf(PADDING);

        let is_cred_error = last_error
            .as_ref()
            .is_some_and(|e| e.contains("credentials not found"));

        let (title, desc, btn_label) = if is_cred_error {
            (
                i18n.t("credentials_not_found"),
                i18n.t("run_claude_login_desc"),
                i18n.t("Open Claude.ai \u{2192}"),
            )
        } else {
            (
                i18n.t("Claude Code not detected"),
                i18n.t("install_claude_desc"),
                i18n.t("Install Claude Code \u{2192}"),
            )
        };

        // Warning title
        let warn_str = format!("\u{26A0} {}", title);
        let warn_text = wide(&warn_str);
        let warn_format = d2d.get_text_format(12, true, 0, 1).clone();
        let warn_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.yellow) as *const _, None)
            .unwrap();
        rt.DrawText(
            &warn_text[..warn_text.len() - 1],
            &warn_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad,
                bottom: y + self.sf(24),
            },
            &warn_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += self.sf(28);

        // Description (word wrap)
        let desc_text = wide(desc);
        let desc_format = d2d.get_text_format_wrap(11, false);
        let desc_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &desc_text[..desc_text.len() - 1],
            &desc_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad,
                bottom: y + self.sf(60),
            },
            &desc_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += self.sf(70);

        // Action button
        let btn_h = self.sf(28);
        let btn_rect = D2D_RECT_F {
            left: pad,
            top: y,
            right: w - pad,
            bottom: y + btn_h,
        };
        *install_rect = to_win32_rect(&btn_rect);

        let btn_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
            .unwrap();
        rt.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: btn_rect,
                radiusX: 4.0,
                radiusY: 4.0,
            },
            &btn_brush,
        );

        let btn_text = wide(btn_label);
        let btn_format = d2d.get_text_format(12, false, 0, 1).clone();
        let white = D2D1_COLOR_F {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let btn_text_brush = rt.CreateSolidColorBrush(&white as *const _, None).unwrap();
        let text_rect = D2D_RECT_F {
            left: pad + self.sf(8),
            top: y,
            right: w - pad - self.sf(8),
            bottom: y + btn_h,
        };
        rt.DrawText(
            &btn_text[..btn_text.len() - 1],
            &btn_format,
            &text_rect,
            &btn_text_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += btn_h + self.sf(8);

        y
    }

    unsafe fn draw_chatgpt_section(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        mut y: f32,
        colors: &ThemeColors,
        i18n: &I18n,
        link_rect: &mut RECT,
    ) -> f32 {
        let pad = self.sf(PADDING);

        // Section header
        let header_str = format!("\u{25CE} {}", i18n.t("CHATGPT / CODEX"));
        let header_text = wide(&header_str);
        let header_format = d2d.get_text_format(12, true, 0, 1).clone();
        let header_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &header_text[..header_text.len() - 1],
            &header_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad,
                bottom: y + self.sf(20),
            },
            &header_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += self.sf(24);

        // Info text (word wrap)
        let info = format!("\u{24D8} {}", i18n.t("openai_no_api"));
        let info_text = wide(&info);
        let info_format = d2d.get_text_format_wrap(11, false);
        let info_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &info_text[..info_text.len() - 1],
            &info_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad,
                bottom: y + self.sf(50),
            },
            &info_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += self.sf(55);

        // Link
        let link_str = format!("\u{1F4CA} {}", i18n.t("Open ChatGPT Usage \u{2192}"));
        let link_text = wide(&link_str);
        let link_format = d2d.get_text_format(12, false, 0, 1).clone();
        let link_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
            .unwrap();
        let lr = D2D_RECT_F {
            left: pad,
            top: y,
            right: w - pad,
            bottom: y + self.sf(22),
        };
        *link_rect = to_win32_rect(&lr);
        rt.DrawText(
            &link_text[..link_text.len() - 1],
            &link_format,
            &lr,
            &link_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += self.sf(28);

        y
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_chart(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        mut y: f32,
        data: &[f64],
        reset_lines: &[f64],
        colors: &ThemeColors,
        i18n: &I18n,
        hovered: &HoveredElement,
        chart_rect_out: &mut RECT,
        chart_bar_count_out: &mut usize,
    ) -> f32 {
        let pad = self.sf(PADDING);

        // Header
        let title = i18n.t("Usage History (24h)");
        let title_text = wide(title);
        let title_format = d2d.get_text_format(11, true, 0, 1).clone();
        let title_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &title_text[..title_text.len() - 1],
            &title_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - pad,
                bottom: y + self.sf(18),
            },
            &title_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += self.sf(22);

        let chart_h = self.sf(70);
        let chart_w = w - pad * 2.0;

        // Output chart area for hit-testing
        *chart_rect_out = RECT {
            left: pad as i32,
            top: y as i32,
            right: (pad + chart_w) as i32,
            bottom: (y + chart_h) as i32,
        };
        *chart_bar_count_out = data.len();

        // Chart background
        let bg_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.surface) as *const _, None)
            .unwrap();
        rt.FillRectangle(
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: pad + chart_w,
                bottom: y + chart_h,
            },
            &bg_brush,
        );

        // Grid lines at 25%, 50%, 75%
        let grid_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.separator) as *const _, None)
            .unwrap();
        for pct in [25, 50, 75] {
            let gy = y + chart_h - (chart_h * pct as f32 / 100.0);
            rt.DrawLine(
                D2D_POINT_2F { x: pad, y: gy },
                D2D_POINT_2F {
                    x: pad + chart_w,
                    y: gy,
                },
                &grid_brush,
                1.0,
                None,
            );
        }

        if !data.is_empty() {
            let bar_w = (chart_w / data.len() as f32).max(2.0);
            let gap = 1.0f32.max(bar_w / 6.0);
            for (i, &val) in data.iter().enumerate() {
                let bar_h_px = (val / 100.0) * chart_h as f64;
                if bar_h_px > 0.5 {
                    let bar_x = pad + i as f32 * bar_w;
                    let color = if val >= 80.0 {
                        colorref_to_d2d(colors.red)
                    } else if val >= 50.0 {
                        colorref_to_d2d(colors.yellow)
                    } else {
                        colorref_to_d2d(colors.green)
                    };
                    let bar_brush = rt.CreateSolidColorBrush(&color as *const _, None).unwrap();
                    rt.FillRectangle(
                        &D2D_RECT_F {
                            left: bar_x + gap,
                            top: y + chart_h - bar_h_px as f32,
                            right: bar_x + bar_w - gap,
                            bottom: y + chart_h,
                        },
                        &bar_brush,
                    );
                }
            }
        }

        // Reset lines (dashed vertical)
        if !reset_lines.is_empty() {
            let reset_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
                .unwrap();
            let chart_top = y;
            for &hours_ago in reset_lines {
                let rx = pad + chart_w * (1.0 - hours_ago as f32 / 24.0);
                if rx >= pad && rx <= pad + chart_w {
                    let dash = 3.0f32;
                    let mut dy = chart_top;
                    while dy < chart_top + chart_h {
                        let end = (dy + dash).min(chart_top + chart_h);
                        rt.DrawLine(
                            D2D_POINT_2F { x: rx, y: dy },
                            D2D_POINT_2F { x: rx, y: end },
                            &reset_brush,
                            1.0,
                            None,
                        );
                        dy += dash * 2.0;
                    }
                }
            }
        }

        // Hover tooltip
        if let HoveredElement::ChartBar(idx) = hovered {
            let idx = *idx;
            if idx < data.len() {
                let val = data[idx];
                let bar_w = (chart_w / data.len() as f32).max(2.0);
                let bar_cx = pad + idx as f32 * bar_w + bar_w / 2.0;
                let hours_ago = 24.0 * (1.0 - (idx as f64 + 0.5) / data.len() as f64);

                // Show actual clock time for this bar
                let bar_time =
                    chrono::Local::now() - chrono::Duration::seconds((hours_ago * 3600.0) as i64);
                let time_str = if is_system_24h() {
                    bar_time.format("%H:%M").to_string()
                } else {
                    bar_time.format("%I:%M %p").to_string()
                };
                let tip_text = format!("{:.0}% | {}", val, time_str);
                let tip_wide = wide(&tip_text);

                let tip_w = self.sf(100);
                let tip_h = self.sf(22);
                let tip_x = (bar_cx - tip_w / 2.0).clamp(pad, pad + chart_w - tip_w);
                let bar_h_px = (val / 100.0) * chart_h as f64;
                let tip_y = (y + chart_h - bar_h_px as f32 - tip_h - self.sf(4)).max(y - tip_h);

                // Tooltip background
                let tip_bg = rt
                    .CreateSolidColorBrush(&colorref_to_d2d(colors.surface) as *const _, None)
                    .unwrap();
                let tip_border_brush = rt
                    .CreateSolidColorBrush(&colorref_to_d2d(colors.border) as *const _, None)
                    .unwrap();
                let tip_rect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: tip_x,
                        top: tip_y,
                        right: tip_x + tip_w,
                        bottom: tip_y + tip_h,
                    },
                    radiusX: 4.0,
                    radiusY: 4.0,
                };
                rt.FillRoundedRectangle(&tip_rect, &tip_bg);
                rt.DrawRoundedRectangle(&tip_rect, &tip_border_brush, 1.0, None);

                // Tooltip text
                let tip_format = d2d.get_text_format(9, true, 2, 1).clone();
                let tip_text_brush = rt
                    .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
                    .unwrap();
                rt.DrawText(
                    &tip_wide[..tip_wide.len() - 1],
                    &tip_format,
                    &D2D_RECT_F {
                        left: tip_x,
                        top: tip_y,
                        right: tip_x + tip_w,
                        bottom: tip_y + tip_h,
                    },
                    &tip_text_brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                    DWRITE_MEASURING_MODE_NATURAL,
                );
            }
        }

        y += chart_h + self.sf(4);

        // X-axis labels
        let labels = ["24h ago", "18h ago", "12h ago", "6h ago", "now"];
        for (i, label) in labels.iter().enumerate() {
            let lx = pad + (i as f32 * chart_w / 4.0);
            let text = wide(label);
            let format = d2d.get_text_format(9, false, 0, 0).clone();
            let brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
                .unwrap();
            rt.DrawText(
                &text[..text.len() - 1],
                &format,
                &D2D_RECT_F {
                    left: lx - self.sf(14),
                    top: y,
                    right: lx + self.sf(30),
                    bottom: y + self.sf(14),
                },
                &brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
        y += self.sf(14);

        y
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn draw_footer(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        w: f32,
        y: f32,
        last_updated: &str,
        colors: &ThemeColors,
        i18n: &I18n,
        hovered: &HoveredElement,
        refresh_rect: &mut RECT,
        _status_link_rect: &mut RECT,
    ) {
        let h = self.sf(FOOTER_H);
        let pad = self.sf(PADDING);

        // Footer background
        let surface_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.surface) as *const _, None)
            .unwrap();
        rt.FillRectangle(
            &D2D_RECT_F {
                left: 0.0,
                top: y,
                right: w,
                bottom: y + h,
            },
            &surface_brush,
        );

        // Last updated text
        let updated_text = format!("{} {}", i18n.t("Last updated:"), last_updated);
        let updated_wide = wide(&updated_text);
        let updated_format = d2d.get_text_format(10, false, 0, 1).clone();
        let text_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &updated_wide[..updated_wide.len() - 1],
            &updated_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w - self.sf(90),
                bottom: y + h,
            },
            &text_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        // Refresh button
        let btn_w = self.sf(78);
        let btn_h = self.sf(26);
        let btn_rect = D2D_RECT_F {
            left: w - btn_w - pad,
            top: y + (h - btn_h) / 2.0,
            right: w - pad,
            bottom: y + (h - btn_h) / 2.0 + btn_h,
        };
        *refresh_rect = to_win32_rect(&btn_rect);

        let is_hovered = matches!(hovered, HoveredElement::RefreshButton);

        if is_hovered {
            // Filled accent button when hovered
            let fill_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
                .unwrap();
            rt.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: btn_rect,
                    radiusX: self.sf(6),
                    radiusY: self.sf(6),
                },
                &fill_brush,
            );
            // White text on hover
            let white = D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            };
            let text_brush = rt.CreateSolidColorBrush(&white as *const _, None).unwrap();
            let refresh_text = wide(i18n.t("Refresh"));
            let text_format = d2d.get_text_format(10, false, 2, 1).clone();
            rt.DrawText(
                &refresh_text[..refresh_text.len() - 1],
                &text_format,
                &btn_rect,
                &text_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        } else {
            // Outlined button (normal state)
            let border_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
                .unwrap();
            let bg_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.surface) as *const _, None)
                .unwrap();
            rt.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: btn_rect,
                    radiusX: self.sf(6),
                    radiusY: self.sf(6),
                },
                &bg_brush,
            );
            rt.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: btn_rect,
                    radiusX: self.sf(6),
                    radiusY: self.sf(6),
                },
                &border_brush,
                1.0,
                None,
            );
            // Accent text
            let accent_brush = rt
                .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
                .unwrap();
            let refresh_text = wide(i18n.t("Refresh"));
            let text_format = d2d.get_text_format(10, false, 2, 1).clone();
            rt.DrawText(
                &refresh_text[..refresh_text.len() - 1],
                &text_format,
                &btn_rect,
                &accent_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
    }

    unsafe fn draw_text_centered(
        &self,
        rt: &ID2D1HwndRenderTarget,
        d2d: &mut D2DResources,
        text: &str,
        rect: D2D_RECT_F,
        color: windows::Win32::Foundation::COLORREF,
        size: i32,
        bold: bool,
    ) {
        let format = d2d.get_text_format(size, bold, 2, 1).clone();
        let brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(color) as *const _, None)
            .unwrap();
        let text_wide = wide(text);
        rt.DrawText(
            &text_wide[..text_wide.len() - 1],
            &format,
            &rect,
            &brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
    }

    unsafe fn draw_border(&self, rt: &ID2D1HwndRenderTarget, w: f32, h: f32, colors: &ThemeColors) {
        let brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.border) as *const _, None)
            .unwrap();
        rt.DrawRectangle(
            &D2D_RECT_F {
                left: 0.5,
                top: 0.5,
                right: w - 0.5,
                bottom: h - 0.5,
            },
            &brush,
            1.0,
            None,
        );
    }
}

/// Settings panel rendering (D2D)
#[allow(clippy::too_many_arguments)]
pub unsafe fn draw_settings_panel(
    d2d: &mut D2DResources,
    rect: &RECT,
    colors: &ThemeColors,
    i18n: &I18n,
    config: &crate::config::Config,
    back_rect: &mut RECT,
    close_rect: &mut RECT,
    setting_rects: &mut [RECT; 9],
    hovered: &HoveredElement,
) {
    let Some(rt) = d2d.render_target.clone() else {
        return;
    };
    let w = (rect.right - rect.left) as f32;
    let pad = 16.0f32;
    let header_h = 40.0f32;
    let row_h = 38.0f32;

    // Header background
    let surf_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.surface) as *const _, None)
        .unwrap();
    rt.FillRectangle(
        &D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: w,
            bottom: header_h,
        },
        &surf_brush,
    );

    // Header separator
    let sep_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.separator) as *const _, None)
        .unwrap();
    rt.DrawLine(
        D2D_POINT_2F {
            x: 0.0,
            y: header_h,
        },
        D2D_POINT_2F { x: w, y: header_h },
        &sep_brush,
        1.0,
        None,
    );

    // Back button: "← Back"
    let back_r = D2D_RECT_F {
        left: pad,
        top: 0.0,
        right: w / 2.0,
        bottom: header_h,
    };
    *back_rect = to_win32_rect(&back_r);

    // Hover highlight for back button
    if matches!(hovered, HoveredElement::BackButton) {
        let hover_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.hover) as *const _, None)
            .unwrap();
        rt.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: pad - 4.0,
                    top: 6.0,
                    right: pad + 80.0,
                    bottom: header_h - 6.0,
                },
                radiusX: 4.0,
                radiusY: 4.0,
            },
            &hover_brush,
        );
    }

    let back_text = wide(i18n.t("Back"));
    let back_format = d2d.get_text_format(13, true, 0, 1).clone();
    let accent_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
        .unwrap();
    rt.DrawText(
        &back_text[..back_text.len() - 1],
        &back_format,
        &back_r,
        &accent_brush,
        D2D1_DRAW_TEXT_OPTIONS_NONE,
        DWRITE_MEASURING_MODE_NATURAL,
    );

    // Title centered
    let title_text = wide(i18n.t("Settings"));
    let title_format = d2d.get_text_format(13, true, 2, 1).clone();
    let title_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
        .unwrap();
    rt.DrawText(
        &title_text[..title_text.len() - 1],
        &title_format,
        &D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: w,
            bottom: header_h,
        },
        &title_brush,
        D2D1_DRAW_TEXT_OPTIONS_NONE,
        DWRITE_MEASURING_MODE_NATURAL,
    );

    // Close button ×
    let close_r = D2D_RECT_F {
        left: w - 36.0,
        top: 0.0,
        right: w - 4.0,
        bottom: header_h,
    };
    *close_rect = to_win32_rect(&close_r);

    if matches!(hovered, HoveredElement::CloseButton) {
        let hover_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.hover) as *const _, None)
            .unwrap();
        rt.FillRoundedRectangle(
            &D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: w - 36.0,
                    top: 6.0,
                    right: w - 4.0,
                    bottom: header_h - 6.0,
                },
                radiusX: 4.0,
                radiusY: 4.0,
            },
            &hover_brush,
        );
    }

    let close_text = wide("\u{00D7}");
    let close_format = d2d.get_text_format(13, false, 2, 1).clone();
    let close_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
        .unwrap();
    rt.DrawText(
        &close_text[..close_text.len() - 1],
        &close_format,
        &close_r,
        &close_brush,
        D2D1_DRAW_TEXT_OPTIONS_NONE,
        DWRITE_MEASURING_MODE_NATURAL,
    );

    let mut y = header_h + 8.0;

    // Settings rows
    let check_on = "\u{2611}"; // ☑
    let check_off = "\u{2610}"; // ☐
                                // Build language display: "Auto (English)" or "English", "Українська", etc.
    let lang_display = if config.language == "auto" {
        let detected = crate::i18n::Locale::detect_from_windows();
        format!("{} ({})", i18n.t("Auto"), detected.display_name())
    } else {
        crate::i18n::Locale::from_str(&config.language)
            .map(|l| l.display_name().to_string())
            .unwrap_or_else(|| config.language.to_uppercase())
    };

    let rows: Vec<(String, String)> = vec![
        (
            i18n.t("Theme").to_string(),
            i18n.t(&capitalize(&config.theme)).to_string(),
        ),
        (i18n.t("Language").to_string(), lang_display),
        (
            i18n.t("Compact mode").to_string(),
            (if config.compact_mode {
                check_on
            } else {
                check_off
            })
            .to_string(),
        ),
        (
            i18n.t("Show ChatGPT section").to_string(),
            (if config.show_chatgpt_section {
                check_on
            } else {
                check_off
            })
            .to_string(),
        ),
        (
            i18n.t("Start with Windows").to_string(),
            (if config.autostart {
                check_on
            } else {
                check_off
            })
            .to_string(),
        ),
        (
            i18n.t("Show widget").to_string(),
            (if config.show_widget {
                check_on
            } else {
                check_off
            })
            .to_string(),
        ),
        (
            i18n.t("Check for updates").to_string(),
            (if config.check_updates {
                check_on
            } else {
                check_off
            })
            .to_string(),
        ),
        (
            i18n.t("Accessibility patterns").to_string(),
            (if config.accessibility_patterns {
                check_on
            } else {
                check_off
            })
            .to_string(),
        ),
        (
            i18n.t("Icon style").to_string(),
            i18n.t(&capitalize(&config.tray_icon_style)).to_string(),
        ),
    ];

    for (i, (label, value)) in rows.iter().enumerate() {
        let is_hovered = matches!(hovered, HoveredElement::SettingRow(idx) if *idx == i);

        // Row background
        let row_color = if is_hovered {
            colorref_to_d2d(colors.hover)
        } else if i % 2 == 1 {
            colorref_to_d2d(colors.surface)
        } else {
            colorref_to_d2d(colors.background)
        };
        let row_brush = rt
            .CreateSolidColorBrush(&row_color as *const _, None)
            .unwrap();
        rt.FillRectangle(
            &D2D_RECT_F {
                left: 0.0,
                top: y,
                right: w,
                bottom: y + row_h,
            },
            &row_brush,
        );

        setting_rects[i] = RECT {
            left: 0,
            top: y as i32,
            right: w as i32,
            bottom: (y + row_h) as i32,
        };

        // Label (left)
        let label_text = wide(label);
        let label_format = d2d.get_text_format(12, false, 0, 1).clone();
        let label_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &label_text[..label_text.len() - 1],
            &label_format,
            &D2D_RECT_F {
                left: pad,
                top: y,
                right: w / 2.0 + 40.0,
                bottom: y + row_h,
            },
            &label_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        // Value (right)
        let val_text = wide(value);
        let val_format = d2d.get_text_format(12, false, 1, 1).clone();
        let val_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
            .unwrap();
        rt.DrawText(
            &val_text[..val_text.len() - 1],
            &val_format,
            &D2D_RECT_F {
                left: w / 2.0 + 40.0,
                top: y,
                right: w - pad,
                bottom: y + row_h,
            },
            &val_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );

        y += row_h;

        // Separator between rows
        if i < rows.len() - 1 {
            rt.DrawLine(
                D2D_POINT_2F { x: pad, y },
                D2D_POINT_2F { x: w - pad, y },
                &sep_brush,
                1.0,
                None,
            );
        }
    }

    // Icon legend section
    y += 8.0;
    rt.DrawLine(
        D2D_POINT_2F { x: pad, y },
        D2D_POINT_2F { x: w - pad, y },
        &sep_brush,
        1.0,
        None,
    );
    y += 8.0;

    // Legend title
    let legend_title = wide(i18n.t("Tray icon colors:"));
    let legend_format = d2d.get_text_format(11, false, 0, 0).clone();
    let legend_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
        .unwrap();
    rt.DrawText(
        &legend_title[..legend_title.len() - 1],
        &legend_format,
        &D2D_RECT_F {
            left: pad,
            top: y,
            right: w - pad,
            bottom: y + 18.0,
        },
        &legend_brush,
        D2D1_DRAW_TEXT_OPTIONS_NONE,
        DWRITE_MEASURING_MODE_NATURAL,
    );
    y += 20.0;

    // Icon items: colored circle + text
    let icon_items: [(windows::Win32::Foundation::COLORREF, &str); 4] = [
        (colors.green, i18n.t("< 50% usage")),
        (colors.yellow, i18n.t("50-79% usage")),
        (colors.red, i18n.t(">= 80% usage")),
        (colors.separator, i18n.t("No data")),
    ];

    for (color, label) in &icon_items {
        let circle_size = 10.0f32;
        let circle_y = y + (16.0 - circle_size) / 2.0;
        let cx = pad + 2.0 + circle_size / 2.0;
        let cy = circle_y + circle_size / 2.0;

        let circle_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(*color) as *const _, None)
            .unwrap();
        rt.FillEllipse(
            &D2D1_ELLIPSE {
                point: D2D_POINT_2F { x: cx, y: cy },
                radiusX: circle_size / 2.0,
                radiusY: circle_size / 2.0,
            },
            &circle_brush,
        );

        let lbl_text = wide(label);
        let lbl_format = d2d.get_text_format(11, false, 0, 0).clone();
        let lbl_brush = rt
            .CreateSolidColorBrush(&colorref_to_d2d(colors.text_primary) as *const _, None)
            .unwrap();
        rt.DrawText(
            &lbl_text[..lbl_text.len() - 1],
            &lbl_format,
            &D2D_RECT_F {
                left: pad + 18.0,
                top: y,
                right: w - pad,
                bottom: y + 16.0,
            },
            &lbl_brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
            DWRITE_MEASURING_MODE_NATURAL,
        );
        y += 18.0;
    }

    // Footer
    let footer_y = (rect.bottom - 44) as f32;
    rt.DrawLine(
        D2D_POINT_2F {
            x: 0.0,
            y: footer_y,
        },
        D2D_POINT_2F { x: w, y: footer_y },
        &sep_brush,
        1.0,
        None,
    );

    let fy = footer_y + 6.0;
    let footer_text1 = wide(&format!(
        "ClaudeMeter v{} by klivak",
        env!("CARGO_PKG_VERSION")
    ));
    let footer_format = d2d.get_text_format(10, false, 0, 0).clone();
    let footer_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.text_secondary) as *const _, None)
        .unwrap();
    rt.DrawText(
        &footer_text1[..footer_text1.len() - 1],
        &footer_format,
        &D2D_RECT_F {
            left: pad,
            top: fy,
            right: w - pad,
            bottom: fy + 16.0,
        },
        &footer_brush,
        D2D1_DRAW_TEXT_OPTIONS_NONE,
        DWRITE_MEASURING_MODE_NATURAL,
    );

    let footer_text2 = wide("github.com/klivak/claudemeter");
    let footer_link_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.accent) as *const _, None)
        .unwrap();
    rt.DrawText(
        &footer_text2[..footer_text2.len() - 1],
        &footer_format,
        &D2D_RECT_F {
            left: pad,
            top: fy + 16.0,
            right: w - pad,
            bottom: fy + 32.0,
        },
        &footer_link_brush,
        D2D1_DRAW_TEXT_OPTIONS_NONE,
        DWRITE_MEASURING_MODE_NATURAL,
    );

    // 1px border
    let border_brush = rt
        .CreateSolidColorBrush(&colorref_to_d2d(colors.border) as *const _, None)
        .unwrap();
    let h = (rect.bottom - rect.top) as f32;
    rt.DrawRectangle(
        &D2D_RECT_F {
            left: 0.5,
            top: 0.5,
            right: w - 0.5,
            bottom: h - 0.5,
        },
        &border_brush,
        1.0,
        None,
    );
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn to_win32_rect(r: &D2D_RECT_F) -> RECT {
    RECT {
        left: r.left as i32,
        top: r.top as i32,
        right: r.right as i32,
        bottom: r.bottom as i32,
    }
}
