use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub thresholds: Vec<u8>,
    pub sound: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            thresholds: vec![50, 75, 90],
            sound: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHoursConfig {
    pub enabled: bool,
    pub start: String,
    pub end: String,
}

impl Default for QuietHoursConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            start: "22:00".to_string(),
            end: "08:00".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomColors {
    pub background: Option<String>,
    pub surface: Option<String>,
    pub text_primary: Option<String>,
    pub text_secondary: Option<String>,
    pub progress_bg: Option<String>,
    pub green: Option<String>,
    pub yellow: Option<String>,
    pub red: Option<String>,
    pub accent: Option<String>,
    pub separator: Option<String>,
    pub hover: Option<String>,
    pub border: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: String,
    pub polling_interval_seconds: u64,
    pub notifications: NotificationConfig,
    pub autostart: bool,
    pub compact_mode: bool,
    pub theme: String,
    pub language: String,
    pub show_chatgpt_section: bool,
    pub chatgpt_usage_url: String,
    pub claude_install_url: String,
    #[serde(default)]
    pub show_widget: bool,
    #[serde(default = "default_true")]
    pub check_updates: bool,
    #[serde(default)]
    pub accessibility_patterns: bool,
    #[serde(default = "default_icon_style")]
    pub tray_icon_style: String,
    #[serde(default)]
    pub custom_colors: CustomColors,
    #[serde(default)]
    pub quiet_hours: QuietHoursConfig,
}

fn default_icon_style() -> String {
    "number".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            polling_interval_seconds: 120,
            notifications: NotificationConfig::default(),
            autostart: false,
            compact_mode: false,
            theme: "auto".to_string(),
            language: "auto".to_string(),
            show_chatgpt_section: false,
            chatgpt_usage_url: "https://chatgpt.com/codex/settings/usage".to_string(),
            claude_install_url: "https://claude.ai/download".to_string(),
            show_widget: false,
            check_updates: true,
            accessibility_patterns: false,
            tray_icon_style: "number".to_string(),
            custom_colors: CustomColors::default(),
            quiet_hours: QuietHoursConfig::default(),
        }
    }
}

impl Config {
    pub fn polling_interval_clamped(&self) -> u64 {
        self.polling_interval_seconds.clamp(30, 600)
    }

    /// Validate and fix config values to safe ranges.
    pub fn validate(&mut self) {
        self.polling_interval_seconds = self.polling_interval_seconds.clamp(30, 600);

        // Thresholds: keep only 1..=100, remove duplicates, sort
        self.notifications
            .thresholds
            .retain(|&t| (1..=100).contains(&t));
        self.notifications.thresholds.sort();
        self.notifications.thresholds.dedup();
        if self.notifications.thresholds.is_empty() {
            self.notifications.thresholds = vec![50, 75, 90];
        }

        // Validate theme
        if !["auto", "dark", "light"].contains(&self.theme.as_str()) {
            self.theme = "auto".to_string();
        }

        // Validate tray icon style
        if !["number", "ring", "bar"].contains(&self.tray_icon_style.as_str()) {
            self.tray_icon_style = "number".to_string();
        }

        // Validate language
        if ![
            "auto", "en", "uk", "es", "de", "fr", "pt", "ja", "ko", "zh", "it", "hi", "tr", "nl",
            "pl", "vi", "ru", "th", "id", "sv", "cs", "ar", "ro", "da", "fi", "hu",
        ]
        .contains(&self.language.as_str())
        {
            self.language = "auto".to_string();
        }
    }
}

pub struct ConfigManager {
    path: PathBuf,
    last_modified: Option<SystemTime>,
    pub config: Config,
}

impl ConfigManager {
    pub fn new(exe_dir: &Path) -> Self {
        let path = exe_dir.join("config.json");
        let mut mgr = Self {
            path,
            last_modified: None,
            config: Config::default(),
        };
        mgr.load();
        mgr
    }

    fn load(&mut self) {
        if self.path.exists() {
            match fs::read_to_string(&self.path) {
                Ok(content) => match serde_json::from_str::<Config>(&content) {
                    Ok(mut cfg) => {
                        cfg.validate();
                        self.config = cfg;
                        self.last_modified = fs::metadata(&self.path)
                            .ok()
                            .and_then(|m| m.modified().ok());
                    }
                    Err(e) => {
                        log::warn!("Failed to parse config.json: {e}. Using defaults.");
                    }
                },
                Err(e) => {
                    log::warn!("Failed to read config.json: {e}. Using defaults.");
                }
            }
        } else {
            self.save();
        }
    }

    /// Check if config file changed on disk and reload if so.
    pub fn reload_if_changed(&mut self) {
        let mtime = fs::metadata(&self.path)
            .ok()
            .and_then(|m| m.modified().ok());
        if mtime != self.last_modified {
            log::debug!("Config file changed, reloading.");
            self.load();
        }
    }

    pub fn save(&self) {
        match serde_json::to_string_pretty(&self.config) {
            Ok(content) => {
                if let Err(e) = fs::write(&self.path, content) {
                    log::error!("Failed to write config.json: {e}");
                }
            }
            Err(e) => log::error!("Failed to serialize config: {e}"),
        }
    }
}
