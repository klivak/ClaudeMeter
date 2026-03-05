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
mod updater;
mod widget;

use crate::config::ConfigManager;
use crate::db::Database;
use crate::i18n::{format_duration, I18n};
use crate::notifications::NotificationTracker;
use crate::providers::claude::{ClaudeClient, UsageResponse};
use crate::theme::{resolve_theme, ThemeMode};
use crate::tray::{
    build_tooltip, TrayIcon, IDM_ABOUT, IDM_AUTOSTART, IDM_EXIT, IDM_EXPORT_CSV, IDM_OPEN_CHATGPT,
    IDM_OPEN_CLAUDE, IDM_OPEN_DASHBOARD, IDM_REFRESH, IDM_SETTINGS, WM_TRAY_ICON,
};
use crate::ui::colors::colorref_to_d2d;
use crate::ui::render::{draw_settings_panel, D2DResources, HoveredElement, PopupRenderer};
use chrono::{Local, Timelike};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};
use windows::Win32::UI::WindowsAndMessaging::LWA_ALPHA;
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, GetWindowLongW, LoadCursorW, LoadIconW, PeekMessageW, PostMessageW,
    PostQuitMessage, RegisterClassExW, SetCursor, SetForegroundWindow, SetLayeredWindowAttributes,
    SetWindowLongW, ShowWindow, TrackPopupMenu, TranslateMessage, CS_DROPSHADOW, CS_HREDRAW,
    CS_VREDRAW, GWL_EXSTYLE, HMENU, IDC_ARROW, IDC_HAND, IDI_APPLICATION, MF_CHECKED, MF_SEPARATOR,
    MF_STRING, MF_UNCHECKED, MSG, PM_REMOVE, SW_HIDE, SW_SHOWNOACTIVATE, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, TPM_RETURNCMD, WM_COMMAND, WM_DESTROY, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_PAINT, WM_RBUTTONUP, WM_SETCURSOR, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

// Language popup menu IDs
const IDM_LANG_AUTO: u32 = 5000;
const IDM_LANG_BASE: u32 = 5001;

// Theme popup menu IDs
const IDM_THEME_AUTO: u32 = 5100;
const IDM_THEME_DARK: u32 = 5101;
const IDM_THEME_LIGHT: u32 = 5102;

