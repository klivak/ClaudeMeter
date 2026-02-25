#![windows_subsystem = "windows"]
#![allow(static_mut_refs)]

mod autostart;
mod config;
mod credentials;
mod db;
mod i18n;
mod notifications;
mod popup;
mod providers;
mod theme;
mod tray;
mod ui;

use crate::config::ConfigManager;
use crate::db::Database;
use crate::i18n::{format_duration, I18n};
use crate::notifications::{send_toast, NotificationTracker};
use crate::providers::claude::{ClaudeClient, UsageResponse};
use crate::theme::{resolve_theme, ThemeMode};
use crate::tray::{
    build_tooltip, TrayIcon, IDM_ABOUT, IDM_AUTOSTART, IDM_EXIT, IDM_OPEN_CHATGPT, IDM_OPEN_CLAUDE,
    IDM_OPEN_DASHBOARD, IDM_REFRESH, IDM_SETTINGS, WM_TRAY_ICON,
};
use crate::ui::render::PopupRenderer;
use chrono::Local;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, LoadIconW, PeekMessageW, PostMessageW, PostQuitMessage, RegisterClassExW,
    SetForegroundWindow, TrackPopupMenu, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HMENU,
    IDI_APPLICATION, MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, PM_REMOVE,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RETURNCMD, WM_COMMAND, WM_DESTROY, WM_KILLFOCUS,
    WM_LBUTTONUP, WM_PAINT, WM_RBUTTONUP, WM_TIMER, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

const WINDOW_CLASS: &str = "ClaudeMeterMain";
const POPUP_CLASS: &str = "ClaudeMeterPopup";
const TIMER_POLL: usize = 1;
const TIMER_POLL_INTERVAL_MS: u32 = 120_000; // 2 minutes
const WM_POLL_RESULT: u32 = 0x0400 + 20; // WM_USER + 20

/// Shared application state accessible from the window proc.
struct AppState {
    config_mgr: ConfigManager,
    i18n: I18n,
    tray: Option<TrayIcon>,
    usage: Option<UsageResponse>,
    last_updated: String,
    last_error: Option<String>,
    popup_hwnd: HWND,
    popup_visible: bool,
    popup_in_settings: bool,
    // Hit-test rectangles (in popup client coordinates)
    settings_rect: RECT,
    close_rect: RECT,
    refresh_rect: RECT,
    install_rect: RECT,
    chatgpt_link_rect: RECT,
    notification_tracker: NotificationTracker,
    exe_dir: std::path::PathBuf,
    chart_data: Vec<f64>,
}

// Safety: AppState is accessed only from the main thread via raw pointer.
unsafe impl Send for AppState {}
unsafe impl Sync for AppState {}

static mut APP_STATE: Option<AppState> = None;

fn main() {
    env_logger::init();

    // Single-instance check
    if !ensure_single_instance() {
        log::info!("Another instance is already running.");
        return;
    }

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_default();

    let config_mgr = ConfigManager::new(&exe_dir);
    let i18n = I18n::from_config(&config_mgr.config.language);

    // Try to open DB early to verify it's accessible, log warning if not.
    if let Err(e) = Database::open(&exe_dir) {
        log::warn!("Database open check failed: {e}. History will not be saved.");
    }

    unsafe { run_message_loop(exe_dir, config_mgr, i18n) };
}

