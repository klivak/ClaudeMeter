use crate::providers::claude::UsageResponse;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, HICON, IDI_APPLICATION, WM_USER};

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
}

pub struct TrayIcon {
    hwnd: HWND,
    current_color: TrayIconColor,
    icon_green: HICON,
    icon_yellow: HICON,
    icon_red: HICON,
    icon_gray: HICON,
}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Result<Self, String> {
        // For now, use default application icon as placeholder
        // In a full build, these would be loaded from embedded resources
        let default_icon = unsafe { LoadIconW(None, IDI_APPLICATION).map_err(|e| e.to_string())? };

        let tray = Self {
            hwnd,
            current_color: TrayIconColor::Gray,
            icon_green: default_icon,
            icon_yellow: default_icon,
            icon_red: default_icon,
            icon_gray: default_icon,
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

        let icon = match color {
            TrayIconColor::Green => self.icon_green,
            TrayIconColor::Yellow => self.icon_yellow,
            TrayIconColor::Red => self.icon_red,
            TrayIconColor::Gray => self.icon_gray,
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

    fn make_nid(&self) -> NOTIFYICONDATAW {
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd;
        nid.uID = TRAY_ID;
        nid
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let nid = self.make_nid();
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
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
                    .and_then(|r| crate::i18n::seconds_until(r))
                    .map(|s| format!(" | {}", format_duration(s)))
                    .unwrap_or_default();
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
