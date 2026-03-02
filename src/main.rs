#![windows_subsystem = "windows"]
#![allow(static_mut_refs)]
#![allow(clippy::too_many_arguments)]

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
use crate::ui::colors::colorref_to_d2d;
use crate::ui::render::{draw_settings_panel, D2DResources, HoveredElement, PopupRenderer};
use chrono::Local;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, LoadCursorW, LoadIconW, PeekMessageW, PostMessageW, PostQuitMessage,
    RegisterClassExW, SetCursor, SetForegroundWindow, TrackPopupMenu, TranslateMessage,
    CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, HMENU, IDC_ARROW, IDC_HAND, IDI_APPLICATION, MF_CHECKED,
    MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, PM_REMOVE, TPM_BOTTOMALIGN, TPM_LEFTALIGN,
    TPM_RETURNCMD, WM_COMMAND, WM_DESTROY, WM_KILLFOCUS, WM_LBUTTONUP, WM_MOUSEMOVE, WM_PAINT,
    WM_RBUTTONUP, WM_SETCURSOR, WM_TIMER, WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
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
    main_hwnd: HWND,
    popup_hwnd: HWND,
    popup_visible: bool,
    popup_in_settings: bool,
    // Hit-test rectangles (in popup client coordinates)
    settings_rect: RECT,
    close_rect: RECT,
    refresh_rect: RECT,
    install_rect: RECT,
    chatgpt_link_rect: RECT,
    back_rect: RECT,
    setting_rects: [RECT; 5],
    notification_tracker: NotificationTracker,
    exe_dir: std::path::PathBuf,
    chart_data: Vec<f64>,
    chart_reset_lines: Vec<f64>,
    // Direct2D resources
    d2d: Option<D2DResources>,
    hovered_element: HoveredElement,
    mouse_tracking: bool,
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

    // Apply DWM attributes (rounded corners + dark mode)
    apply_dwm_rounded_corners(popup_hwnd);

    // Initialize D2D resources
    let d2d = match D2DResources::new() {
        Ok(d) => Some(d),
        Err(e) => {
            log::error!("Failed to init Direct2D: {e}");
            None
        }
    };

    // Initialize app state
    APP_STATE = Some(AppState {
        config_mgr,
        i18n,
        tray: None,
        usage: None,
        last_updated: String::new(),
        last_error: None,
        main_hwnd,
        popup_hwnd,
        popup_visible: false,
        popup_in_settings: false,
        settings_rect: RECT::default(),
        close_rect: RECT::default(),
        refresh_rect: RECT::default(),
        install_rect: RECT::default(),
        chatgpt_link_rect: RECT::default(),
        back_rect: RECT::default(),
        setting_rects: [RECT::default(); 5],
        notification_tracker: NotificationTracker::new(),
        exe_dir,
        chart_data: Vec::new(),
        chart_reset_lines: Vec::new(),
        d2d,
        hovered_element: HoveredElement::None,
        mouse_tracking: false,
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
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(main_wnd_proc),
        hInstance: hinstance.into(),
        hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };
    RegisterClassExW(&wc);
}