unsafe fn run_message_loop(exe_dir: std::path::PathBuf, config_mgr: ConfigManager, i18n: I18n) {
    let hinstance = GetModuleHandleW(None).unwrap();

    // Register window classes
    register_main_class(hinstance);
    register_popup_class(hinstance);

    // Create hidden message window
    let main_class_w = wide(WINDOW_CLASS);
    let main_title_w = wide("ClaudeMeter");
    let popup_class_w = wide(POPUP_CLASS);
    let popup_title_w = wide("ClaudeMeter Dashboard");

    let main_hwnd = CreateWindowExW(
        windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(0),
        PCWSTR(main_class_w.as_ptr()),
        PCWSTR(main_title_w.as_ptr()),
        windows::Win32::UI::WindowsAndMessaging::WINDOW_STYLE(0),
        0,
        0,
        0,
        0,
        None,
        None,
        hinstance,
        None,
    )
    .unwrap();

    // Create popup window (hidden initially)
    let popup_hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        PCWSTR(popup_class_w.as_ptr()),
        PCWSTR(popup_title_w.as_ptr()),
        WS_POPUP,
        0,
        0,
        crate::ui::render::POPUP_WIDTH,
        400,
        None,
        None,
        hinstance,
        None,
    )
    .unwrap();

    // Initialize app state
    APP_STATE = Some(AppState {
        config_mgr,
        i18n,
        tray: None,
        usage: None,
        last_updated: String::new(),
        last_error: None,
        popup_hwnd,
        popup_visible: false,
        popup_in_settings: false,
        settings_rect: RECT::default(),
        close_rect: RECT::default(),
        refresh_rect: RECT::default(),
        install_rect: RECT::default(),
        chatgpt_link_rect: RECT::default(),
        notification_tracker: NotificationTracker::new(),
        exe_dir,
        chart_data: Vec::new(),
    });

    // Create tray icon
    if let Some(state) = APP_STATE.as_mut() {
        match TrayIcon::new(main_hwnd) {
            Ok(tray) => state.tray = Some(tray),
            Err(e) => log::error!("Failed to create tray icon: {e}"),
        }
    }

    // Initial poll (async via tokio)
    trigger_poll(main_hwnd);

    // Set up polling timer
    let interval = APP_STATE
        .as_ref()
        .map(|s| s.config_mgr.config.polling_interval_clamped() as u32 * 1000)
        .unwrap_or(TIMER_POLL_INTERVAL_MS);
    windows::Win32::UI::WindowsAndMessaging::SetTimer(main_hwnd, TIMER_POLL, interval, None);

    // Message loop
    let mut msg = MSG::default();
    loop {
        let has_msg = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
        if has_msg {
            if msg.message == windows::Win32::UI::WindowsAndMessaging::WM_QUIT {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        } else {
            // Yield CPU when idle
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

unsafe fn register_main_class(hinstance: windows::Win32::Foundation::HMODULE) {
    let class_name = wide(WINDOW_CLASS);
    let mut wc = WNDCLASSEXW::default();
    wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = Some(main_wnd_proc);
    wc.hInstance = hinstance.into();
    wc.hIcon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
    wc.lpszClassName = PCWSTR(class_name.as_ptr());
    RegisterClassExW(&wc);
}

unsafe fn register_popup_class(hinstance: windows::Win32::Foundation::HMODULE) {
    let class_name = wide(POPUP_CLASS);
    let mut wc = WNDCLASSEXW::default();
    wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = Some(popup_wnd_proc);
    wc.hInstance = hinstance.into();
    wc.hIcon = LoadIconW(None, IDI_APPLICATION).unwrap_or_default();
    wc.lpszClassName = PCWSTR(class_name.as_ptr());
    // COLOR_WINDOW = 5, so (COLOR_WINDOW + 1) = 6 as background brush
    wc.hbrBackground = windows::Win32::Graphics::Gdi::HBRUSH(6usize as *mut _);
    RegisterClassExW(&wc);
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAY_ICON => {
            let event = (lparam.0 & 0xFFFF) as u32;
            match event {
                WM_LBUTTONUP => {
                    toggle_popup(hwnd);
                }
                WM_RBUTTONUP => {
                    show_context_menu(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let cmd = (wparam.0 & 0xFFFF) as u32;
            handle_menu_command(hwnd, cmd);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 as usize == TIMER_POLL {
                trigger_poll(hwnd);
            }
            LRESULT(0)
        }
        WM_POLL_RESULT => {
            // Poll result received (usage data posted back to main thread)
            // wparam = pointer to Box<Option<UsageResponse>>
            // lparam = pointer to Box<Option<String>> (error)
            let usage_ptr = wparam.0 as *mut Option<UsageResponse>;
            let err_ptr = lparam.0 as *mut Option<String>;
            if !usage_ptr.is_null() {
                let usage = *Box::from_raw(usage_ptr);
                let err = if !err_ptr.is_null() {
                    *Box::from_raw(err_ptr)
                } else {
                    None
                };
                on_poll_result(hwnd, usage, err);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe extern "system" fn popup_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect = RECT::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect);

            if let Some(state) = APP_STATE.as_mut() {
                state.config_mgr.reload_if_changed();
                let theme_mode = ThemeMode::from_str(&state.config_mgr.config.theme);
                let resolved = resolve_theme(theme_mode);
                let colors = crate::ui::colors::ThemeColors::for_theme(resolved);
                let renderer = PopupRenderer::new(hwnd);

                if state.popup_in_settings {
                    draw_settings_panel(hdc, &rect, &colors, &state.i18n, &state.config_mgr.config);
                } else {
                    renderer.draw(
                        hdc,
                        &rect,
                        &state.usage,
                        &state.last_updated,
                        state.config_mgr.config.show_chatgpt_section,
                        state.config_mgr.config.compact_mode,
                        &colors,
                        &state.i18n,
                        &state.chart_data,
                        &mut state.settings_rect,
                        &mut state.close_rect,
                        &mut state.refresh_rect,
                        &mut state.install_rect,
                        &mut state.chatgpt_link_rect,
                    );
                }
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i32;
            let pt = POINT { x, y };

            if let Some(state) = APP_STATE.as_mut() {
                if crate::popup::point_in_rect(pt, state.close_rect) {
                    state.popup_visible = false;
                    let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                        hwnd,
                        windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
                    );
                } else if crate::popup::point_in_rect(pt, state.settings_rect) {
                    state.popup_in_settings = !state.popup_in_settings;
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if crate::popup::point_in_rect(pt, state.refresh_rect) {
                    // Find main hwnd and trigger poll
                    trigger_poll(hwnd); // Note: passes popup hwnd; poll result goes to main
                } else if crate::popup::point_in_rect(pt, state.install_rect) {
                    let url = state.config_mgr.config.claude_install_url.clone();
                    let _ = open::that(&url);
                } else if crate::popup::point_in_rect(pt, state.chatgpt_link_rect) {
                    let url = state.config_mgr.config.chatgpt_usage_url.clone();
                    let _ = open::that(&url);
                }
            }
            LRESULT(0)
        }
        // Close popup when clicking outside (WM_KILLFOCUS)
        WM_KILLFOCUS => {
            if let Some(state) = APP_STATE.as_mut() {
                if state.popup_visible {
                    state.popup_visible = false;
                    let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                        hwnd,
                        windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
                    );
                }
            }
            LRESULT(0)
        }
        WM_DESTROY => LRESULT(0),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn draw_settings_panel(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    colors: &crate::ui::colors::ThemeColors,
    i18n: &I18n,
    config: &crate::config::Config,
) {
    use windows::Win32::Graphics::Gdi::{
        CreateSolidBrush, DeleteObject, DrawTextW, FillRect, SelectObject, SetBkMode, SetTextColor,
        TRANSPARENT,
    };

    let bg = CreateSolidBrush(colors.background);
    let _ = FillRect(hdc, rect, bg);
    let _ = DeleteObject(bg);

    let w = rect.right - rect.left;
    let pad = 16i32;

    // Header
    let surf = CreateSolidBrush(colors.surface);
    let header = RECT {
        left: 0,
        top: 0,
        right: w,
        bottom: 36,
    };
    let _ = FillRect(hdc, &header, surf);
    let _ = DeleteObject(surf);

    let font = create_font_helper(hdc, 13, true);
    let old = SelectObject(hdc, font);
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, colors.text_primary);
    let mut back_text = wide(i18n.t("Back"));
    let mut back_rect = RECT {
        left: pad,
        top: 0,
        right: w - 40,
        bottom: 36,
    };
    DrawTextW(
        hdc,
        &mut back_text,
        &mut back_rect,
        windows::Win32::Graphics::Gdi::DT_LEFT
            | windows::Win32::Graphics::Gdi::DT_SINGLELINE
            | windows::Win32::Graphics::Gdi::DT_VCENTER,
    );

    // Close button
    SetTextColor(hdc, colors.text_secondary);
    let mut close_text = wide("\u{00D7}");
    let mut cr = RECT {
        left: w - 32,
        top: 0,
        right: w - 4,
        bottom: 36,
    };
    DrawTextW(
        hdc,
        &mut close_text,
        &mut cr,
        windows::Win32::Graphics::Gdi::DT_CENTER
            | windows::Win32::Graphics::Gdi::DT_SINGLELINE
            | windows::Win32::Graphics::Gdi::DT_VCENTER,
    );

    let _ = SelectObject(hdc, old);
    let _ = DeleteObject(font);

    let mut y = 44i32;

    // Settings rows
    let rows: &[(&str, &str)] = &[
        (
            "Theme",
            &format!(
                "{}: {}",
                i18n.t("Theme"),
                i18n.t(&capitalize(&config.theme))
            ),
        ),
        (
            "Language",
            &format!("{}: {}", i18n.t("Language"), &config.language),
        ),
        (
            "Compact mode",
            &format!(
                "[{}] {}",
                if config.compact_mode { "x" } else { " " },
                i18n.t("Compact mode")
            ),
        ),
        (
            "Show ChatGPT section",
            &format!(
                "[{}] {}",
                if config.show_chatgpt_section {
                    "x"
                } else {
                    " "
                },
                i18n.t("Show ChatGPT section")
            ),
        ),
        (
            "Start with Windows",
            &format!(
                "[{}] {}",
                if config.autostart { "x" } else { " " },
                i18n.t("Start with Windows")
            ),
        ),
    ];

    for (_, row_text) in rows {
        let f = create_font_helper(hdc, 11, false);
        let old2 = SelectObject(hdc, f);
        SetBkMode(hdc, TRANSPARENT);
        SetTextColor(hdc, colors.text_primary);
        let mut r = RECT {
            left: pad,
            top: y,
            right: w - pad,
            bottom: y + 22,
        };
        let mut rw = wide(row_text);
        DrawTextW(
            hdc,
            &mut rw,
            &mut r,
            windows::Win32::Graphics::Gdi::DT_LEFT
                | windows::Win32::Graphics::Gdi::DT_SINGLELINE
                | windows::Win32::Graphics::Gdi::DT_VCENTER,
        );
        let _ = SelectObject(hdc, old2);
        let _ = DeleteObject(f);
        y += 26;
    }

    // Footer
    y = rect.bottom - 50;
    let f3 = create_font_helper(hdc, 10, false);
    let old3 = SelectObject(hdc, f3);
    SetBkMode(hdc, TRANSPARENT);
    SetTextColor(hdc, colors.text_secondary);
    let mut footer = wide("ClaudeMeter v1.0.0 by klivak\ngithub.com/klivak/claudemeter");
    let mut fr = RECT {
        left: pad,
        top: y,
        right: w - pad,
        bottom: rect.bottom,
    };
    DrawTextW(
        hdc,
        &mut footer,
        &mut fr,
        windows::Win32::Graphics::Gdi::DT_LEFT | windows::Win32::Graphics::Gdi::DT_WORDBREAK,
    );
    let _ = SelectObject(hdc, old3);
    let _ = DeleteObject(f3);
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

unsafe fn create_font_helper(
    _hdc: windows::Win32::Graphics::Gdi::HDC,
    size_pt: i32,
    bold: bool,
) -> windows::Win32::Graphics::Gdi::HFONT {
    use windows::Win32::Graphics::Gdi::{
        CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
    };
    let height = -(size_pt * 96 / 72);
    let weight = if bold { 700i32 } else { 400i32 };
    let face: Vec<u16> = "Segoe UI"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut face_arr = [0u16; 32];
    for (i, &c) in face.iter().enumerate().take(31) {
        face_arr[i] = c;
    }
    windows::Win32::Graphics::Gdi::CreateFontW(
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

unsafe fn toggle_popup(main_hwnd: HWND) {
    if let Some(state) = APP_STATE.as_mut() {
        if state.popup_visible {
            state.popup_visible = false;
            let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
                state.popup_hwnd,
                windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
            );
        } else {
            show_popup(main_hwnd);
        }
    }
}

unsafe fn show_popup(_main_hwnd: HWND) {
    if let Some(state) = APP_STATE.as_mut() {
        state.config_mgr.reload_if_changed();

        let theme_mode = ThemeMode::from_str(&state.config_mgr.config.theme);
        let _resolved = resolve_theme(theme_mode);

        // Calculate height
        let renderer = PopupRenderer::new(state.popup_hwnd);
        let h = renderer.calculate_height(
            &state.usage,
            state.config_mgr.config.show_chatgpt_section,
            state.config_mgr.config.compact_mode,
        );

        // Position near taskbar
        let mut work_area = RECT::default();
        let _ = windows::Win32::UI::WindowsAndMessaging::SystemParametersInfoW(
            windows::Win32::UI::WindowsAndMessaging::SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut RECT as *mut _),
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        let x = work_area.right - crate::ui::render::POPUP_WIDTH - 10;
        let y = work_area.bottom - h - 10;

        let _ = windows::Win32::UI::WindowsAndMessaging::MoveWindow(
            state.popup_hwnd,
            x.max(0),
            y.max(0),
            crate::ui::render::POPUP_WIDTH,
            h,
            false,
        );
        let _ = windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
            state.popup_hwnd,
            windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST,
            0,
            0,
            0,
            0,
            windows::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                | windows::Win32::UI::WindowsAndMessaging::SWP_NOSIZE
                | windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
        );
        let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
            state.popup_hwnd,
            windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
        );
        let _ = SetForegroundWindow(state.popup_hwnd);
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(state.popup_hwnd, None, true);
        state.popup_visible = true;
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    if let Some(state) = APP_STATE.as_ref() {
        let menu = CreatePopupMenu().unwrap();
        let show_chatgpt = state.config_mgr.config.show_chatgpt_section;
        let autostart = state.config_mgr.config.autostart;

        append_menu_str(menu, IDM_REFRESH, state.i18n.t("Refresh Now"));
        append_menu_sep(menu);
        append_menu_str(menu, IDM_OPEN_DASHBOARD, state.i18n.t("Open Dashboard"));
        append_menu_sep(menu);
        append_menu_str(menu, IDM_OPEN_CLAUDE, "Open Claude.ai \u{2192}");
        if show_chatgpt {
            append_menu_str(
                menu,
                IDM_OPEN_CHATGPT,
                state.i18n.t("Open ChatGPT Usage \u{2192}"),
            );
        }
        append_menu_sep(menu);
        append_menu_str(menu, IDM_SETTINGS, state.i18n.t("Settings"));
        // Autostart toggle with checkmark
        let autostart_flag = if autostart {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING | MF_UNCHECKED
        };
        let autostart_text = wide(state.i18n.t("Start with Windows"));
        let _ = AppendMenuW(
            menu,
            autostart_flag,
            IDM_AUTOSTART as usize,
            PCWSTR(autostart_text.as_ptr()),
        );
        append_menu_sep(menu);
        append_menu_str(menu, IDM_ABOUT, &format!("ClaudeMeter v1.0.0"));
        append_menu_str(menu, IDM_EXIT, state.i18n.t("Exit"));

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        let _ = SetForegroundWindow(hwnd);
        let cmd = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
            pt.x,
            pt.y,
            0,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);

        if cmd.as_bool() {
            handle_menu_command(hwnd, cmd.0 as u32);
        }
    }
}

unsafe fn append_menu_str(menu: HMENU, id: u32, text: &str) {
    let wide_text = wide(text);
    let _ = AppendMenuW(menu, MF_STRING, id as usize, PCWSTR(wide_text.as_ptr()));
}

unsafe fn append_menu_sep(menu: HMENU) {
    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
}

unsafe fn handle_menu_command(hwnd: HWND, cmd: u32) {
    match cmd {
        IDM_REFRESH => {
            trigger_poll(hwnd);
        }
        IDM_OPEN_DASHBOARD => {
            show_popup(hwnd);
        }
        IDM_OPEN_CLAUDE => {
            let _ = open::that("https://claude.ai");
        }
        IDM_OPEN_CHATGPT => {
            if let Some(state) = APP_STATE.as_ref() {
                let url = state.config_mgr.config.chatgpt_usage_url.clone();
                let _ = open::that(&url);
            }
        }
        IDM_SETTINGS => {
            if let Some(state) = APP_STATE.as_mut() {
                state.popup_in_settings = true;
                show_popup(hwnd);
            }
        }
        IDM_AUTOSTART => {
            if let Some(state) = APP_STATE.as_mut() {
                let new_val = !state.config_mgr.config.autostart;
                state.config_mgr.config.autostart = new_val;
                state.config_mgr.save();
                let exe_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                let _ = autostart::set_autostart(new_val, &exe_path);
            }
        }
        IDM_ABOUT => {
            // Show simple about message
            let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                hwnd,
                windows::core::PCWSTR(wide("ClaudeMeter v1.0.0\nby klivak\nhttps://github.com/klivak/claudemeter\n\nMIT License").as_ptr()),
                windows::core::PCWSTR(wide("About ClaudeMeter").as_ptr()),
                windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION | windows::Win32::UI::WindowsAndMessaging::MB_OK,
            );
        }
        IDM_EXIT => {
            // Clean up tray icon
            if let Some(state) = APP_STATE.as_mut() {
                state.tray = None;
            }
            PostQuitMessage(0);
        }
        _ => {}
    }
}

/// Spawn async poll task. Result is posted back to main hwnd via WM_USER+20.
unsafe fn trigger_poll(hwnd: HWND) {
    // We run a background thread for async work to avoid blocking the message loop.
    // tokio::spawn would require a runtime, so we use std::thread + tokio block_on.
    let hwnd_val = hwnd.0 as isize;

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(async move {
            let (usage, error) = do_poll().await;

            // Post result back to main thread
            let hwnd = HWND(hwnd_val as *mut _);
            let usage_box = Box::new(usage);
            let err_box = Box::new(error);
            let usage_ptr = Box::into_raw(usage_box) as isize;
            let err_ptr = Box::into_raw(err_box) as isize;

            let _ = PostMessageW(
                hwnd,
                WM_POLL_RESULT,
                WPARAM(usage_ptr as usize),
                LPARAM(err_ptr),
            );
        });
    });
}

async fn do_poll() -> (Option<UsageResponse>, Option<String>) {
    let token = match credentials::read_claude_token() {
        Ok(t) => t,
        Err(e) => {
            log::warn!("Could not read Claude token: {e}");
            return (None, Some(e.to_string()));
        }
    };

    let client = match ClaudeClient::new() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to create HTTP client: {e}");
            return (None, Some(e));
        }
    };

    match client.fetch_usage(&token).await {
        Ok(usage) => (Some(usage), None),
        Err(e) => {
            log::warn!("Failed to fetch usage: {e}");
            (None, Some(e))
        }
    }
}

unsafe fn on_poll_result(_hwnd: HWND, usage: Option<UsageResponse>, error: Option<String>) {
    if let Some(state) = APP_STATE.as_mut() {
        if let Some(u) = &usage {
            state.last_updated = Local::now().format("%H:%M:%S").to_string();

            // Store to DB (best effort)
            if let Ok(db) = Database::open(&state.exe_dir) {
                for (key, metric) in u.all_metrics() {
                    let _ = db.insert(
                        "claude",
                        &key,
                        metric.utilization,
                        metric.resets_at.as_deref(),
                    );
                }
                // Load chart data
                if let Ok(points) = db.query_24h_chart() {
                    state.chart_data = points.iter().map(|p| p.utilization).collect();
                }
            }

            // Check notifications
            let thresholds = state.config_mgr.config.notifications.thresholds.clone();
            if state.config_mgr.config.notifications.enabled {
                for (key, metric) in u.all_metrics() {
                    let fired =
                        state
                            .notification_tracker
                            .check(&key, metric.utilization, &thresholds);
                    for threshold in fired {
                        let metric_name = providers::claude::format_metric_name(&key);
                        let reset_info = metric
                            .resets_at
                            .as_deref()
                            .and_then(|r| i18n::seconds_until(r))
                            .map(|s| format!(" Resets in {}.", format_duration(s)))
                            .unwrap_or_default();

                        let (title, body) = if threshold >= 90 {
                            (
                                format!("\u{1F534} {}", state.i18n.t("Usage Critical")),
                                format!(
                                    "{} at {:.0}%!{}",
                                    metric_name, metric.utilization, reset_info
                                ),
                            )
                        } else {
                            (
                                format!("\u{26A0} {}", state.i18n.t("Usage Alert")),
                                format!(
                                    "{} at {:.0}%.{}",
                                    metric_name, metric.utilization, reset_info
                                ),
                            )
                        };
                        send_toast(&title, &body);
                    }
                }
            }
        }

        state.usage = usage;
        state.last_error = error;

        // Update tray
        let tooltip = build_tooltip(&state.usage, state.config_mgr.config.show_chatgpt_section);
        if let Some(tray) = &mut state.tray {
            tray.update(&state.usage, &tooltip);
        }

        // Refresh popup if visible
        if state.popup_visible {
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(state.popup_hwnd, None, true);
        }
    }
}

fn ensure_single_instance() -> bool {
    let name = wide("ClaudeMeter-SingleInstance");
    unsafe {
        let mutex = CreateMutexW(None, true, windows::core::PCWSTR(name.as_ptr()));
        match mutex {
            Ok(_) => {
                let last_err = GetLastError();
                // ERROR_ALREADY_EXISTS = 183
                if last_err.0 == 183 {
                    return false;
                }
                true
            }
            Err(_) => true, // Proceed anyway if we can't create mutex
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
