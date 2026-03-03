//! Mini floating widget — always-on-top small window showing current usage %.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, FillRect, SelectObject,
    SetBkMode, SetTextColor, TextOutW, HBRUSH, HGDIOBJ, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, LoadCursorW, RegisterClassExW, CS_HREDRAW,
    CS_VREDRAW, IDC_ARROW, WM_LBUTTONUP, WM_NCHITTEST, WM_PAINT, WNDCLASSEXW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::ui::colors::rgb;

pub const WIDGET_CLASS: &str = "ClaudeMeterWidget";
const WIDGET_W: i32 = 52;
const WIDGET_H: i32 = 28;

/// Register the widget window class.
pub unsafe fn register_widget_class() {
    let hinstance = GetModuleHandleW(None).unwrap_or_default();
    let class_name: Vec<u16> = WIDGET_CLASS
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(widget_wnd_proc),
        hInstance: hinstance.into(),
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        hbrBackground: HBRUSH(std::ptr::null_mut()),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    RegisterClassExW(&wc);
}

/// Create the widget window (hidden by default).
pub unsafe fn create_widget_window() -> Option<HWND> {
    let hinstance = GetModuleHandleW(None).unwrap_or_default();
    let class_name: Vec<u16> = WIDGET_CLASS
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let title: Vec<u16> = "ClaudeMeter Widget"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_LAYERED,
        PCWSTR(class_name.as_ptr()),
        PCWSTR(title.as_ptr()),
        WS_POPUP,
        100,
        100,
        WIDGET_W,
        WIDGET_H,
        None,
        None,
        hinstance,
        None,
    )
    .ok()?;

    // Set 230/255 opacity
    let _ = windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes(
        hwnd,
        windows::Win32::Foundation::COLORREF(0),
        230,
        windows::Win32::UI::WindowsAndMessaging::LWA_ALPHA,
    );

    // Round corners on Win11
    let corner_pref: u32 = 2;
    let _ = windows::Win32::Graphics::Dwm::DwmSetWindowAttribute(
        hwnd,
        windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(33),
        &corner_pref as *const u32 as *const _,
        std::mem::size_of::<u32>() as u32,
    );

    Some(hwnd)
}

unsafe extern "system" fn widget_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_NCHITTEST => {
            // Make entire widget draggable
            LRESULT(2) // HTCAPTION
        }
        WM_LBUTTONUP => {
            // Click opens main popup
            if let Some(state) = crate::APP_STATE.as_ref() {
                let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(
                    state.main_hwnd,
                    crate::tray::WM_TRAY_ICON,
                    WPARAM(0),
                    LPARAM(WM_LBUTTONUP as isize),
                );
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let _hdc = BeginPaint(hwnd, &mut ps);
            let hdc = ps.hdc;

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            // Get current usage text and color
            let (text, bg_color) = if let Some(state) = crate::APP_STATE.as_ref() {
                let max_util = state.usage.as_ref().and_then(|u| u.max_utilization());
                match max_util {
                    Some(u) if u >= 80.0 => (format!("{}%", u.round() as u32), rgb(210, 15, 57)),
                    Some(u) if u >= 50.0 => (format!("{}%", u.round() as u32), rgb(223, 142, 29)),
                    Some(u) => (format!("{}%", u.round() as u32), rgb(64, 160, 43)),
                    None => ("—".to_string(), rgb(128, 128, 128)),
                }
            } else {
                ("—".to_string(), rgb(128, 128, 128))
            };

            // Fill background
            let bg_brush = CreateSolidBrush(bg_color);
            FillRect(hdc, &rect, bg_brush);
            let _ = DeleteObject(HGDIOBJ(bg_brush.0));

            // Draw text
            let font = CreateFontW(
                16,
                0,
                0,
                0,
                700,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                PCWSTR(
                    "Segoe UI"
                        .encode_utf16()
                        .chain(std::iter::once(0))
                        .collect::<Vec<u16>>()
                        .as_ptr(),
                ),
            );
            let old_font = SelectObject(hdc, HGDIOBJ(font.0 as *mut _));
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, windows::Win32::Foundation::COLORREF(0x00FFFFFF)); // white

            let text_wide: Vec<u16> = text.encode_utf16().collect();
            // Center text
            let cx = (rect.right - rect.left) / 2;
            let cy = (rect.bottom - rect.top) / 2;

            // Simple centering: approximate char width
            let text_w = text_wide.len() as i32 * 7;
            let x = cx - text_w / 2;
            let y = cy - 8;
            let _ = TextOutW(hdc, x, y, &text_wide);

            let _ = SelectObject(hdc, old_font);
            let _ = DeleteObject(HGDIOBJ(font.0 as *mut _));

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Update the widget display (trigger repaint).
pub unsafe fn invalidate_widget(hwnd: HWND) {
    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
}