unsafe fn register_popup_class(hinstance: windows::Win32::Foundation::HMODULE) {
    let class_name = wide(POPUP_CLASS);
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
        lpfnWndProc: Some(popup_wnd_proc),
        hInstance: hinstance.into(),
        hIcon: LoadIconW(None, IDI_APPLICATION).unwrap_or_default(),
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
        lpszClassName: PCWSTR(class_name.as_ptr()),
        hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(6usize as *mut _),
        ..Default::default()
    };
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
            if wparam.0 == TIMER_POLL {
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
            let _hdc = BeginPaint(hwnd, &mut ps);

            let mut rect = RECT::default();
            let _ = windows::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut rect);

            if let Some(state) = APP_STATE.as_mut() {
                state.config_mgr.reload_if_changed();
                let theme_mode = ThemeMode::from_str(&state.config_mgr.config.theme);
                let resolved = resolve_theme(theme_mode);
                let colors = crate::ui::colors::ThemeColors::for_theme(resolved);

                // Apply DWM dark mode based on theme
                apply_dwm_dark_mode(hwnd, matches!(resolved, crate::theme::ResolvedTheme::Dark));

                if let Some(d2d) = state.d2d.as_mut() {
                    if d2d.ensure_render_target(hwnd).is_ok() {
                        // Clone the COM render target (cheap AddRef) to avoid
                        // borrow conflict with d2d being passed mutably to draw fns
                        if let Some(rt) = d2d.render_target.clone() {
                            rt.BeginDraw();
                            let bg = colorref_to_d2d(colors.background);
                            rt.Clear(Some(&bg as *const _));

                            let renderer = PopupRenderer::new(hwnd);

                            if state.popup_in_settings {
                                draw_settings_panel(
                                    d2d,
                                    &rect,
                                    &colors,
                                    &state.i18n,
                                    &state.config_mgr.config,
                                    &mut state.back_rect,
                                    &mut state.close_rect,
                                    &mut state.setting_rects,
                                    &state.hovered_element,
                                );
                            } else {
                                renderer.draw(
                                    d2d,
                                    &rect,
                                    &state.usage,
                                    &state.last_updated,
                                    state.config_mgr.config.show_chatgpt_section,
                                    state.config_mgr.config.compact_mode,
                                    &colors,
                                    &state.i18n,
                                    &state.chart_data,
                                    &state.chart_reset_lines,
                                    &state.last_error,
                                    &state.hovered_element,
                                    &mut state.settings_rect,
                                    &mut state.close_rect,
                                    &mut state.refresh_rect,
                                    &mut state.install_rect,
                                    &mut state.chatgpt_link_rect,
                                );
                            }

                            if rt.EndDraw(None, None).is_err() {
                                d2d.discard_render_target();
                            }
                        }
                    }
                }
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let pt = POINT { x, y };

            if let Some(state) = APP_STATE.as_mut() {
                if !state.mouse_tracking {
                    let mut tme = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    let _ = TrackMouseEvent(&mut tme);
                    state.mouse_tracking = true;
                }

                let new_hover = if state.popup_in_settings {
                    if crate::popup::point_in_rect(pt, state.back_rect) {
                        HoveredElement::BackButton
                    } else if crate::popup::point_in_rect(pt, state.close_rect) {
                        HoveredElement::CloseButton
                    } else {
                        let mut found = HoveredElement::None;
                        for (i, rect) in state.setting_rects.iter().enumerate() {
                            if crate::popup::point_in_rect(pt, *rect) {
                                found = HoveredElement::SettingRow(i);
                                break;
                            }
                        }
                        found
                    }
                } else if crate::popup::point_in_rect(pt, state.settings_rect) {
                    HoveredElement::SettingsButton
                } else if crate::popup::point_in_rect(pt, state.close_rect) {
                    HoveredElement::CloseButton
                } else if crate::popup::point_in_rect(pt, state.refresh_rect) {
                    HoveredElement::RefreshButton
                } else if crate::popup::point_in_rect(pt, state.install_rect) {
                    HoveredElement::InstallButton
                } else if crate::popup::point_in_rect(pt, state.chatgpt_link_rect) {
                    HoveredElement::ChatGptLink
                } else {
                    HoveredElement::None
                };

                if new_hover != state.hovered_element {
                    state.hovered_element = new_hover;
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            if let Some(state) = APP_STATE.as_mut() {
                state.mouse_tracking = false;
                if state.hovered_element != HoveredElement::None {
                    state.hovered_element = HoveredElement::None;
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, false);
                }
            }
            LRESULT(0)
        }
        WM_SETCURSOR => {
            if let Some(state) = APP_STATE.as_ref() {
                if !matches!(state.hovered_element, HoveredElement::None) {
                    let hand = LoadCursorW(None, IDC_HAND).unwrap_or_default();
                    SetCursor(hand);
                    return LRESULT(1);
                }
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
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
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.back_rect)
                {
                    state.popup_in_settings = false;
                    let renderer = PopupRenderer::new(hwnd);
                    let h = renderer.calculate_height(
                        &state.usage,
                        state.config_mgr.config.show_chatgpt_section,
                        state.config_mgr.config.compact_mode,
                    );
                    resize_popup(hwnd, h);
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[0])
                {
                    // Theme: cycle auto → dark → light → auto
                    let next = match state.config_mgr.config.theme.as_str() {
                        "auto" => "dark",
                        "dark" => "light",
                        _ => "auto",
                    };
                    state.config_mgr.config.theme = next.to_string();
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[1])
                {
                    // Language: cycle auto → en → uk → es → de → fr → auto
                    let next = match state.config_mgr.config.language.as_str() {
                        "auto" => "en",
                        "en" => "uk",
                        "uk" => "es",
                        "es" => "de",
                        "de" => "fr",
                        _ => "auto",
                    };
                    state.config_mgr.config.language = next.to_string();
                    state.i18n = I18n::from_config(&state.config_mgr.config.language);
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[2])
                {
                    // Compact mode: toggle
                    state.config_mgr.config.compact_mode = !state.config_mgr.config.compact_mode;
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[3])
                {
                    // Show ChatGPT section: toggle
                    state.config_mgr.config.show_chatgpt_section =
                        !state.config_mgr.config.show_chatgpt_section;
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[4])
                {
                    // Autostart: toggle + apply
                    state.config_mgr.config.autostart = !state.config_mgr.config.autostart;
                    let exe_path = state
                        .exe_dir
                        .join("claudemeter.exe")
                        .to_string_lossy()
                        .to_string();
                    let _ = autostart::set_autostart(state.config_mgr.config.autostart, &exe_path);
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if crate::popup::point_in_rect(pt, state.settings_rect) {
                    state.popup_in_settings = !state.popup_in_settings;
                    let h = if state.popup_in_settings {
                        settings_panel_height()
                    } else {
                        let renderer = PopupRenderer::new(hwnd);
                        renderer.calculate_height(
                            &state.usage,
                            state.config_mgr.config.show_chatgpt_section,
                            state.config_mgr.config.compact_mode,
                        )
                    };
                    resize_popup(hwnd, h);
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if crate::popup::point_in_rect(pt, state.refresh_rect) {
                    state.last_updated = "...".to_string();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                    trigger_poll(state.main_hwnd);
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
                state.hovered_element = HoveredElement::None;
                state.mouse_tracking = false;
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

/// Apply DWM rounded corners to popup window (Win11+, silently fails on Win10)
unsafe fn apply_dwm_rounded_corners(hwnd: HWND) {
    // DWMWA_WINDOW_CORNER_PREFERENCE = 33, DWMWCP_ROUND = 2
    let corner_pref: u32 = 2;
    let _ = DwmSetWindowAttribute(
        hwnd,
        windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(33),
        &corner_pref as *const u32 as *const _,
        std::mem::size_of::<u32>() as u32,
    );
}

/// Apply DWM dark/light mode to popup window (Win11+, silently fails on Win10)
unsafe fn apply_dwm_dark_mode(hwnd: HWND, is_dark: bool) {
    // DWMWA_USE_IMMERSIVE_DARK_MODE = 20
    let dark: u32 = if is_dark { 1 } else { 0 };
    let _ = DwmSetWindowAttribute(
        hwnd,
        windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(20),
        &dark as *const u32 as *const _,
        std::mem::size_of::<u32>() as u32,
    );
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

/// Calculate the settings panel height (header + rows + footer).
fn settings_panel_height() -> i32 {
    let header_h = 40;
    let row_h = 38;
    let num_rows = 5;
    let legend_h = 8 + 1 + 8 + 20 + (4 * 18); // sep + gap + title + 4 icon items
    let footer_h = 44;
    header_h + 8 + (num_rows * row_h) + legend_h + footer_h
}

unsafe fn resize_popup(popup_hwnd: HWND, h: i32) {
    let mut work_area = RECT::default();
    let _ = windows::Win32::UI::WindowsAndMessaging::SystemParametersInfoW(
        windows::Win32::UI::WindowsAndMessaging::SPI_GETWORKAREA,
        0,
        Some(&mut work_area as *mut RECT as *mut _),
        windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
    );
    let popup_w = crate::ui::render::POPUP_WIDTH;
    let x = work_area.right - popup_w - 10;
    let y = work_area.bottom - h - 10;

    let _ = windows::Win32::UI::WindowsAndMessaging::MoveWindow(
        popup_hwnd,
        x.max(0),
        y.max(0),
        popup_w,
        h,
        true,
    );

    // Resize D2D render target
    if let Some(state) = APP_STATE.as_mut() {
        if let Some(d2d) = state.d2d.as_mut() {
            d2d.resize(popup_w as u32, h as u32);
        }
    }
}

unsafe fn show_popup(_main_hwnd: HWND) {
    if let Some(state) = APP_STATE.as_mut() {
        state.config_mgr.reload_if_changed();

        let theme_mode = ThemeMode::from_str(&state.config_mgr.config.theme);
        let _resolved = resolve_theme(theme_mode);

        // Calculate height based on current mode
        let h = if state.popup_in_settings {
            settings_panel_height()
        } else {
            let renderer = PopupRenderer::new(state.popup_hwnd);
            renderer.calculate_height(
                &state.usage,
                state.config_mgr.config.show_chatgpt_section,
                state.config_mgr.config.compact_mode,
            )
        };

        resize_popup(state.popup_hwnd, h);
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
        append_menu_str(menu, IDM_ABOUT, "ClaudeMeter v1.0.0");
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
    let cred_info = match credentials::read_claude_token() {
        Ok(info) => info,
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

    match client.fetch_usage(&cred_info.access_token).await {
        Ok(mut usage) => {
            usage.subscription_type = cred_info.subscription_type;
            usage.rate_limit_tier = cred_info.rate_limit_tier;
            (Some(usage), None)
        }
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

            // Calculate 5-hour session reset lines for chart
            state.chart_reset_lines.clear();
            if let Some(fh) = u.five_hour.as_ref() {
                if let Some(secs) = fh.resets_at.as_deref().and_then(crate::i18n::seconds_until) {
                    let hours_until = secs as f64 / 3600.0;
                    let mut hours_ago = 5.0 - hours_until;
                    while hours_ago <= 24.0 {
                        if hours_ago > 0.0 {
                            state.chart_reset_lines.push(hours_ago);
                        }
                        hours_ago += 5.0;
                    }
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
                            .and_then(i18n::seconds_until)
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

        // Refresh popup if visible (resize + repaint)
        if state.popup_visible && !state.popup_in_settings {
            let renderer = PopupRenderer::new(state.popup_hwnd);
            let h = renderer.calculate_height(
                &state.usage,
                state.config_mgr.config.show_chatgpt_section,
                state.config_mgr.config.compact_mode,
            );
            resize_popup(state.popup_hwnd, h);
            let _ = windows::Win32::Graphics::Gdi::InvalidateRect(state.popup_hwnd, None, true);
        } else if state.popup_visible {
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
