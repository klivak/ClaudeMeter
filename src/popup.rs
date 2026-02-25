use crate::config::Config;
use crate::providers::claude::UsageResponse;
use crate::ui::render::{PopupRenderer, POPUP_WIDTH};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::InvalidateRect;
use windows::Win32::UI::WindowsAndMessaging::{
    MoveWindow, SetWindowPos, ShowWindow, SystemParametersInfoW, HWND_TOPMOST, SPI_GETWORKAREA,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, SW_SHOW,
};

#[allow(dead_code)]
pub const WM_UPDATE_DATA: u32 = windows::Win32::UI::WindowsAndMessaging::WM_USER + 10;
#[allow(dead_code)]
pub const WM_SHOW_POPUP: u32 = windows::Win32::UI::WindowsAndMessaging::WM_USER + 11;
#[allow(dead_code)]
pub const WM_HIDE_POPUP: u32 = windows::Win32::UI::WindowsAndMessaging::WM_USER + 12;

#[allow(dead_code)]
pub struct PopupWindow {
    pub hwnd: HWND,
    pub visible: bool,
    pub in_settings: bool,
    pub settings_rect: RECT,
    pub close_rect: RECT,
    pub refresh_rect: RECT,
    pub install_rect: RECT,
    pub chatgpt_link_rect: RECT,
}

impl PopupWindow {
    pub fn invalidate(&self) {
        unsafe {
            let _ = InvalidateRect(self.hwnd, None, true);
        }
    }

    pub fn show_near_tray(&mut self, config: &Config, usage: &Option<UsageResponse>) {
        let height = self.calculate_height(config, usage);
        let (x, y) = get_popup_position(POPUP_WIDTH, height);

        unsafe {
            let _ = MoveWindow(self.hwnd, x, y, POPUP_WIDTH, height, false);
            let _ = SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            );
            let _ = ShowWindow(self.hwnd, SW_SHOW);
        }
        self.visible = true;
    }

    pub fn hide(&mut self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
        self.visible = false;
    }

    fn calculate_height(&self, config: &Config, usage: &Option<UsageResponse>) -> i32 {
        let renderer = PopupRenderer { dpi_scale: 1.0 };
        renderer.calculate_height(usage, config.show_chatgpt_section, config.compact_mode)
    }
}

/// Get position for the popup window — above the taskbar, near the bottom-right.
fn get_popup_position(w: i32, h: i32) -> (i32, i32) {
    unsafe {
        let mut work_area = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut RECT as *mut _),
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        let x = work_area.right - w - 10;
        let y = work_area.bottom - h - 10;
        (x.max(0), y.max(0))
    }
}

/// Check if a point is inside a RECT.
pub fn point_in_rect(pt: POINT, rect: RECT) -> bool {
    pt.x >= rect.left && pt.x < rect.right && pt.y >= rect.top && pt.y < rect.bottom
}
