use crate::providers::claude::UsageResponse;
use crate::ui::colors::ColorRef;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, DeleteDC, DeleteObject, DrawTextW,
    SelectObject, SetBkMode, SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DT_CENTER, DT_SINGLELINE,
    DT_VCENTER, FF_DONTCARE, FW_BOLD, OUT_DEFAULT_PRECIS, PROOF_QUALITY, TRANSPARENT,
};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, DestroyIcon, LoadIconW, LoadImageW, HICON, ICONINFO, IDI_APPLICATION,
    IMAGE_ICON, LR_DEFAULTSIZE, LR_SHARED, WM_USER,
};

// Tray callback message
pub const WM_TRAY_ICON: u32 = WM_USER + 1;
pub const TRAY_ID: u32 = 1;

// Context menu command IDs
pub const IDM_REFRESH: u32 = 1001;
pub const IDM_OPEN_DASHBOARD: u32 = 1002;
pub const IDM_OPEN_CLAUDE: u32 = 1003;
pub const IDM_OPEN_CHATGPT: u32 = 1004;
pub const IDM_SETTINGS: u32 = 1005;
pub const IDM_AUTOSTART: u32 = 1006;
pub const IDM_ABOUT: u32 = 1007;
pub const IDM_EXPORT_CSV: u32 = 1009;
pub const IDM_EXIT: u32 = 1008;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayIconColor {
    Green,
    Yellow,
    Red,
    Gray,
}

impl TrayIconColor {
    pub fn from_utilization(max_util: Option<f64>) -> Self {
        match max_util {
            None => Self::Gray,
            Some(u) if u >= 80.0 => Self::Red,
            Some(u) if u >= 50.0 => Self::Yellow,
            Some(_) => Self::Green,
        }
    }

    fn to_colorref(self) -> ColorRef {
        use crate::ui::colors::rgb;
        match self {
            Self::Green => rgb(64, 160, 43),   // #40a02b
            Self::Yellow => rgb(223, 142, 29), // #df8e1d
            Self::Red => rgb(210, 15, 57),     // #d20f39
            Self::Gray => rgb(128, 128, 128),
        }
    }

    /// Text color for the tray icon number (GDI COLORREF format: 0x00BBGGRR)
    fn text_colorref(self) -> u32 {
        match self {
            Self::Green | Self::Yellow => 0x00000000, // black text on bright backgrounds
            Self::Red | Self::Gray => 0x00FFFFFF,     // white text on dark backgrounds
        }
    }
}

pub struct TrayIcon {
    hwnd: HWND,
    current_color: TrayIconColor,
    icon_green: HICON,
    icon_yellow: HICON,
    icon_red: HICON,
    icon_gray: HICON,
    dynamic_icon: Option<HICON>,
}

// Resource IDs for embedded tray icons (must match build.rs)
const ICON_GREEN_ID: u16 = 101;
const ICON_YELLOW_ID: u16 = 102;
const ICON_RED_ID: u16 = 103;
const ICON_GRAY_ID: u16 = 104;

fn load_icon_from_resource(resource_id: u16) -> Result<HICON, String> {
    unsafe {
        let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
            .map_err(|e| format!("GetModuleHandleW: {e}"))?;
        let handle = LoadImageW(
            hinstance,
            windows::core::PCWSTR(resource_id as usize as *const u16),
            IMAGE_ICON,
            16,
            16,
            LR_DEFAULTSIZE | LR_SHARED,
        )
        .map_err(|e| format!("Failed to load icon resource {resource_id}: {e}"))?;
        Ok(HICON(handle.0))
    }
}

