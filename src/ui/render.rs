/// Rendering helpers for the popup window using GDI.
///
/// The rendering is done entirely with Win32 GDI calls:
/// - FillRect for backgrounds and progress bars
/// - DrawTextW for text rendering
/// - MoveToEx + LineTo for separators
///
/// All coordinates are in logical pixels (DPI-scaled by Windows automatically
/// when PerMonitorV2 DPI awareness is set and we use the correct DPI scaling).

use crate::i18n::{format_duration, seconds_until, I18n};
use crate::providers::claude::{format_metric_name, UsageResponse};
use crate::ui::colors::{ColorRef, ThemeColors};
use windows::Win32::Foundation::{COLORREF, HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DeleteObject, DrawTextW, FillRect,
    GetDC, GetDeviceCaps, LineTo, MoveToEx, ReleaseDC, SelectObject, SetBkMode,
    SetTextColor, HDC, LOGPIXELSX, PS_SOLID, TRANSPARENT, DT_LEFT,
    DT_SINGLELINE, DT_VCENTER, DT_END_ELLIPSIS,
};

pub const POPUP_WIDTH: i32 = 360;
pub const HEADER_HEIGHT: i32 = 36;
pub const PADDING: i32 = 16;
pub const METRIC_LABEL_H: i32 = 20;
pub const PROGRESS_H: i32 = 10;
pub const RESET_LABEL_H: i32 = 18;
pub const SECTION_GAP: i32 = 14;
pub const ITEM_GAP: i32 = 8;
pub const SEPARATOR_H: i32 = 1;
pub const FOOTER_H: i32 = 36;

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

    /// Calculate the total height needed for the popup in full mode.
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
                HEADER_HEIGHT + PADDING + metric_count * (PROGRESS_H + ITEM_GAP) + PADDING + FOOTER_H,
            );
        }

        let mut h = HEADER_HEIGHT + PADDING;

        match usage {
            None => {
                h += 120; // "not detected" message
            }
            Some(u) => {
                let metric_count = u.all_metrics().len() as i32;
                // Each metric: label + progress bar + reset label + gap
                h += metric_count * (METRIC_LABEL_H + PROGRESS_H + RESET_LABEL_H + SECTION_GAP);
                h += SECTION_GAP; // after section
            }
        }

        if show_chatgpt {
            h += 10 + 20 + 40 + 28 + 10; // section header + info text + link
        }

        h += SEPARATOR_H + PADDING;
        // History chart
        h += 20 + 80 + PADDING;

        h += SEPARATOR_H + FOOTER_H;

        self.scale(h)
    }

    pub fn draw(
        &self,
        hdc: HDC,
        rect: &RECT,
        usage: &Option<UsageResponse>,
        last_updated: &str,
        show_chatgpt: bool,
        compact: bool,
        colors: &ThemeColors,
        i18n: &I18n,
        chart_data: &[f64],
        // Button hit areas (output)
        settings_rect: &mut RECT,
        close_rect: &mut RECT,
        refresh_rect: &mut RECT,
        install_rect: &mut RECT,
        chatgpt_link_rect: &mut RECT,
    ) {
        let w = rect.right - rect.left;

        unsafe {
            // Background
            let bg_brush = CreateSolidBrush(colors.background);
            let _ = FillRect(hdc, rect, bg_brush);
            let _ = DeleteObject(bg_brush);

            let mut y = 0i32;

            // Header
            y = self.draw_header(hdc, w, y, colors, i18n, settings_rect, close_rect);

            // Separator
            y = self.draw_separator(hdc, w, y, colors);

            y += self.scale(PADDING);

            if compact {
                y = self.draw_compact_metrics(hdc, w, y, usage, colors, i18n);
            } else {
                match usage {
                    None => {
                        y = self.draw_not_detected(hdc, w, y, colors, i18n, install_rect);
                    }
                    Some(u) => {
                        y = self.draw_claude_section(hdc, w, y, u, colors, i18n);
                    }
                }

                if show_chatgpt {
                    y = self.draw_separator(hdc, w, y, colors);
                    y += self.scale(8);
                    y = self.draw_chatgpt_section(hdc, w, y, colors, i18n, chatgpt_link_rect);
                }

                // History chart
                y = self.draw_separator(hdc, w, y, colors);
                y += self.scale(PADDING);
                y = self.draw_chart(hdc, w, y, chart_data, colors, i18n);
                y += self.scale(PADDING);
            }

            // Footer
            self.draw_separator(hdc, w, y, colors);
            self.draw_footer(hdc, w, y + self.scale(SEPARATOR_H), last_updated, colors, i18n, refresh_rect);
        }
    }

    unsafe fn draw_header(
        &self,
        hdc: HDC,
        w: i32,
        y: i32,
        colors: &ThemeColors,
        i18n: &I18n,
        settings_rect: &mut RECT,
        close_rect: &mut RECT,
    ) -> i32 {
        let h = self.scale(HEADER_HEIGHT);
        let surface_brush = CreateSolidBrush(colors.surface);
        let header_rect = RECT { left: 0, top: y, right: w, bottom: y + h };
        let _ = FillRect(hdc, &header_rect, surface_brush);
        let _ = DeleteObject(surface_brush);

        // Title
        let font = self.create_font(14, true);
        let old_font = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_primary);
        let mut title = wide(i18n.t("ClaudeMeter"));
        let mut text_rect = RECT {
            left: self.scale(PADDING),
            top: y,
            right: w - self.scale(80),
            bottom: y + h,
        };
        DrawTextW(hdc, &mut title, &mut text_rect, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old_font);
        let _ = DeleteObject(font);

        // ⚙ button
        let btn_size = self.scale(28);
        let btn_y = y + (h - btn_size) / 2;
        *settings_rect = RECT {
            left: w - self.scale(64),
            top: btn_y,
            right: w - self.scale(36),
            bottom: btn_y + btn_size,
        };
        self.draw_text_centered(hdc, "\u{2699}", *settings_rect, colors.text_secondary, 14, false);

        // × button
        *close_rect = RECT {
            left: w - self.scale(32),
            top: btn_y,
            right: w - self.scale(4),
            bottom: btn_y + btn_size,
        };
        self.draw_text_centered(hdc, "\u{00D7}", *close_rect, colors.text_secondary, 14, false);

        y + h
    }

    unsafe fn draw_separator(&self, hdc: HDC, w: i32, y: i32, colors: &ThemeColors) -> i32 {
        let pen = CreatePen(PS_SOLID, 1, colors.separator);
        let old_pen = SelectObject(hdc, pen);
        let _ = MoveToEx(hdc, 0, y, None);
        let _ = LineTo(hdc, w, y);
        let _ = SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
        y + self.scale(SEPARATOR_H)
    }

    unsafe fn draw_claude_section(
        &self,
        hdc: HDC,
        w: i32,
        mut y: i32,
        usage: &UsageResponse,
        colors: &ThemeColors,
        i18n: &I18n,
    ) -> i32 {
        // Section header: "☁ CLAUDE · Pro plan"
        let plan = i18n.t(usage.detected_plan());
        let header_str = format!("\u{2601} {} \u{00B7} {} {}", i18n.t("CLAUDE"), i18n.t("Plan"), plan);
        let font = self.create_font(12, true);
        let old_font = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_secondary);
        let mut r = RECT {
            left: self.scale(PADDING),
            top: y,
            right: w - self.scale(PADDING),
            bottom: y + self.scale(20),
        };
        let mut header_wide = wide(&header_str);
        DrawTextW(hdc, &mut header_wide, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old_font);
        let _ = DeleteObject(font);
        y += self.scale(24);

        for (key, metric) in usage.all_metrics() {
            y = self.draw_metric(hdc, w, y, &key, metric.utilization, metric.resets_at.as_deref(), colors, i18n);
            y += self.scale(SECTION_GAP);
        }

        y
    }

    unsafe fn draw_metric(
        &self,
        hdc: HDC,
        w: i32,
        mut y: i32,
        key: &str,
        utilization: f64,
        resets_at: Option<&str>,
        colors: &ThemeColors,
        i18n: &I18n,
    ) -> i32 {
        let pad = self.scale(PADDING);
        let content_w = w - pad * 2;

        // Label + percentage on same line
        let metric_name_str = format_metric_name(key);
        let display_name = i18n.t(&metric_name_str);
        let pct_str = format!("{:.0}%", utilization);

        let font_label = self.create_font(12, false);
        let old_font = SelectObject(hdc, font_label);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_primary);

        let mut label_rect = RECT {
            left: pad,
            top: y,
            right: w - pad - self.scale(40),
            bottom: y + self.scale(METRIC_LABEL_H),
        };
        let mut label_wide = wide(display_name);
        DrawTextW(hdc, &mut label_wide, &mut label_rect, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS);

        // Percentage right-aligned
        SetTextColor(hdc, colors.progress_color(utilization));
        let mut pct_rect = RECT {
            left: w - pad - self.scale(40),
            top: y,
            right: w - pad,
            bottom: y + self.scale(METRIC_LABEL_H),
        };
        let mut pct_wide = wide(&pct_str);
        DrawTextW(hdc, &mut pct_wide, &mut pct_rect, windows::Win32::Graphics::Gdi::DT_RIGHT | DT_SINGLELINE | DT_VCENTER);

        let _ = SelectObject(hdc, old_font);
        let _ = DeleteObject(font_label);

        y += self.scale(METRIC_LABEL_H + 4);

        // Progress bar background
        let bar_rect = RECT {
            left: pad,
            top: y,
            right: pad + content_w,
            bottom: y + self.scale(PROGRESS_H),
        };
        let bg_brush = CreateSolidBrush(colors.progress_bg);
        let _ = FillRect(hdc, &bar_rect, bg_brush);
        let _ = DeleteObject(bg_brush);

        // Progress bar fill
        let fill_w = ((content_w as f64 * utilization / 100.0) as i32).max(0).min(content_w);
        if fill_w > 0 {
            let fill_rect = RECT {
                left: pad,
                top: y,
                right: pad + fill_w,
                bottom: y + self.scale(PROGRESS_H),
            };
            let fill_brush = CreateSolidBrush(colors.progress_color(utilization));
            let _ = FillRect(hdc, &fill_rect, fill_brush);
            let _ = DeleteObject(fill_brush);
        }

        y += self.scale(PROGRESS_H + 4);

        // Reset time
        if let Some(reset_str) = resets_at {
            let reset_text = if let Some(secs) = seconds_until(reset_str) {
                if secs > 0 {
                    format!("{} {}", i18n.t("resets in"), format_duration(secs))
                } else {
                    "resetting soon".to_string()
                }
            } else {
                String::new()
            };

            if !reset_text.is_empty() {
                let font_small = self.create_font(11, false);
                let old_font2 = SelectObject(hdc, font_small);
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, colors.text_secondary);
                let mut reset_rect = RECT {
                    left: pad,
                    top: y,
                    right: w - pad,
                    bottom: y + self.scale(RESET_LABEL_H),
                };
                let mut reset_wide = wide(&reset_text);
                DrawTextW(hdc, &mut reset_wide, &mut reset_rect, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
                let _ = SelectObject(hdc, old_font2);
                let _ = DeleteObject(font_small);
                y += self.scale(RESET_LABEL_H);
            }
        }

        y
    }

    unsafe fn draw_compact_metrics(
        &self,
        hdc: HDC,
        w: i32,
        mut y: i32,
        usage: &Option<UsageResponse>,
        colors: &ThemeColors,
        i18n: &I18n,
    ) -> i32 {
        let pad = self.scale(PADDING);
        let content_w = w - pad * 2;

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
            let font = self.create_font(11, false);
            let old_font = SelectObject(hdc, font);
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, colors.text_primary);
            let mut label_rect = RECT {
                left: pad,
                top: y,
                right: w - pad - self.scale(35),
                bottom: y + self.scale(16),
            };
            let mut label_wide = wide(name);
            DrawTextW(hdc, &mut label_wide, &mut label_rect, DT_LEFT | DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS);

            SetTextColor(hdc, colors.progress_color(*utilization));
            let pct = format!("{:.0}%", utilization);
            let mut pct_rect = RECT {
                left: w - pad - self.scale(35),
                top: y,
                right: w - pad,
                bottom: y + self.scale(16),
            };
            let mut pct_wide = wide(&pct);
            DrawTextW(hdc, &mut pct_wide, &mut pct_rect, windows::Win32::Graphics::Gdi::DT_RIGHT | DT_SINGLELINE | DT_VCENTER);
            let _ = SelectObject(hdc, old_font);
            let _ = DeleteObject(font);
            y += self.scale(16);

            // Progress bar
            let bar_rect = RECT { left: pad, top: y, right: pad + content_w, bottom: y + self.scale(8) };
            let bg = CreateSolidBrush(colors.progress_bg);
            let _ = FillRect(hdc, &bar_rect, bg);
            let _ = DeleteObject(bg);
            let fill_w = ((content_w as f64 * utilization / 100.0) as i32).max(0).min(content_w);
            if fill_w > 0 {
                let fill_rect = RECT { left: pad, top: y, right: pad + fill_w, bottom: y + self.scale(8) };
                let fill = CreateSolidBrush(colors.progress_color(*utilization));
                let _ = FillRect(hdc, &fill_rect, fill);
                let _ = DeleteObject(fill);
            }
            y += self.scale(8 + ITEM_GAP);
        }

        y
    }

    unsafe fn draw_not_detected(
        &self,
        hdc: HDC,
        w: i32,
        mut y: i32,
        colors: &ThemeColors,
        i18n: &I18n,
        install_rect: &mut RECT,
    ) -> i32 {
        let pad = self.scale(PADDING);

        let font = self.create_font(12, true);
        let old_font = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.yellow);
        let mut r = RECT { left: pad, top: y, right: w - pad, bottom: y + self.scale(24) };
        let mut warn_wide = wide(&format!("\u{26A0} {}", i18n.t("Claude Code not detected")));
        DrawTextW(hdc, &mut warn_wide, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old_font);
        let _ = DeleteObject(font);
        y += self.scale(28);

        let font2 = self.create_font(11, false);
        let old2 = SelectObject(hdc, font2);
        SetTextColor(hdc, colors.text_secondary);
        let desc = i18n.t("install_claude_desc");
        let mut r2 = RECT { left: pad, top: y, right: w - pad, bottom: y + self.scale(60) };
        let mut desc_wide = wide(desc);
        DrawTextW(hdc, &mut desc_wide, &mut r2, DT_LEFT | windows::Win32::Graphics::Gdi::DT_WORDBREAK);
        let _ = SelectObject(hdc, old2);
        let _ = DeleteObject(font2);
        y += self.scale(70);

        // Install link button
        let btn_h = self.scale(28);
        *install_rect = RECT { left: pad, top: y, right: w - pad, bottom: y + btn_h };
        let btn_brush = CreateSolidBrush(colors.accent);
        let _ = FillRect(hdc, install_rect, btn_brush);
        let _ = DeleteObject(btn_brush);
        let font3 = self.create_font(12, false);
        let old3 = SelectObject(hdc, font3);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, COLORREF(0x00FFFFFF)); // white
        let mut btn_text = *install_rect;
        let mut btn_wide = wide(i18n.t("Install Claude Code \u{2192}"));
        DrawTextW(hdc, &mut btn_wide, &mut btn_text, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old3);
        let _ = DeleteObject(font3);
        y += btn_h + self.scale(8);

        y
    }

    unsafe fn draw_chatgpt_section(
        &self,
        hdc: HDC,
        w: i32,
        mut y: i32,
        colors: &ThemeColors,
        i18n: &I18n,
        link_rect: &mut RECT,
    ) -> i32 {
        let pad = self.scale(PADDING);

        // Section header
        let font = self.create_font(12, true);
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_secondary);
        let header_str = format!("\u{25CE} {}", i18n.t("CHATGPT / CODEX"));
        let mut r = RECT { left: pad, top: y, right: w - pad, bottom: y + self.scale(20) };
        let mut header_wide = wide(&header_str);
        DrawTextW(hdc, &mut header_wide, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font);
        y += self.scale(24);

        // Info text
        let font2 = self.create_font(11, false);
        let old2 = SelectObject(hdc, font2);
        SetTextColor(hdc, colors.text_secondary);
        let info = format!("\u{24D8} {}", i18n.t("openai_no_api"));
        let mut r2 = RECT { left: pad, top: y, right: w - pad, bottom: y + self.scale(50) };
        let mut info_wide = wide(&info);
        DrawTextW(hdc, &mut info_wide, &mut r2, DT_LEFT | windows::Win32::Graphics::Gdi::DT_WORDBREAK);
        let _ = SelectObject(hdc, old2);
        let _ = DeleteObject(font2);
        y += self.scale(55);

        // Link
        let font3 = self.create_font(12, false);
        let old3 = SelectObject(hdc, font3);
        SetTextColor(hdc, colors.accent);
        let link_text = format!("\u{1F4CA} {}", i18n.t("Open ChatGPT Usage \u{2192}"));
        *link_rect = RECT { left: pad, top: y, right: w - pad, bottom: y + self.scale(22) };
        let mut lr = *link_rect;
        let mut link_wide = wide(&link_text);
        DrawTextW(hdc, &mut link_wide, &mut lr, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old3);
        let _ = DeleteObject(font3);
        y += self.scale(28);

        y
    }

    unsafe fn draw_chart(
        &self,
        hdc: HDC,
        w: i32,
        mut y: i32,
        data: &[f64],
        colors: &ThemeColors,
        i18n: &I18n,
    ) -> i32 {
        let pad = self.scale(PADDING);

        // Header
        let font = self.create_font(11, true);
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_secondary);
        let title = format!("\u{1F4C8} {}", i18n.t("Usage History (24h)"));
        let mut r = RECT { left: pad, top: y, right: w - pad, bottom: y + self.scale(18) };
        let mut title_wide = wide(&title);
        DrawTextW(hdc, &mut title_wide, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font);
        y += self.scale(20);

        let chart_h = self.scale(60);
        let chart_w = w - pad * 2;
        let chart_rect = RECT { left: pad, top: y, right: pad + chart_w, bottom: y + chart_h };

        // Chart background
        let bg = CreateSolidBrush(colors.surface);
        let _ = FillRect(hdc, &chart_rect, bg);
        let _ = DeleteObject(bg);

        if !data.is_empty() {
            let bar_w = (chart_w / data.len() as i32).max(2);
            for (i, &val) in data.iter().enumerate() {
                let bar_h = ((val / 100.0) * chart_h as f64) as i32;
                if bar_h > 0 {
                    let bar_x = pad + i as i32 * bar_w;
                    let bar_rect = RECT {
                        left: bar_x,
                        top: y + chart_h - bar_h,
                        right: bar_x + bar_w - 1,
                        bottom: y + chart_h,
                    };
                    let color = if val >= 80.0 { colors.red } else if val >= 50.0 { colors.yellow } else { colors.green };
                    let bar_brush = CreateSolidBrush(color);
                    let _ = FillRect(hdc, &bar_rect, bar_brush);
                    let _ = DeleteObject(bar_brush);
                }
            }
        }

        y += chart_h + self.scale(4);

        // X-axis labels
        let font2 = self.create_font(9, false);
        let old2 = SelectObject(hdc, font2);
        SetTextColor(hdc, colors.text_secondary);
        let labels = ["24h", "18h", "12h", "6h", "now"];
        for (i, label) in labels.iter().enumerate() {
            let lx = pad + (i as i32 * chart_w / 4);
            let mut lr = RECT { left: lx - self.scale(10), top: y, right: lx + self.scale(20), bottom: y + self.scale(14) };
            let mut label_wide = wide(label);
            DrawTextW(hdc, &mut label_wide, &mut lr, DT_LEFT | DT_SINGLELINE);
        }
        let _ = SelectObject(hdc, old2);
        let _ = DeleteObject(font2);
        y += self.scale(14);

        y
    }

    unsafe fn draw_footer(
        &self,
        hdc: HDC,
        w: i32,
        y: i32,
        last_updated: &str,
        colors: &ThemeColors,
        i18n: &I18n,
        refresh_rect: &mut RECT,
    ) {
        let h = self.scale(FOOTER_H);
        let pad = self.scale(PADDING);

        let surface_brush = CreateSolidBrush(colors.surface);
        let footer_rect = RECT { left: 0, top: y, right: w, bottom: y + h };
        let _ = FillRect(hdc, &footer_rect, surface_brush);
        let _ = DeleteObject(surface_brush);

        let font = self.create_font(10, false);
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_secondary);
        let updated_text = format!("{} {}", i18n.t("Last updated:"), last_updated);
        let mut r = RECT { left: pad, top: y, right: w - self.scale(80), bottom: y + h };
        let mut updated_wide = wide(&updated_text);
        DrawTextW(hdc, &mut updated_wide, &mut r, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font);

        // Refresh button
        let btn_w = self.scale(70);
        *refresh_rect = RECT {
            left: w - btn_w - pad,
            top: y + (h - self.scale(22)) / 2,
            right: w - pad,
            bottom: y + (h - self.scale(22)) / 2 + self.scale(22),
        };
        let btn_brush = CreateSolidBrush(colors.surface);
        let _ = FillRect(hdc, refresh_rect, btn_brush);
        let _ = DeleteObject(btn_brush);
        let font2 = self.create_font(10, false);
        let old2 = SelectObject(hdc, font2);
        SetTextColor(hdc, colors.accent);
        let refresh_text = format!("\u{1F504} {}", i18n.t("Refresh"));
        let mut br = *refresh_rect;
        let mut refresh_wide = wide(&refresh_text);
        DrawTextW(hdc, &mut refresh_wide, &mut br, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
        let _ = SelectObject(hdc, old2);
        let _ = DeleteObject(font2);
    }

    unsafe fn draw_text_centered(&self, hdc: HDC, text: &str, rect: RECT, color: ColorRef, size: i32, bold: bool) {
        let font = self.create_font(size, bold);
        let old = SelectObject(hdc, font);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, color);
        let mut r = rect;
        let mut text_wide = wide(text);
        DrawTextW(
            hdc,
            &mut text_wide,
            &mut r,
            windows::Win32::Graphics::Gdi::DT_CENTER | DT_SINGLELINE | DT_VCENTER,
        );
        let _ = SelectObject(hdc, old);
        let _ = DeleteObject(font);
    }

    unsafe fn create_font(&self, size_pt: i32, bold: bool) -> windows::Win32::Graphics::Gdi::HFONT {
        use windows::Win32::Graphics::Gdi::{
            CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
        };
        let height = -self.scale(size_pt);
        let weight = if bold { 700i32 } else { 400i32 };
        let face: Vec<u16> = "Segoe UI".encode_utf16().chain(std::iter::once(0)).collect();
        let mut face_arr = [0u16; 32];
        for (i, &c) in face.iter().enumerate().take(31) {
            face_arr[i] = c;
        }
        CreateFontW(
            height,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            0,
            windows::core::PCWSTR(face_arr.as_ptr()),
        )
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
