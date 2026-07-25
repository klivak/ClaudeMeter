use crate::config::{Config, ConfigManager};
use crate::credentials::read_claude_token;
use crate::db::Database;
use crate::i18n::I18n;
use crate::providers::claude::{ClaudeClient, MetricFilter, UsageResponse};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetricEntry {
    /// Raw API key, e.g. "five_hour" — used by the UI to pick the session metric.
    key: String,
    /// Human-readable name, e.g. "Weekly (7-day)"
    name: String,
    percent: u32,
    resets_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MacStatus {
    state: String,
    title: String,
    detail: String,
    plan: Option<String>,
    percent: Option<u32>,
    /// Per-limit breakdown (5-hour, weekly, Sonnet, Opus, ...). Empty until first fetch.
    #[serde(default)]
    metrics: Vec<MetricEntry>,
    /// Downgrade comparison, e.g. "On Max 5x: weekly ~96%, session ~124%".
    #[serde(default)]
    tier_note: Option<String>,
    last_api_update: Option<String>,
    data_age_seconds: Option<u64>,
    error: Option<String>,
    /// 24h usage history for the five_hour metric, 48 buckets oldest-first
    /// (index 0 = 24h ago, index 47 = now). Each value is a 0-100 percent.
    /// Mirrors the Windows popup's "Usage History (24h)" chart.
    #[serde(default)]
    chart: Vec<u32>,
    /// Past 5-hour session reset points, as "hours ago" (0-24). Drawn as dashed
    /// vertical lines on the chart, matching the Windows popup.
    #[serde(default)]
    chart_resets: Vec<f64>,
    /// Localized menu strings for the Swift UI, keyed by a stable identifier.
    /// The UI keeps English literals as a fallback, so an older app bundle with
    /// a newer agent (or vice versa) still renders.
    #[serde(default)]
    labels: HashMap<String, String>,
    /// Highest alert threshold already announced per metric key. Persisted so a
    /// one-shot `--refresh` process does not re-fire notifications the running
    /// agent has already sent.
    #[serde(default)]
    alerted: HashMap<String, u8>,
}

impl MacStatus {
    fn refreshing() -> Self {
        Self {
            state: "refreshing".to_string(),
            title: "Refreshing...".to_string(),
            detail: "Requesting fresh Claude usage data".to_string(),
            plan: None,
            percent: None,
            metrics: Vec::new(),
            tier_note: None,
            last_api_update: None,
            data_age_seconds: None,
            error: None,
            chart: Vec::new(),
            chart_resets: Vec::new(),
            labels: HashMap::new(),
            alerted: HashMap::new(),
        }
    }

    fn error(message: String) -> Self {
        Self {
            state: "error".to_string(),
            title: "API error".to_string(),
            detail: message.clone(),
            plan: None,
            percent: None,
            metrics: Vec::new(),
            tier_note: None,
            last_api_update: None,
            data_age_seconds: None,
            error: Some(message),
            chart: Vec::new(),
            chart_resets: Vec::new(),
            labels: HashMap::new(),
            alerted: HashMap::new(),
        }
    }
}

pub fn run() {
    env_logger::init();

    let exe_dir = app_data_dir();
    if let Err(e) = std::fs::create_dir_all(&exe_dir) {
        log::warn!("Failed to create app data directory {:?}: {e}", exe_dir);
    }

    let args: Vec<String> = std::env::args().collect();
    let once = args.iter().any(|arg| arg == "--once" || arg == "--refresh");
    let status_only = args.iter().any(|arg| arg == "--status");

    if status_only {
        print_status(&exe_dir);
        return;
    }

    // One-shot commands the menu bar app shells out to. Each prints a single
    // line so the Swift side can show a result without parsing.
    if let Some(path) = flag_value(&args, "--export-csv") {
        export_history(&exe_dir, Path::new(&path), true);
        return;
    }
    if let Some(path) = flag_value(&args, "--export-json") {
        export_history(&exe_dir, Path::new(&path), false);
        return;
    }
    if let Some(assignment) = flag_value(&args, "--set") {
        apply_setting(&exe_dir, &assignment);
        return;
    }

    let config_mgr = ConfigManager::new(&exe_dir);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    rt.block_on(async move {
        let config = config_mgr.config.clone();

        let client = match ClaudeClient::new() {
            Ok(client) => client,
            Err(e) => {
                let message = format!("Failed to create Claude client: {e}");
                append_log(&exe_dir, &message);
                let i18n = I18n::from_config(&config.language);
                write_error(&exe_dir, message, Vec::new(), &config, &i18n);
                return;
            }
        };

        if once {
            poll_once(&exe_dir, &client, &config).await;
            return;
        }

        if config.show_startup_notification {
            let i18n = I18n::from_config(&config.language);
            notify(
                "ClaudeMeter",
                i18n.t("Running in system tray. Click the icon for details."),
                config.notifications.sound,
            );
        }

        loop {
            poll_once(&exe_dir, &client, &config).await;
            let interval = config.polling_interval_seconds.max(60);
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}

/// Value following a CLI flag, e.g. `--set language=uk` or `--set language uk`.
fn flag_value(args: &[String], flag: &str) -> Option<String> {
    let position = args.iter().position(|arg| arg == flag);
    if let Some(index) = position {
        return args.get(index + 1).cloned();
    }
    args.iter()
        .find_map(|arg| arg.strip_prefix(&format!("{flag}=")).map(str::to_string))
}

/// Export the usage history the menu bar app collected. Prints the destination
/// on success so the Swift side can report it.
fn export_history(exe_dir: &Path, path: &Path, csv: bool) {
    let result = Database::open(exe_dir).and_then(|db| {
        if csv {
            db.export_csv(path)
        } else {
            db.export_json(path)
        }
    });
    match result {
        Ok(rows) => println!("Exported {rows} rows to {}", path.display()),
        Err(e) => println!("Export failed: {e}"),
    }
}

/// Apply a single `key=value` config change, validated and saved through the
/// same `ConfigManager` the app uses, so the menu never writes invalid JSON.
fn apply_setting(exe_dir: &Path, assignment: &str) {
    let Some((key, value)) = assignment.split_once('=') else {
        println!("Expected key=value, got: {assignment}");
        return;
    };
    let mut mgr = ConfigManager::new(exe_dir);
    let as_bool = || matches!(value, "true" | "1" | "on" | "yes");

    match key {
        "show_chatgpt_section" => mgr.config.show_chatgpt_section = as_bool(),
        "show_model_limits" => mgr.config.show_model_limits = as_bool(),
        "show_extra_usage" => mgr.config.show_extra_usage = as_bool(),
        "show_startup_notification" => mgr.config.show_startup_notification = as_bool(),
        "token_expiry_warning" => mgr.config.token_expiry_warning = as_bool(),
        "notifications.enabled" => mgr.config.notifications.enabled = as_bool(),
        "notifications.sound" => mgr.config.notifications.sound = as_bool(),
        "notifications.thresholds" => {
            mgr.config.notifications.thresholds =
                crate::config::cycle_notification_thresholds(&mgr.config.notifications.thresholds);
        }
        "language" => mgr.config.language = value.to_string(),
        "polling_interval_seconds" => match value.parse::<u64>() {
            Ok(seconds) => mgr.config.polling_interval_seconds = seconds,
            Err(_) => {
                println!("Not a number: {value}");
                return;
            }
        },
        "plan_override" => {
            mgr.config.plan_override = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        }
        _ => {
            println!("Unknown setting: {key}");
            return;
        }
    }

    mgr.config.validate();
    mgr.save();
    println!("Updated {key}");
}

async fn poll_once(exe_dir: &Path, client: &ClaudeClient, config: &Config) {
    let i18n = I18n::from_config(&config.language);
    mark_refreshing(exe_dir, &i18n);

    // Codex usage comes from local session logs, so it is available even when
    // the Claude API call below fails — read it first and carry it into every
    // outcome.
    let codex = if config.show_chatgpt_section {
        codex_metrics(&i18n)
    } else {
        Vec::new()
    };

    let credential = match read_claude_token() {
        Ok(credential) => credential,
        Err(e) => {
            let message = format!("Claude credentials unavailable: {e}");
            append_log(exe_dir, &message);
            if config.token_expiry_warning {
                notify(
                    "ClaudeMeter",
                    i18n.t("run_claude_login_desc"),
                    config.notifications.sound,
                );
            }
            write_error(exe_dir, message, codex, config, &i18n);
            return;
        }
    };

    let mut usage = match client.fetch_usage(&credential.access_token).await {
        Ok(usage) => usage,
        Err(e) => {
            let message = format!("Usage poll failed: {e}");
            append_log(exe_dir, &message);
            write_error(exe_dir, message, codex, config, &i18n);
            return;
        }
    };

    usage.subscription_type = credential.subscription_type;
    usage.rate_limit_tier = credential.rate_limit_tier;

    save_history(exe_dir, &usage);
    publish_status(exe_dir, &usage, codex, config, &i18n);
}

/// Read the local Codex session logs and turn the rolling windows into menu
/// entries. Keys are prefixed with `codex_` so the Swift UI never confuses them
/// with Claude's `five_hour` / `seven_day` metrics.
fn codex_metrics(i18n: &I18n) -> Vec<MetricEntry> {
    let Some(dir) = crate::providers::codex::default_sessions_dir() else {
        return Vec::new();
    };
    let Some(status) = crate::providers::codex::latest_status(&dir, chrono::Utc::now()) else {
        return Vec::new();
    };
    [
        ("five_hour", &status.five_hour),
        ("seven_day", &status.weekly),
    ]
    .into_iter()
    .filter_map(|(key, window)| {
        let window = window.as_ref()?;
        Some(MetricEntry {
            key: format!("codex_{key}"),
            name: format!("{} \u{00B7} {}", i18n.t("CODEX"), i18n.metric_name(key)),
            percent: window.used_percent.round() as u32,
            resets_at: window.resets_at_rfc3339(),
        })
    })
    .collect()
}

/// Replace any Codex entries in a preserved status with a freshly read set,
/// leaving the Claude metrics untouched.
fn merge_codex(metrics: &mut Vec<MetricEntry>, codex: Vec<MetricEntry>) {
    metrics.retain(|m| !m.key.starts_with("codex_"));
    metrics.extend(codex);
}

fn save_history(exe_dir: &Path, usage: &UsageResponse) {
    let db = match Database::open(exe_dir) {
        Ok(db) => db,
        Err(e) => {
            append_log(exe_dir, &format!("Database unavailable: {e}"));
            return;
        }
    };

    for (metric, value) in usage.all_metrics() {
        if let Err(e) = db.insert(
            "claude",
            &metric,
            value.utilization,
            value.resets_at.as_deref(),
        ) {
            append_log(exe_dir, &format!("Failed to save metric {metric}: {e}"));
        }
    }
}

fn publish_status(
    exe_dir: &Path,
    usage: &UsageResponse,
    codex: Vec<MetricEntry>,
    config: &Config,
    i18n: &I18n,
) {
    let plan_override = config.plan_override.as_deref();
    let percent = usage.max_utilization().unwrap_or(0.0).round() as u32;
    let plan = plan_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| usage.detected_plan());
    let now = Local::now();
    let last_api_update = now.to_rfc3339();
    let message = format!("{plan}: {percent}%");

    // Respect the same "which limits do I care about" settings as the Windows
    // dashboard, so hiding model limits or extra usage applies on both.
    let filter = MetricFilter::from_config(config);
    let mut metrics: Vec<MetricEntry> = usage
        .all_metrics()
        .into_iter()
        .filter(|(key, _)| !filter.hides(key))
        .map(|(key, m)| MetricEntry {
            name: i18n.metric_name(&key),
            percent: m.utilization.round() as u32,
            resets_at: m.resets_at.clone(),
            key,
        })
        .collect();
    metrics.extend(codex);

    let tier_note = plan_override.and_then(|p| build_tier_note(p, usage));

    // 24h history for the menu-bar chart. save_history() already inserted this
    // poll's reading, so reopening the DB here picks up the freshest bucket.
    let chart = Database::open(exe_dir)
        .and_then(|db| db.query_24h_chart())
        .map(|slots| slots.iter().map(|v| v.round() as u32).collect())
        .unwrap_or_default();

    // Past 5-hour session reset points as "hours ago", stepping back by 5h from
    // the most recent reset up to 24h. Same logic as windows_app.rs.
    let mut chart_resets = Vec::new();
    if let Some(secs) = usage
        .five_hour
        .as_ref()
        .and_then(|fh| fh.resets_at.as_deref())
        .and_then(seconds_until)
    {
        let hours_until = secs as f64 / 3600.0;
        let mut hours_ago = 5.0 - hours_until;
        while hours_ago <= 24.0 {
            if hours_ago > 0.0 {
                chart_resets.push(hours_ago);
            }
            hours_ago += 5.0;
        }
    }

    let alerted = check_alerts(exe_dir, &metrics, config, i18n);

    let status = MacStatus {
        state: "live".to_string(),
        title: format!("{percent}%"),
        detail: message.clone(),
        plan: Some(plan),
        percent: Some(percent),
        metrics,
        tier_note,
        last_api_update: Some(last_api_update),
        data_age_seconds: Some(0),
        error: None,
        chart,
        chart_resets,
        labels: menu_labels(i18n, config),
        alerted,
    };

    append_log(
        exe_dir,
        &format!("[{}] {}", now.format("%Y-%m-%d %H:%M:%S"), message),
    );
    write_status(exe_dir, &status);
}

/// Fire a notification the first time a metric crosses each configured
/// threshold, and re-arm a threshold once usage falls back below it.
///
/// Returns the updated per-metric high-water marks, which are persisted in
/// `status.json` so the alert state survives the one-shot `--refresh` process
/// the menu bar spawns. Codex windows are included: they are ordinary metrics
/// here, so they alert exactly like Claude's.
fn check_alerts(
    exe_dir: &Path,
    metrics: &[MetricEntry],
    config: &Config,
    i18n: &I18n,
) -> HashMap<String, u8> {
    let mut alerted = read_status(exe_dir)
        .map(|prev| prev.alerted)
        .unwrap_or_default();

    if !config.notifications.enabled {
        return alerted;
    }

    for metric in metrics {
        let percent = metric.percent.min(u8::MAX as u32) as u8;
        let previous = alerted.get(&metric.key).copied().unwrap_or(0);

        // Highest configured threshold this metric currently exceeds.
        let crossed = config
            .notifications
            .thresholds
            .iter()
            .copied()
            .filter(|threshold| percent >= *threshold)
            .max()
            .unwrap_or(0);

        if crossed > previous {
            let title = if crossed >= 90 {
                i18n.t("Usage Critical")
            } else {
                i18n.t("Usage Alert")
            };
            notify(
                title,
                &format!("{}: {}%", metric.name, metric.percent),
                config.notifications.sound,
            );
        }

        if crossed == 0 {
            alerted.remove(&metric.key);
        } else {
            alerted.insert(metric.key.clone(), crossed);
        }
    }

    alerted
}

/// Localized strings for the Swift menu, keyed by stable identifiers.
fn menu_labels(i18n: &I18n, config: &Config) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    let mut put = |key: &str, value: &str| {
        labels.insert(key.to_string(), value.to_string());
    };

    put("refresh_now", i18n.t("Refresh Now"));
    put("open_claude_usage", i18n.t("Open Claude Usage"));
    put("check_for_updates", i18n.t("Check for updates"));
    put("open_config", i18n.t("Open Config"));
    put("export_config", i18n.t("Export Config"));
    put("import_config", i18n.t("Import Config"));
    put("export_csv", i18n.t("Export History (CSV)"));
    put("export_json", i18n.t("Export History (JSON)"));
    put("enable_autostart", i18n.t("Enable Autostart"));
    put("disable_autostart", i18n.t("Disable Autostart"));
    put("open_logs", i18n.t("Open Logs"));
    put("quit", i18n.t("Quit"));
    put("settings", i18n.t("Settings"));
    put("language", i18n.t("Language"));
    put("auto", i18n.t("Auto"));
    put("notifications", i18n.t("Notifications"));
    put("notification_sound", i18n.t("Notification sound"));
    put("alert_thresholds", i18n.t("Alert thresholds"));
    put("show_codex_section", i18n.t("Show Codex section"));
    put("show_model_limits", i18n.t("Show model limits"));
    put("show_extra_usage", i18n.t("Show extra usage"));
    put(
        "show_startup_notification",
        i18n.t("Show startup notification"),
    );
    put(
        "show_login_expiry_warning",
        i18n.t("Show login expiry warning"),
    );
    put("usage_history_24h", i18n.t("Usage History (24h)"));
    put("freshness", i18n.t("Freshness"));
    put("live", i18n.t("Live"));
    put("cached", i18n.t("Cached"));
    put("api_error", i18n.t("api_error"));
    put("refreshing", i18n.t("Refreshing"));
    put("resets_in", i18n.t("resets in"));
    put("update_available", i18n.t("Update available"));
    put("latest_version", i18n.t("latest_version"));
    put("update_check_failed", i18n.t("update_check_failed"));
    put("pace_too_early", i18n.t("pace_too_early"));
    put("pace_projection", i18n.t("pace_projection"));
    put("time_ago", i18n.t("time_ago"));

    // Current values so the settings submenu can render checkmarks without
    // reading config.json itself.
    put(
        "value_thresholds",
        &crate::config::format_notification_thresholds(&config.notifications.thresholds),
    );
    put("value_language", &config.language);
    for (key, enabled) in [
        ("value_show_codex_section", config.show_chatgpt_section),
        ("value_show_model_limits", config.show_model_limits),
        ("value_show_extra_usage", config.show_extra_usage),
        (
            "value_show_startup_notification",
            config.show_startup_notification,
        ),
        (
            "value_show_login_expiry_warning",
            config.token_expiry_warning,
        ),
        ("value_notification_sound", config.notifications.sound),
        ("value_notifications_enabled", config.notifications.enabled),
    ] {
        put(key, if enabled { "true" } else { "false" });
    }

    labels
}

/// Plan tier as a multiple of the Pro base allowance. Used to estimate what
/// usage would look like on a smaller plan. Recognized labels: Pro, Max 5x,
/// Max 20x (bare "Max" assumed 5x, its entry tier).
fn plan_multiplier(plan: &str) -> Option<f64> {
    let p = plan.to_lowercase();
    if p.contains("20x") {
        Some(20.0)
    } else if p.contains("5x") || p.contains("max") {
        Some(5.0)
    } else if p.contains("pro") {
        Some(1.0)
    } else {
        None
    }
}

/// The next cheaper tier and its multiplier, or None if already at the bottom.
fn lower_tier(mult: f64) -> Option<(&'static str, f64)> {
    if mult >= 20.0 {
        Some(("Max 5x", 5.0))
    } else if mult >= 5.0 {
        Some(("Pro", 1.0))
    } else {
        None
    }
}

/// "On Max 5x: weekly ~96%, session ~124% — would throttle": estimate the
/// session and weekly usage if the same work ran on the next tier down.
/// Assumes limits scale linearly with the tier multiplier.
fn build_tier_note(plan: &str, usage: &UsageResponse) -> Option<String> {
    let mult = plan_multiplier(plan)?;
    let (lower_name, lower_mult) = lower_tier(mult)?;
    let factor = mult / lower_mult;

    let week = usage.seven_day.as_ref().map(|m| m.utilization);
    let five = usage.five_hour.as_ref().map(|m| m.utilization);

    let mut parts = Vec::new();
    if let Some(w) = week {
        parts.push(format!("weekly ~{}%", (w * factor).round() as i64));
    }
    if let Some(f) = five {
        parts.push(format!("session ~{}%", (f * factor).round() as i64));
    }
    if parts.is_empty() {
        return None;
    }

    let over = week.is_some_and(|w| w * factor > 100.0) || five.is_some_and(|f| f * factor > 100.0);
    let verdict = if over {
        " — would throttle"
    } else {
        " — would fit"
    };
    Some(format!(
        "On {}: {}{}",
        lower_name,
        parts.join(", "),
        verdict
    ))
}

/// Seconds until an RFC3339 reset timestamp (negative if already past).
/// Local copy of the i18n helper, which is Windows-gated.
fn seconds_until(resets_at: &str) -> Option<i64> {
    let reset: chrono::DateTime<chrono::Utc> = resets_at.parse().ok()?;
    Some(
        reset
            .signed_duration_since(chrono::Utc::now())
            .num_seconds(),
    )
}

fn write_status(exe_dir: &Path, status: &MacStatus) {
    let path = exe_dir.join("status.json");
    match serde_json::to_string_pretty(status) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                log::warn!("Failed to write macOS status: {e}");
            }
        }
        Err(e) => log::warn!("Failed to serialize macOS status: {e}"),
    }
}