const WINDOW_CLASS: &str = "ClaudeMeterMain";
const POPUP_CLASS: &str = "ClaudeMeterPopup";
const TIMER_POLL: usize = 1;
const TIMER_ANIM: usize = 2;
const TIMER_BLINK: usize = 3;
const TIMER_FADE: usize = 4;
const TIMER_POLL_INTERVAL_MS: u32 = 120_000; // 2 minutes
const ANIM_INTERVAL_MS: u32 = 16; // ~60fps
const BLINK_INTERVAL_MS: u32 = 500;
const FADE_INTERVAL_MS: u32 = 16;
const IDLE_TIMEOUT_MS: u32 = 5 * 60 * 1000; // 5 minutes
const WM_POLL_RESULT: u32 = 0x0400 + 20; // WM_USER + 20
const WM_UPDATE_AVAILABLE: u32 = 0x0400 + 21; // WM_USER + 21

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
    status_link_rect: RECT,
    back_rect: RECT,
    setting_rects: [RECT; 9],
    notification_tracker: NotificationTracker,
    exe_dir: std::path::PathBuf,
    chart_data: Vec<f64>,
    chart_reset_lines: Vec<f64>,
    // Chart hit-testing
    chart_rect: RECT,
    chart_bar_count: usize,
    // Animation state for progress bars
    anim_targets: Vec<f64>,
    anim_current: Vec<f64>,
    anim_active: bool,
    // Fade-in animation
    fade_alpha: u8,
    // Tray icon blink on critical usage
    blink_active: bool,
    blink_visible: bool,
    // Retry backoff
    consecutive_failures: u32,
    // Last poll timestamp for auto-refresh
    last_poll_time: Option<std::time::Instant>,
    // Direct2D resources
    d2d: Option<D2DResources>,
    hovered_element: HoveredElement,
    mouse_tracking: bool,
    // Mini-widget window
    widget_hwnd: Option<HWND>,
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
    apply_mica_backdrop(popup_hwnd);

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
        status_link_rect: RECT::default(),
        back_rect: RECT::default(),
        setting_rects: [RECT::default(); 9],
        notification_tracker: NotificationTracker::new(),
        exe_dir,
        chart_data: Vec::new(),
        chart_reset_lines: Vec::new(),
        chart_rect: RECT::default(),
        chart_bar_count: 0,
        anim_targets: Vec::new(),
        anim_current: Vec::new(),
        anim_active: false,
        fade_alpha: 255,
        blink_active: false,
        blink_visible: true,
        consecutive_failures: 0,
        last_poll_time: None,
        d2d,
        hovered_element: HoveredElement::None,
        mouse_tracking: false,
        widget_hwnd: None,
    });

    // Create tray icon
    if let Some(state) = APP_STATE.as_mut() {
        match TrayIcon::new(main_hwnd) {
            Ok(tray) => state.tray = Some(tray),
            Err(e) => log::error!("Failed to create tray icon: {e}"),
        }

        // Startup notification (balloon tip from tray icon)
        if state.config_mgr.config.notifications.enabled {
            if let Some(tray) = &state.tray {
                tray.show_balloon(
                    "ClaudeMeter",
                    state
                        .i18n
                        .t("Running in system tray. Click the icon for details."),
                );
            }
        }
    }

    // Register and create mini-widget window
    widget::register_widget_class();
    if let Some(state) = APP_STATE.as_mut() {
        if let Some(w) = widget::create_widget_window() {
            state.widget_hwnd = Some(w);
            if state.config_mgr.config.show_widget {
                let _ = ShowWindow(w, SW_SHOWNOACTIVATE);
            }
        }
    }

    // Auto-update check (background thread)
    if APP_STATE
        .as_ref()
        .is_some_and(|s| s.config_mgr.config.check_updates)
    {
        let hwnd_raw = main_hwnd.0 as usize;
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            if let Some((tag, url)) = rt.block_on(updater::check_for_update()) {
                // Post update notification back to main thread
                let data = Box::new((tag, url));
                let hwnd = HWND(hwnd_raw as *mut _);
                let _ = PostMessageW(
                    hwnd,
                    WM_UPDATE_AVAILABLE,
                    WPARAM(Box::into_raw(data) as usize),
                    LPARAM(0),
                );
            }
        });
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
                // Skip polling when user is idle (screen locked, AFK)
                if !is_user_idle(IDLE_TIMEOUT_MS) {
                    trigger_poll(hwnd);
                }
            } else if wparam.0 == TIMER_BLINK {
                // Blink tray icon when critical usage
                if let Some(state) = APP_STATE.as_mut() {
                    state.blink_visible = !state.blink_visible;
                    if let Some(tray) = &mut state.tray {
                        let style = &state.config_mgr.config.tray_icon_style.clone();
                        if state.blink_visible {
                            let tooltip = build_tooltip(
                                &state.usage,
                                state.config_mgr.config.show_chatgpt_section,
                            );
                            tray.update(&state.usage, &tooltip, style);
                        } else {
                            tray.update(&None, "ClaudeMeter", style);
                        }
                    }
                }
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
        WM_UPDATE_AVAILABLE => {
            // Auto-update notification from background thread
            let data_ptr = wparam.0 as *mut (String, String);
            if !data_ptr.is_null() {
                let (tag, url) = *Box::from_raw(data_ptr);
                if let Some(state) = APP_STATE.as_ref() {
                    if let Some(tray) = &state.tray {
                        let title = state.i18n.t("Update available");
                        let body = format!(
                            "{} {}",
                            tag,
                            state.i18n.t("is available. Click to download.")
                        );
                        tray.show_balloon(title, &body);
                    }
                }
                log::info!("Update available: {} — {}", tag, url);
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
                let colors = crate::ui::colors::ThemeColors::for_theme(resolved)
                    .with_overrides(&state.config_mgr.config.custom_colors);

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
                                    &state.anim_current,
                                    &mut state.settings_rect,
                                    &mut state.close_rect,
                                    &mut state.refresh_rect,
                                    &mut state.install_rect,
                                    &mut state.chatgpt_link_rect,
                                    &mut state.status_link_rect,
                                    &mut state.chart_rect,
                                    &mut state.chart_bar_count,
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
        WM_TIMER => {
            if wparam.0 == TIMER_ANIM {
                if let Some(state) = APP_STATE.as_mut() {
                    if state.anim_active {
                        let mut all_done = true;
                        for (cur, &tgt) in
                            state.anim_current.iter_mut().zip(state.anim_targets.iter())
                        {
                            let diff = tgt - *cur;
                            if diff.abs() < 0.5 {
                                *cur = tgt;
                            } else {
                                *cur += diff * 0.15;
                                all_done = false;
                            }
                        }
                        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, false);
                        if all_done {
                            state.anim_active = false;
                            let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(
                                hwnd, TIMER_ANIM,
                            );
                        }
                    }
                }
            } else if wparam.0 == TIMER_FADE {
                if let Some(state) = APP_STATE.as_mut() {
                    let new_alpha = (state.fade_alpha as u16 + 30).min(255) as u8;
                    state.fade_alpha = new_alpha;
                    let _ = SetLayeredWindowAttributes(
                        hwnd,
                        windows::Win32::Foundation::COLORREF(0),
                        new_alpha,
                        LWA_ALPHA,
                    );
                    if new_alpha == 255 {
                        // Fade complete — remove WS_EX_LAYERED for normal rendering
                        let ex = GetWindowLongW(hwnd, GWL_EXSTYLE);
                        SetWindowLongW(hwnd, GWL_EXSTYLE, ex & !(WS_EX_LAYERED.0 as i32));
                        let _ =
                            windows::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_FADE);
                    }
                }
            }
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
                } else if crate::popup::point_in_rect(pt, state.status_link_rect) {
                    HoveredElement::StatusLink
                } else if crate::popup::point_in_rect(pt, state.chart_rect)
                    && state.chart_bar_count > 0
                {
                    let chart_w = state.chart_rect.right - state.chart_rect.left;
                    let rel_x = pt.x - state.chart_rect.left;
                    let bar_idx = (rel_x as usize * state.chart_bar_count / chart_w as usize)
                        .min(state.chart_bar_count - 1);
                    HoveredElement::ChartBar(bar_idx)
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
                if !matches!(
                    state.hovered_element,
                    HoveredElement::None | HoveredElement::ChartBar(_)
                ) {
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
                    hide_popup(state);
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
                    // Theme: show popup menu
                    show_theme_popup(hwnd, pt, state);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[1])
                {
                    // Language: show popup menu with all languages
                    show_language_popup(hwnd, pt, state);
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
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[5])
                {
                    // Show widget: toggle
                    state.config_mgr.config.show_widget = !state.config_mgr.config.show_widget;
                    state.config_mgr.save();
                    // Toggle widget visibility
                    if state.config_mgr.config.show_widget {
                        if let Some(w) = state.widget_hwnd {
                            let _ = ShowWindow(w, SW_SHOWNOACTIVATE);
                        }
                    } else if let Some(w) = state.widget_hwnd {
                        let _ = ShowWindow(w, SW_HIDE);
                    }
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[6])
                {
                    // Check for updates: toggle
                    state.config_mgr.config.check_updates = !state.config_mgr.config.check_updates;
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[7])
                {
                    // Accessibility patterns: toggle
                    state.config_mgr.config.accessibility_patterns =
                        !state.config_mgr.config.accessibility_patterns;
                    state.config_mgr.save();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
                } else if state.popup_in_settings
                    && crate::popup::point_in_rect(pt, state.setting_rects[8])
                {
                    // Icon style: cycle number → ring → bar → number
                    let next = match state.config_mgr.config.tray_icon_style.as_str() {
                        "number" => "ring",
                        "ring" => "bar",
                        _ => "number",
                    };
                    state.config_mgr.config.tray_icon_style = next.to_string();
                    state.config_mgr.save();
                    // Immediately update tray icon with new style
                    let tooltip =
                        build_tooltip(&state.usage, state.config_mgr.config.show_chatgpt_section);
                    if let Some(tray) = &mut state.tray {
                        tray.update(&state.usage, &tooltip, next);
                    }
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
                } else if crate::popup::point_in_rect(pt, state.status_link_rect) {
                    let _ = open::that("https://status.claude.com/");
                }
            }
            LRESULT(0)
        }
        WM_KEYDOWN => {
            let vk = wparam.0 as u16;
            if vk == 0x1B {
                // VK_ESCAPE — close popup
                if let Some(state) = APP_STATE.as_mut() {
                    hide_popup(state);
                }
            } else if vk == 0x74 {
                // VK_F5 — refresh
                if let Some(state) = APP_STATE.as_mut() {
                    state.last_updated = "...".to_string();
                    let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, false);
                    trigger_poll(state.main_hwnd);
                }
            }
            LRESULT(0)
        }
        // Close popup when clicking outside (WM_KILLFOCUS)
        WM_KILLFOCUS => {
            if let Some(state) = APP_STATE.as_mut() {
                if state.popup_visible {
                    hide_popup(state);
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

/// Apply Mica backdrop to popup window (Win11 22H2+, silently fails on older)
unsafe fn apply_mica_backdrop(hwnd: HWND) {
    // DWMWA_SYSTEMBACKDROP_TYPE = 38, value 2 = Mica
    let backdrop_type: u32 = 2;
    let _ = DwmSetWindowAttribute(
        hwnd,
        windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(38),
        &backdrop_type as *const u32 as *const _,
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

/// Hide the popup and release D2D resources to reclaim memory.
unsafe fn hide_popup(state: &mut AppState) {
    state.popup_visible = false;
    state.hovered_element = HoveredElement::None;
    state.mouse_tracking = false;
    state.anim_active = false;
    let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(state.popup_hwnd, TIMER_ANIM);
    let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(state.popup_hwnd, TIMER_FADE);
    // Remove WS_EX_LAYERED if still set from fade
    let ex = GetWindowLongW(state.popup_hwnd, GWL_EXSTYLE);
    if ex & WS_EX_LAYERED.0 as i32 != 0 {
        SetWindowLongW(
            state.popup_hwnd,
            GWL_EXSTYLE,
            ex & !(WS_EX_LAYERED.0 as i32),
        );
    }
    let _ = windows::Win32::UI::WindowsAndMessaging::ShowWindow(
        state.popup_hwnd,
        windows::Win32::UI::WindowsAndMessaging::SW_HIDE,
    );
    if let Some(d2d) = state.d2d.as_mut() {
        d2d.release();
    }
    trim_working_set();
}

/// Ask Windows to trim the process working set so freed memory is returned to the OS.
fn trim_working_set() {
    extern "system" {
        fn SetProcessWorkingSetSize(
            hProcess: *mut core::ffi::c_void,
            dwMinimumWorkingSetSize: usize,
            dwMaximumWorkingSetSize: usize,
        ) -> i32;
    }
    unsafe {
        let process = windows::Win32::System::Threading::GetCurrentProcess();
        SetProcessWorkingSetSize(process.0, usize::MAX, usize::MAX);
    }
}

unsafe fn toggle_popup(main_hwnd: HWND) {
    if let Some(state) = APP_STATE.as_mut() {
        if state.popup_visible {
            hide_popup(state);
        } else {
            show_popup(main_hwnd);
        }
    }
}

/// Calculate the settings panel height (header + rows + footer).
fn settings_panel_height() -> i32 {
    let header_h = 40;
    let row_h = 38;
    let num_rows = 9;
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

        // Fade-in: add WS_EX_LAYERED and start transparent
        let ex = GetWindowLongW(state.popup_hwnd, GWL_EXSTYLE);
        SetWindowLongW(state.popup_hwnd, GWL_EXSTYLE, ex | WS_EX_LAYERED.0 as i32);
        state.fade_alpha = 0;
        let _ = SetLayeredWindowAttributes(
            state.popup_hwnd,
            windows::Win32::Foundation::COLORREF(0),
            0,
            LWA_ALPHA,
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

        // Start fade-in timer
        windows::Win32::UI::WindowsAndMessaging::SetTimer(
            state.popup_hwnd,
            TIMER_FADE,
            FADE_INTERVAL_MS,
            None,
        );

        // Stop tray icon blink when user opens popup
        if state.blink_active {
            state.blink_active = false;
            state.blink_visible = true;
            let _ =
                windows::Win32::UI::WindowsAndMessaging::KillTimer(state.main_hwnd, TIMER_BLINK);
            // Restore normal icon
            let tooltip = build_tooltip(&state.usage, state.config_mgr.config.show_chatgpt_section);
            if let Some(tray) = &mut state.tray {
                let style = &state.config_mgr.config.tray_icon_style.clone();
                tray.update(&state.usage, &tooltip, style);
            }
        }

        // Start animation if we have targets
        if !state.anim_targets.is_empty() {
            state.anim_current = vec![0.0; state.anim_targets.len()];
            state.anim_active = true;
            windows::Win32::UI::WindowsAndMessaging::SetTimer(
                state.popup_hwnd,
                TIMER_ANIM,
                ANIM_INTERVAL_MS,
                None,
            );
        }

        // Auto-refresh if data is stale (older than 60s)
        let stale = state
            .last_poll_time
            .map(|t| t.elapsed().as_secs() > 60)
            .unwrap_or(true);
        if stale {
            trigger_poll(state.main_hwnd);
        }
    }
}

unsafe fn show_theme_popup(hwnd: HWND, client_pt: POINT, state: &mut AppState) {
    let menu = CreatePopupMenu().unwrap();
    let current = &state.config_mgr.config.theme;

    let themes = [
        (IDM_THEME_AUTO, "auto", "Auto"),
        (IDM_THEME_DARK, "dark", "Dark"),
        (IDM_THEME_LIGHT, "light", "Light"),
    ];

    for (id, value, i18n_key) in &themes {
        let flag = if current == *value {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING | MF_UNCHECKED
        };
        let label = wide(state.i18n.t(i18n_key));
        let _ = AppendMenuW(menu, flag, *id as usize, PCWSTR(label.as_ptr()));
    }

    let mut screen_pt = client_pt;
    let _ = ClientToScreen(hwnd, &mut screen_pt);

    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_RETURNCMD,
        screen_pt.x,
        screen_pt.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);

    if cmd.as_bool() {
        let new_theme = match cmd.0 as u32 {
            IDM_THEME_DARK => "dark",
            IDM_THEME_LIGHT => "light",
            _ => "auto",
        };
        state.config_mgr.config.theme = new_theme.to_string();
        state.config_mgr.save();
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
    }
}

unsafe fn show_language_popup(hwnd: HWND, client_pt: POINT, state: &mut AppState) {
    let menu = CreatePopupMenu().unwrap();
    let current_lang = &state.config_mgr.config.language;

    // "Auto (detected)" option
    let auto_flag = if current_lang == "auto" {
        MF_STRING | MF_CHECKED
    } else {
        MF_STRING | MF_UNCHECKED
    };
    let auto_label = state.i18n.t("Auto (English)");
    let auto_wide = wide(auto_label);
    let _ = AppendMenuW(
        menu,
        auto_flag,
        IDM_LANG_AUTO as usize,
        PCWSTR(auto_wide.as_ptr()),
    );
    append_menu_sep(menu);

    // All locales sorted alphabetically by display name
    for (i, locale) in crate::i18n::Locale::all().iter().enumerate() {
        let flag = if current_lang == locale.as_str() {
            MF_STRING | MF_CHECKED
        } else {
            MF_STRING | MF_UNCHECKED
        };
        let label_wide = wide(locale.display_name());
        let _ = AppendMenuW(
            menu,
            flag,
            (IDM_LANG_BASE + i as u32) as usize,
            PCWSTR(label_wide.as_ptr()),
        );
    }

    // Convert client coords to screen coords for TrackPopupMenu
    let mut screen_pt = client_pt;
    let _ = ClientToScreen(hwnd, &mut screen_pt);

    let _ = SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(
        menu,
        TPM_LEFTALIGN | TPM_RETURNCMD,
        screen_pt.x,
        screen_pt.y,
        0,
        hwnd,
        None,
    );
    let _ = DestroyMenu(menu);

    if cmd.as_bool() {
        let cmd_id = cmd.0 as u32;
        let new_lang = if cmd_id == IDM_LANG_AUTO {
            "auto".to_string()
        } else {
            let idx = (cmd_id - IDM_LANG_BASE) as usize;
            let locales = crate::i18n::Locale::all();
            if idx < locales.len() {
                locales[idx].as_str().to_string()
            } else {
                return;
            }
        };
        state.config_mgr.config.language = new_lang;
        state.i18n = I18n::from_config(&state.config_mgr.config.language);
        state.config_mgr.save();
        let _ = windows::Win32::Graphics::Gdi::InvalidateRect(hwnd, None, true);
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
        append_menu_str(menu, IDM_EXPORT_CSV, state.i18n.t("Export History (CSV)"));
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
        let about_label = format!("ClaudeMeter v{}", env!("CARGO_PKG_VERSION"));
        append_menu_str(menu, IDM_ABOUT, &about_label);
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
            let _ = open::that("https://claude.ai/settings/usage");
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
        IDM_EXPORT_CSV => {
            if let Some(state) = APP_STATE.as_ref() {
                let csv_path = state.exe_dir.join("claudemeter_history.csv");
                match Database::open(&state.exe_dir) {
                    Ok(db) => match db.export_csv(&csv_path) {
                        Ok(count) => {
                            let msg =
                                format!("{} rows exported to:\n{}", count, csv_path.display());
                            let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                                hwnd,
                                windows::core::PCWSTR(wide(&msg).as_ptr()),
                                windows::core::PCWSTR(wide("Export CSV").as_ptr()),
                                windows::Win32::UI::WindowsAndMessaging::MB_ICONINFORMATION
                                    | windows::Win32::UI::WindowsAndMessaging::MB_OK,
                            );
                        }
                        Err(e) => {
                            let msg = format!("Export failed: {e}");
                            let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                                hwnd,
                                windows::core::PCWSTR(wide(&msg).as_ptr()),
                                windows::core::PCWSTR(wide("Export CSV").as_ptr()),
                                windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR
                                    | windows::Win32::UI::WindowsAndMessaging::MB_OK,
                            );
                        }
                    },
                    Err(e) => {
                        let msg = format!("Could not open database: {e}");
                        let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                            hwnd,
                            windows::core::PCWSTR(wide(&msg).as_ptr()),
                            windows::core::PCWSTR(wide("Export CSV").as_ptr()),
                            windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR
                                | windows::Win32::UI::WindowsAndMessaging::MB_OK,
                        );
                    }
                }
            }
        }
        IDM_ABOUT => {
            // Show simple about message
            let _ = windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                hwnd,
                windows::core::PCWSTR(wide(&format!("ClaudeMeter v{}\nby klivak\nhttps://github.com/klivak/claudemeter\n\nMIT License", env!("CARGO_PKG_VERSION"))).as_ptr()),
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

unsafe fn on_poll_result(hwnd: HWND, usage: Option<UsageResponse>, error: Option<String>) {
    if let Some(state) = APP_STATE.as_mut() {
        state.last_poll_time = Some(std::time::Instant::now());

        if let Some(u) = &usage {
            state.consecutive_failures = 0;
            state.last_updated = Local::now().format("%H:%M:%S").to_string();

            // Store to DB (best effort)
            if let Ok(db) = Database::open(&state.exe_dir) {
                for (key, metric) in u.all_metrics() {
                    // Skip five_hour when no active session (resets_at is None)
                    if key == "five_hour" && metric.resets_at.is_none() {
                        continue;
                    }
                    let _ = db.insert(
                        "claude",
                        &key,
                        metric.utilization,
                        metric.resets_at.as_deref(),
                    );
                }
                // Load chart data (fixed 48-slot array, oldest first)
                if let Ok(slots) = db.query_24h_chart() {
                    state.chart_data = slots;
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

            // Check notifications (skip during quiet hours)
            let thresholds = state.config_mgr.config.notifications.thresholds.clone();
            let in_quiet = is_in_quiet_hours(&state.config_mgr.config.quiet_hours);
            if state.config_mgr.config.notifications.enabled && !in_quiet {
                for (key, metric) in u.all_metrics() {
                    let fired =
                        state
                            .notification_tracker
                            .check(&key, metric.utilization, &thresholds);
                    for threshold in fired {
                        let metric_name = providers::claude::format_metric_name(&key);
                        let reset_duration = metric
                            .resets_at
                            .as_deref()
                            .and_then(i18n::seconds_until)
                            .map(format_duration);
                        let reset_target = metric
                            .resets_at
                            .as_deref()
                            .and_then(i18n::format_reset_target);
                        let reset_info = match (&reset_duration, &reset_target) {
                            (Some(dur), Some(tgt)) => {
                                format!("\n{} {} {}", state.i18n.t("resets in"), dur, tgt)
                            }
                            (Some(dur), None) => {
                                format!("\n{} {}", state.i18n.t("resets in"), dur)
                            }
                            _ => String::new(),
                        };

                        let (title, body) = if threshold >= 90 {
                            (
                                format!("ClaudeMeter — {}", state.i18n.t("Usage Critical")),
                                format!(
                                    "{}: {:.0}% ({} {}%){}",
                                    metric_name,
                                    metric.utilization,
                                    state.i18n.t("exceeded"),
                                    threshold,
                                    reset_info
                                ),
                            )
                        } else {
                            (
                                format!("ClaudeMeter — {}", state.i18n.t("Usage Alert")),
                                format!(
                                    "{}: {:.0}% ({} {}%){}",
                                    metric_name,
                                    metric.utilization,
                                    state.i18n.t("exceeded"),
                                    threshold,
                                    reset_info
                                ),
                            )
                        };
                        if let Some(tray) = &state.tray {
                            tray.show_balloon(&title, &body);
                        }

                        // Play notification sound
                        if state.config_mgr.config.notifications.sound {
                            play_notification_sound(threshold >= 90);
                        }

                        // Start tray icon blink for critical usage
                        if threshold >= 90 && !state.blink_active {
                            state.blink_active = true;
                            state.blink_visible = true;
                            windows::Win32::UI::WindowsAndMessaging::SetTimer(
                                state.main_hwnd,
                                TIMER_BLINK,
                                BLINK_INTERVAL_MS,
                                None,
                            );
                        }
                    }
                }
            }
        } else {
            // Poll failed — track for backoff
            state.consecutive_failures += 1;

            // Start error blink if there's no cached data to show
            if state.usage.is_none() && !state.blink_active {
                state.blink_active = true;
                state.blink_visible = true;
                windows::Win32::UI::WindowsAndMessaging::SetTimer(
                    state.main_hwnd,
                    TIMER_BLINK,
                    BLINK_INTERVAL_MS,
                    None,
                );
            }
        }

        // Only overwrite usage when poll succeeded; keep previous data on failure
        if usage.is_some() {
            state.usage = usage;
        }
        state.last_error = error;

        // Adjust polling interval with exponential backoff on failures
        let base = state.config_mgr.config.polling_interval_clamped() as u32 * 1000;
        let interval = if state.consecutive_failures > 0 {
            let multiplier = 2u32.pow(state.consecutive_failures.min(3));
            (base * multiplier).min(600_000) // cap at 10 minutes
        } else {
            base
        };
        let _ = windows::Win32::UI::WindowsAndMessaging::KillTimer(hwnd, TIMER_POLL);
        windows::Win32::UI::WindowsAndMessaging::SetTimer(hwnd, TIMER_POLL, interval, None);

        // Start progress bar animation
        if let Some(u) = &state.usage {
            let targets: Vec<f64> = u.all_metrics().iter().map(|(_, m)| m.utilization).collect();
            // Initialize current values at 0 if sizes differ (new data shape)
            if state.anim_current.len() != targets.len() {
                state.anim_current = vec![0.0; targets.len()];
            }
            state.anim_targets = targets;
            state.anim_active = true;
            if state.popup_visible {
                windows::Win32::UI::WindowsAndMessaging::SetTimer(
                    state.popup_hwnd,
                    TIMER_ANIM,
                    ANIM_INTERVAL_MS,
                    None,
                );
            }
        }

        // Update tray
        let tooltip = build_tooltip(&state.usage, state.config_mgr.config.show_chatgpt_section);
        if let Some(tray) = &mut state.tray {
            let style = &state.config_mgr.config.tray_icon_style.clone();
            tray.update(&state.usage, &tooltip, style);
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

        // Refresh mini-widget
        if let Some(w) = state.widget_hwnd {
            widget::invalidate_widget(w);
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

/// Check if current local time falls within the quiet hours window.
fn is_in_quiet_hours(qh: &crate::config::QuietHoursConfig) -> bool {
    if !qh.enabled {
        return false;
    }
    let parse_hm = |s: &str| -> Option<(u32, u32)> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 2 {
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
        } else {
            None
        }
    };
    let (sh, sm) = match parse_hm(&qh.start) {
        Some(v) => v,
        None => return false,
    };
    let (eh, em) = match parse_hm(&qh.end) {
        Some(v) => v,
        None => return false,
    };
    let now = chrono::Local::now();
    let now_mins = now.hour() * 60 + now.minute();
    let start_mins = sh * 60 + sm;
    let end_mins = eh * 60 + em;
    if start_mins <= end_mins {
        // Same-day range (e.g., 08:00 → 18:00)
        now_mins >= start_mins && now_mins < end_mins
    } else {
        // Overnight range (e.g., 22:00 → 08:00)
        now_mins >= start_mins || now_mins < end_mins
    }
}

/// Check if the user has been idle for more than `timeout_ms` milliseconds.
fn is_user_idle(timeout_ms: u32) -> bool {
    #[repr(C)]
    #[allow(clippy::upper_case_acronyms)]
    struct LASTINPUTINFO {
        cb_size: u32,
        dw_time: u32,
    }
    extern "system" {
        fn GetLastInputInfo(plii: *mut LASTINPUTINFO) -> i32;
        fn GetTickCount() -> u32;
    }
    unsafe {
        let mut lii = LASTINPUTINFO {
            cb_size: 8,
            dw_time: 0,
        };
        if GetLastInputInfo(&mut lii) != 0 {
            let idle = GetTickCount().wrapping_sub(lii.dw_time);
            idle > timeout_ms
        } else {
            false
        }
    }
}

/// Play a system notification sound.
fn play_notification_sound(critical: bool) {
    extern "system" {
        fn MessageBeep(uType: u32) -> i32;
    }
    unsafe {
        if critical {
            MessageBeep(0x10); // MB_ICONHAND — critical/error sound
        } else {
            MessageBeep(0x30); // MB_ICONEXCLAMATION — warning sound
        }
    }
}