/// Create a 16x16 icon with a percentage number rendered on a colored background.
fn create_number_icon(value: u32, color: ColorRef, text_color: u32) -> Option<HICON> {
    const SIZE: i32 = 16;

    unsafe {
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return None;
        }

        // Extract RGB from COLORREF (0x00BBGGRR)
        let cr = color.0;
        let r = (cr & 0xFF) as u8;
        let g = ((cr >> 8) & 0xFF) as u8;
        let b = ((cr >> 16) & 0xFF) as u8;

        // Create 32-bit DIB section for the color bitmap
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: SIZE,
                biHeight: -SIZE, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0);
        if hbm_color.is_err() {
            let _ = DeleteDC(dc);
            return None;
        }
        let hbm_color = hbm_color.unwrap();

        let old_bm = SelectObject(dc, hbm_color);

        // Fill with the color (BGRA format, fully opaque)
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, (SIZE * SIZE) as usize);
        let bg_pixel = 0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | b as u32;
        for px in pixels.iter_mut() {
            *px = bg_pixel;
        }

        // Draw text
        let text = if value >= 100 {
            "!!".to_string()
        } else {
            format!("{}", value)
        };
        let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        let font_size = if value >= 10 { 9 } else { 11 };
        let font_name: Vec<u16> = "Segoe UI"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut face_name = [0u16; 32];
        let copy_len = font_name.len().min(31);
        face_name[..copy_len].copy_from_slice(&font_name[..copy_len]);

        let font = CreateFontW(
            -font_size,
            0,
            0,
            0,
            FW_BOLD.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            PROOF_QUALITY.0 as u32,
            (DEFAULT_PITCH.0 | FF_DONTCARE.0) as u32,
            windows::core::PCWSTR(face_name.as_ptr()),
        );

        let old_font = SelectObject(dc, font);
        let _ = SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, windows::Win32::Foundation::COLORREF(text_color));

        let mut rc = RECT {
            left: 0,
            top: 0,
            right: SIZE,
            bottom: SIZE,
        };
        DrawTextW(
            dc,
            &mut text_wide[..text_wide.len() - 1].to_vec(),
            &mut rc,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );

        // Make text pixels fully opaque (DrawTextW doesn't set alpha)
        let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, (SIZE * SIZE) as usize);
        for px in pixels.iter_mut() {
            if *px != bg_pixel {
                *px |= 0xFF000000; // set alpha to 255
            }
        }

        SelectObject(dc, old_font);
        SelectObject(dc, old_bm);
        let _ = DeleteObject(font);

        // Create mask bitmap (all zeros = fully opaque)
        let mask_bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: SIZE,
                biHeight: -SIZE,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut mask_bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let hbm_mask = CreateDIBSection(dc, &mask_bmi, DIB_RGB_COLORS, &mut mask_bits, None, 0);
        if hbm_mask.is_err() {
            let _ = DeleteObject(hbm_color);
            let _ = DeleteDC(dc);
            return None;
        }
        let hbm_mask = hbm_mask.unwrap();

        // mask_bits is already zeroed (all opaque)

        let icon_info = ICONINFO {
            fIcon: true.into(),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: hbm_mask,
            hbmColor: hbm_color,
        };

        let icon = CreateIconIndirect(&icon_info).ok();

        let _ = DeleteObject(hbm_color);
        let _ = DeleteObject(hbm_mask);
        let _ = DeleteDC(dc);

        icon
    }
}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Result<Self, String> {
        let fallback = unsafe { LoadIconW(None, IDI_APPLICATION).map_err(|e| e.to_string())? };

        let icon_green = load_icon_from_resource(ICON_GREEN_ID).unwrap_or(fallback);
        let icon_yellow = load_icon_from_resource(ICON_YELLOW_ID).unwrap_or(fallback);
        let icon_red = load_icon_from_resource(ICON_RED_ID).unwrap_or(fallback);
        let icon_gray = load_icon_from_resource(ICON_GRAY_ID).unwrap_or(fallback);

        let tray = Self {
            hwnd,
            current_color: TrayIconColor::Gray,
            icon_green,
            icon_yellow,
            icon_red,
            icon_gray,
            dynamic_icon: None,
        };

        tray.add_to_tray()?;
        Ok(tray)
    }

    fn add_to_tray(&self) -> Result<(), String> {
        let mut nid = self.make_nid();
        nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
        nid.hIcon = self.icon_gray;
        nid.uCallbackMessage = WM_TRAY_ICON;
        let tip = "ClaudeMeter";
        let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = tip_wide.len().min(127);
        nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

        unsafe {
            Shell_NotifyIconW(NIM_ADD, &nid)
                .ok()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn update(&mut self, usage: &Option<UsageResponse>, tooltip: &str) {
        let max_util = usage.as_ref().and_then(|u| u.max_utilization());
        let color = TrayIconColor::from_utilization(max_util);

        // Try dynamic icon with % number, fall back to static icons
        let icon = if let Some(pct) = max_util {
            let color_ref = color.to_colorref();
            let text_cr = color.text_colorref();
            if let Some(dyn_icon) = create_number_icon(pct.round() as u32, color_ref, text_cr) {
                // Destroy previous dynamic icon
                if let Some(old) = self.dynamic_icon.take() {
                    unsafe {
                        let _ = DestroyIcon(old);
                    }
                }
                self.dynamic_icon = Some(dyn_icon);
                dyn_icon
            } else {
                self.fallback_icon(color)
            }
        } else {
            self.fallback_icon(color)
        };

        let mut nid = self.make_nid();
        nid.uFlags = NIF_ICON | NIF_TIP;
        nid.hIcon = icon;

        // Truncate tooltip to 127 chars (Win32 limit is 128 with null)
        let truncated: String = tooltip.chars().take(127).collect();
        let tip_wide: Vec<u16> = truncated.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = tip_wide.len().min(127);
        nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
        self.current_color = color;
    }

    fn fallback_icon(&self, color: TrayIconColor) -> HICON {
        match color {
            TrayIconColor::Green => self.icon_green,
            TrayIconColor::Yellow => self.icon_yellow,
            TrayIconColor::Red => self.icon_red,
            TrayIconColor::Gray => self.icon_gray,
        }
    }

    fn make_nid(&self) -> NOTIFYICONDATAW {
        NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: TRAY_ID,
            ..Default::default()
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let nid = self.make_nid();
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            if let Some(icon) = self.dynamic_icon.take() {
                let _ = DestroyIcon(icon);
            }
        }
    }
}

/// Build the tooltip string from usage data (max 127 chars).
pub fn build_tooltip(usage: &Option<UsageResponse>, show_chatgpt: bool) -> String {
    use crate::i18n::format_duration;
    use crate::providers::claude::format_metric_name;

    let mut lines = vec!["ClaudeMeter".to_string()];

    match usage {
        None => {
            lines.push("No data".to_string());
        }
        Some(u) => {
            lines.push(format!("Claude ({})", u.detected_plan()));
            for (key, metric) in u.all_metrics() {
                let name = format_metric_name(&key);
                let reset_str = metric
                    .resets_at
                    .as_deref()
                    .and_then(crate::i18n::seconds_until)
                    .map(|s| format!(" | {}", format_duration(s)))
                    .unwrap_or_default();
                lines.push(String::new()); // empty line between metrics
                lines.push(format!("{}: {:.0}%{}", name, metric.utilization, reset_str));
            }
        }
    }

    if show_chatgpt {
        lines.push("ChatGPT: click to open usage".to_string());
    }

    // Join and truncate to 127 chars
    let full = lines.join("\n");
    full.chars().take(127).collect()
}