/// Begin a refresh without blanking the menu. If a prior good reading exists,
/// keep its numbers on screen (and clear any stale error) instead of flashing
/// an empty "refreshing" state every poll. Only show the blank refreshing
/// placeholder on the very first run, when there is no data yet.
fn mark_refreshing(exe_dir: &Path, i18n: &I18n) {
    if let Some(mut prev) = read_status(exe_dir) {
        if prev.percent.is_some() {
            prev.error = None;
            write_status(exe_dir, &prev);
            return;
        }
    }
    let mut status = MacStatus::refreshing();
    status.title = i18n.t("Refreshing").to_string();
    write_status(exe_dir, &status);
}

/// Last published status, if it is present and parseable.
fn read_status(exe_dir: &Path) -> Option<MacStatus> {
    let contents = std::fs::read_to_string(exe_dir.join("status.json")).ok()?;
    serde_json::from_str::<MacStatus>(&contents).ok()
}

/// Record a fetch failure without discarding the last good reading.
/// If a prior live status exists, keep its data and metrics (so the menu
/// keeps showing the numbers with a staleness indicator) and just attach
/// the error. Only blank out when there is no prior data to show.
fn write_error(
    exe_dir: &Path,
    message: String,
    codex: Vec<MetricEntry>,
    config: &Config,
    i18n: &I18n,
) {
    let labels = menu_labels(i18n, config);
    if let Some(mut prev) = read_status(exe_dir) {
        if prev.percent.is_some() {
            prev.error = Some(message);
            merge_codex(&mut prev.metrics, codex);
            prev.alerted = check_alerts(exe_dir, &prev.metrics, config, i18n);
            prev.labels = labels;
            write_status(exe_dir, &prev);
            return;
        }
    }
    // No Claude data yet — still surface Codex, which needs no API or token.
    let mut status = MacStatus::error(message);
    status.alerted = check_alerts(exe_dir, &codex, config, i18n);
    status.metrics = codex;
    status.title = i18n.t("api_error").to_string();
    status.labels = labels;
    write_status(exe_dir, &status);
}

