#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(static_mut_refs)]
#![allow(clippy::too_many_arguments)]

mod config;
mod credentials;
mod db;
mod errors;
mod pace;
mod providers;

#[cfg(windows)]
mod autostart;
// Shared with the macOS menu bar agent, which only consumes the translation
// tables — the formatting helpers the Windows UI uses are unused there.
#[cfg_attr(not(windows), allow(dead_code))]
mod i18n;
#[cfg(windows)]
mod notifications;
#[cfg(windows)]
mod popup;
mod theme;
#[cfg(windows)]
mod tray;
#[cfg(windows)]
mod ui;
#[cfg(windows)]
mod updater;
#[cfg(windows)]
mod widget;
#[cfg(windows)]
mod windows_app;
#[cfg(windows)]
pub(crate) use windows_app::APP_STATE;

#[cfg(target_os = "macos")]
mod macos_app;

#[cfg(windows)]
fn main() {
    windows_app::run();
}

#[cfg(target_os = "macos")]
fn main() {
    macos_app::run();
}

#[cfg(not(any(windows, target_os = "macos")))]
fn main() {
    eprintln!("ClaudeMeter currently supports Windows and macOS.");
}