fn print_status(exe_dir: &Path) {
    let path = exe_dir.join("status.json");
    match std::fs::read_to_string(path) {
        Ok(status) => println!("{status}"),
        Err(_) => println!(
            "{}",
            serde_json::to_string(&MacStatus::refreshing()).unwrap()
        ),
    }
}

fn append_log(exe_dir: &Path, message: &str) {
    let path = exe_dir.join("claudemeter.log");
    let line = format!("{}\n", message);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(line.as_bytes())
        });
}

fn notify(title: &str, message: &str, sound: bool) {
    let sound_clause = if sound { " sound name \"Ping\"" } else { "" };
    let script = format!(
        "display notification \"{}\" with title \"{}\"{}",
        escape_applescript(message),
        escape_applescript(title),
        sound_clause
    );

    let _ = Command::new("osascript").arg("-e").arg(script).status();
}

fn escape_applescript(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn app_data_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("ClaudeMeter");
    }

    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refreshing_status_shape() {
        let status = MacStatus::refreshing();
        assert_eq!(status.state, "refreshing");
        assert_eq!(status.title, "Refreshing...");
        assert!(status.percent.is_none());
    }

    #[test]
    fn test_escape_applescript() {
        assert_eq!(escape_applescript(r#"a\b"c"#), r#"a\\b\"c"#);
    }
}
