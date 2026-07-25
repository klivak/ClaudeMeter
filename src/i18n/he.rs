use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(
        "5-hour session",
        "\u{05e1}\u{05e9}\u{05d9}\u{05d4} \u{05e9}\u{05dc} 5 \u{05e9}\u{05e2}\u{05d5}\u{05ea}",
    );
    m.insert(
        "Weekly (7-day)",
        "\u{05e9}\u{05d1}\u{05d5}\u{05e2}\u{05d9} (7 \u{05d9}\u{05de}\u{05d9}\u{05dd})",
    );
    m.insert("Opus (7-day)", "Opus (7 \u{05d9}\u{05de}\u{05d9}\u{05dd})");
    m.insert(
        "Sonnet (7-day)",
        "Sonnet (7 \u{05d9}\u{05de}\u{05d9}\u{05dd})",
    );
    m.insert(
        "OAuth Apps (7-day)",
        "OAuth Apps (7 \u{05d9}\u{05de}\u{05d9}\u{05dd})",
    );
    m.insert(
        "resets in",
        "\u{05de}\u{05ea}\u{05d0}\u{05e4}\u{05e1} \u{05d1}\u{05e2}\u{05d5}\u{05d3}",
    );
    m.insert("Plan", "\u{05ea}\u{05d5}\u{05db}\u{05e0}\u{05d9}\u{05ea}");
    m.insert("Pro", "Pro");
    m.insert("Max", "Max");
    m.insert(
        "Claude Code not detected",
        "Claude Code \u{05dc}\u{05d0} \u{05d6}\u{05d5}\u{05d4}\u{05d4}",
    );
    m.insert("credentials_not_found", "\u{05d0}\u{05d9}\u{05e9}\u{05d5}\u{05e8}\u{05d9}\u{05dd} \u{05dc}\u{05d0} \u{05e0}\u{05de}\u{05e6}\u{05d0}\u{05d5}");
    m.insert(
        "connection_error",
        "\u{05e9}\u{05d2}\u{05d9}\u{05d0}\u{05ea} \u{05d7}\u{05d9}\u{05d1}\u{05d5}\u{05e8}",
    );
    m.insert("token_expired", "\u{05d4}\u{05d8}\u{05d5}\u{05e7}\u{05df} \u{05e4}\u{05d2} \u{05ea}\u{05d5}\u{05e7}\u{05e3}");
    m.insert(
        "token_expired_desc",
        "\u{05d4}\u{05d8}\u{05d5}\u{05e7}\u{05df} \u{05e4}\u{05d2} \u{05ea}\u{05d5}\u{05e7}\u{05e3}. \u{05d4}\u{05e8}\u{05d9}\u{05e6}\u{05d5} `claude login` \u{05d1}\u{05d8}\u{05e8}\u{05de}\u{05d9}\u{05e0}\u{05dc}.",
    );
    m.insert(
        "rate_limited",
        "\u{05d4}\u{05d2}\u{05d1}\u{05dc}\u{05ea} \u{05e7}\u{05e6}\u{05d1}",
    );
    m.insert(
        "server_error",
        "\u{05e9}\u{05d2}\u{05d9}\u{05d0}\u{05ea} \u{05e9}\u{05e8}\u{05ea}",
    );
    m.insert(
        "server_error_desc",
        "Anthropic API \u{05dc}\u{05d0} \u{05d6}\u{05de}\u{05d9}\u{05df} \u{05d6}\u{05de}\u{05e0}\u{05d9}\u{05ea}. \u{05d9}\u{05e0}\u{05e1}\u{05d4} \u{05e9}\u{05d5}\u{05d1} \u{05d0}\u{05d5}\u{05d8}\u{05d5}\u{05de}\u{05d8}\u{05d9}\u{05ea}.",
    );
    m.insert(
        "run_claude_login_desc",
        "Claude Code \u{05de}\u{05d5}\u{05ea}\u{05e7}\u{05df} \u{05d0}\u{05da} \u{05dc}\u{05d0} \u{05de}\u{05d7}\u{05d5}\u{05d1}\u{05e8}. \u{05d4}\u{05e8}\u{05d9}\u{05e6}\u{05d5} `claude login`.",
    );
    m.insert(
        "install_claude_desc",
        "\u{05d4}\u{05ea}\u{05e7}\u{05d9}\u{05e0}\u{05d5} Claude Code \u{05d5}\u{05d4}\u{05e8}\u{05d9}\u{05e6}\u{05d5} `claude login`.",
    );
    m.insert(
        "Install Claude Code \u{2192}",
        "\u{05d4}\u{05ea}\u{05e7}\u{05e0}\u{05ea} Claude Code \u{2192}",
    );
    m.insert(
        "openai_no_api",
        "OpenAI \u{05dc}\u{05d0} \u{05de}\u{05e1}\u{05e4}\u{05e7}\u{05ea} API \u{05dc}\u{05de}\u{05e2}\u{05e7}\u{05d1} \u{05d0}\u{05d7}\u{05e8} \u{05e9}\u{05d9}\u{05de}\u{05d5}\u{05e9}.",
    );
    m.insert(
        "Check your usage manually:",
        "\u{05d1}\u{05d3}\u{05e7}\u{05d5} \u{05d9}\u{05d3}\u{05e0}\u{05d9}\u{05ea}:",
    );
    m.insert(
        "Open ChatGPT Usage \u{2192}",
        "\u{05e4}\u{05ea}\u{05d7} ChatGPT Usage \u{2192}",
    );
    m.insert(
        "Refresh Now",
        "\u{05e8}\u{05e2}\u{05e0}\u{05df} \u{05e2}\u{05db}\u{05e9}\u{05d9}\u{05d5}",
    );
    m.insert(
        "Open Dashboard",
        "\u{05e4}\u{05ea}\u{05d7} \u{05dc}\u{05d5}\u{05d7} \u{05d1}\u{05e7}\u{05e8}\u{05d4}",
    );
    m.insert("Export History (CSV)", "\u{05d9}\u{05d9}\u{05e6}\u{05d5}\u{05d0} \u{05d4}\u{05d9}\u{05e1}\u{05d8}\u{05d5}\u{05e8}\u{05d9}\u{05d4} (CSV)");
    m.insert("Export History (JSON)", "\u{05d9}\u{05d9}\u{05e6}\u{05d5}\u{05d0} \u{05d4}\u{05d9}\u{05e1}\u{05d8}\u{05d5}\u{05e8}\u{05d9}\u{05d4} (JSON)");
    m.insert("Show extra usage", "הצג שימוש נוסף");
    m.insert("Show model limits", "\u{5d4}\u{5e6}\u{5d2} \u{5de}\u{5d2}\u{5d1}\u{5dc}\u{5d5}\u{5ea} \u{5de}\u{5d5}\u{5d3}\u{5dc}\u{5d9}\u{5dd}");
    m.insert("Usage link icons", "סמלי קישורי שימוש");
    m.insert("Open usage", "פתח שימוש");
    m.insert("Service status", "סטטוס שירות");
    m.insert("CODEX", "CODEX");
    m.insert(
        "Settings",
        "\u{05d4}\u{05d2}\u{05d3}\u{05e8}\u{05d5}\u{05ea}",
    );
    m.insert(
        "Start with Windows",
        "\u{05d4}\u{05e4}\u{05e2}\u{05dc} \u{05e2}\u{05dd} Windows",
    );
    m.insert("About", "\u{05d0}\u{05d5}\u{05d3}\u{05d5}\u{05ea}");
    m.insert("Exit", "\u{05d9}\u{05e6}\u{05d9}\u{05d0}\u{05d4}");
    m.insert(
        "Last updated:",
        "\u{05e2}\u{05d3}\u{05db}\u{05d5}\u{05df} \u{05d0}\u{05d7}\u{05e8}\u{05d5}\u{05df}:",
    );
    m.insert("Refresh", "\u{05e8}\u{05e2}\u{05e0}\u{05d5}\u{05df}");
    m.insert("Status", "\u{05de}\u{05e6}\u{05d1}");
    m.insert(
        "Usage Alert",
        "\u{05d4}\u{05ea}\u{05e8}\u{05d0}\u{05ea} \u{05e9}\u{05d9}\u{05de}\u{05d5}\u{05e9}",
    );
    m.insert(
        "Usage Critical",
        "\u{05e9}\u{05d9}\u{05de}\u{05d5}\u{05e9} \u{05e7}\u{05e8}\u{05d9}\u{05d8}\u{05d9}",
    );
    m.insert(
        "Running in system tray. Click the icon for details.",
        "\u{05e4}\u{05d5}\u{05e2}\u{05dc} \u{05d1}\u{05de}\u{05d2}\u{05e9} \u{05d4}\u{05de}\u{05e2}\u{05e8}\u{05db}\u{05ea}. \u{05dc}\u{05d7}\u{05e6}\u{05d5} \u{05e2}\u{05dc} \u{05d4}\u{05e1}\u{05de}\u{05dc} \u{05dc}\u{05e4}\u{05e8}\u{05d8}\u{05d9}\u{05dd}.",
    );
    m.insert(
        "Compact mode",
        "\u{05de}\u{05e6}\u{05d1} \u{05de}\u{05e6}\u{05d5}\u{05de}\u{05e6}\u{05dd}",
    );
    m.insert(
        "Theme",
        "\u{05e2}\u{05e8}\u{05db}\u{05ea} \u{05e0}\u{05d5}\u{05e9}\u{05d0}",
    );
    m.insert("Language", "\u{05e9}\u{05e4}\u{05d4}");
    m.insert(
        "Notifications",
        "\u{05d4}\u{05ea}\u{05e8}\u{05d0}\u{05d5}\u{05ea}",
    );
    m.insert("Dark", "\u{05db}\u{05d4}\u{05d4}");
    m.insert("Light", "\u{05d1}\u{05d4}\u{05d9}\u{05e8}");
    m.insert(
        "Auto",
        "\u{05d0}\u{05d5}\u{05d8}\u{05d5}\u{05de}\u{05d8}\u{05d9}",
    );
    m.insert(
        "Show ChatGPT section",
        "\u{05d4}\u{05e6}\u{05d2} \u{05de}\u{05d3}\u{05d5}\u{05e8} ChatGPT",
    );
    m.insert("Enabled", "\u{05de}\u{05d5}\u{05e4}\u{05e2}\u{05dc}");
    m.insert("Sound", "\u{05e6}\u{05dc}\u{05d9}\u{05dc}");
    m.insert("Thresholds", "\u{05e1}\u{05e4}\u{05d9}\u{05dd}");
    m.insert(
        "Polling interval",
        "\u{05de}\u{05e8}\u{05d5}\u{05d5}\u{05d7} \u{05d1}\u{05d3}\u{05d9}\u{05e7}\u{05d4}",
    );
    m.insert("seconds", "\u{05e9}\u{05e0}\u{05d9}\u{05d5}\u{05ea}");
    m.insert("Startup", "\u{05d4}\u{05e4}\u{05e2}\u{05dc}\u{05d4}");
    m.insert("General", "\u{05db}\u{05dc}\u{05dc}\u{05d9}");
    m.insert("Back", "\u{2190} \u{05d7}\u{05d6}\u{05e8}\u{05d4}");
    m.insert(
        "Open Claude.ai \u{2192}",
        "\u{05e4}\u{05ea}\u{05d7} Claude.ai \u{2192}",
    );
    m.insert("ClaudeMeter", "ClaudeMeter");
    m.insert("CLAUDE", "CLAUDE");
    m.insert("CHATGPT / CODEX", "CHATGPT / CODEX");
    m.insert(
        "Usage History",
        "\u{05d4}\u{05d9}\u{05e1}\u{05d8}\u{05d5}\u{05e8}\u{05d9}\u{05d4}",
    );
    m.insert("Usage History (24h)", "\u{05d4}\u{05d9}\u{05e1}\u{05d8}\u{05d5}\u{05e8}\u{05d9}\u{05d4} (24 \u{05e9}\u{05e2}\u{05d5}\u{05ea})");
    m.insert("Auto (English)", "Auto (English)");
    m.insert("at", "\u{05d1}");
    m.insert(
        "Resets in",
        "\u{05de}\u{05ea}\u{05d0}\u{05e4}\u{05e1} \u{05d1}\u{05e2}\u{05d5}\u{05d3}",
    );
    m.insert(
        "Tray icon colors:",
        "\u{05e6}\u{05d1}\u{05e2}\u{05d9} \u{05e1}\u{05de}\u{05dc}:",
    );
    m.insert("< 50% usage", "< 50%");
    m.insert("50-79% usage", "50\u{2013}79%");
    m.insert(">= 80% usage", "\u{2265} 80%");
    m.insert(
        "No data",
        "\u{05d0}\u{05d9}\u{05df} \u{05e0}\u{05ea}\u{05d5}\u{05e0}\u{05d9}\u{05dd}",
    );
    m.insert("exceeded", "\u{05d7}\u{05e8}\u{05d9}\u{05d2}\u{05d4}");
    m.insert("Show widget", "\u{05d4}\u{05e6}\u{05d2} widget");
    m.insert(
        "Check for updates",
        "\u{05d1}\u{05d3}\u{05d5}\u{05e7} \u{05e2}\u{05d3}\u{05db}\u{05d5}\u{05e0}\u{05d9}\u{05dd}",
    );
    m.insert("Accessibility patterns", "\u{05ea}\u{05d1}\u{05e0}\u{05d9}\u{05d5}\u{05ea} \u{05e0}\u{05d2}\u{05d9}\u{05e9}\u{05d5}\u{05ea}");
    m.insert(
        "Update available",
        "\u{05e2}\u{05d3}\u{05db}\u{05d5}\u{05df} \u{05d6}\u{05de}\u{05d9}\u{05df}",
    );
    m.insert(
        "is available. Click to download.",
        "\u{05d6}\u{05de}\u{05d9}\u{05df}. \u{05dc}\u{05d7}\u{05e6}\u{05d5} \u{05dc}\u{05d4}\u{05d5}\u{05e8}\u{05d3}\u{05d4}.",
    );
    m.insert(
        "Icon style",
        "\u{05e1}\u{05d2}\u{05e0}\u{05d5}\u{05df} \u{05e1}\u{05de}\u{05dc}",
    );
    m.insert("Number", "\u{05de}\u{05e1}\u{05e4}\u{05e8}");
    m.insert("Ring", "\u{05d8}\u{05d1}\u{05e2}\u{05ea}");
    m.insert("Bar", "\u{05e4}\u{05e1}");
    m.insert("Pie", "\u{05e2}\u{05d5}\u{05d2}\u{05d4}");
    m.insert(
        "Dashboard layout",
        "\u{05e4}\u{05e8}\u{05d9}\u{05e1}\u{05d4}",
    );
    m.insert(
        "Minimal",
        "\u{05de}\u{05d9}\u{05e0}\u{05d9}\u{05de}\u{05dc}\u{05d9}",
    );
    m.insert("Standard", "\u{05e8}\u{05d2}\u{05d9}\u{05dc}");
    m.insert("Detailed", "\u{05de}\u{05e4}\u{05d5}\u{05e8}\u{05d8}");
    m.insert(
        "Hide Extra Usage",
        "\u{05d4}\u{05e1}\u{05ea}\u{05e8} Extra Usage",
    );
    m.insert("stale_token_expired", "נתונים ישנים — הטוקן פג");
    m.insert("stale_data", "נתונים ישנים — הערכים האחרונים");
    m.insert("codex_no_data", "אין חלון מגבלות פעיל ל-Codex. השימוש נקרא מיומני הסשנים המקומיים של Codex — התחילו או המשיכו סשן והוא יופיע כאן.");
    m.insert("Open Codex Usage →", "פתח שימוש ב-Codex →");
    m.insert("Midnight", "חצות");
    m.insert("Sunset", "שקיעה");
    m.insert("Show Codex section", "הצג את מקטע Codex");
    m.insert(
        "Reopen the tray popup to refresh",
        "פתחו שוב את החלון כדי לרענן",
    );
    m.insert("Show startup notification", "הצג התראה בהפעלה");
    m.insert("Show login expiry warning", "התראה על פקיעת ההתחברות");
    m.insert("Alert thresholds", "ספי התראה");
    m.insert("Notification sound", "צליל התראה");
    m.insert("Test notification", "התראת בדיקה");
    m.insert("Send", "שלח");
    m.insert("pace_projection", "קצב: בדרך ל-~{}% עד האיפוס");
    m.insert("pace_too_early", "קצב: מוקדם מדי לחזות");
    m.insert("Open Claude Usage", "פתח שימוש ב-Claude");
    m.insert("Open Config", "פתח את קובץ ההגדרות");
    m.insert("Export Config", "ייצוא הגדרות...");
    m.insert("Import Config", "ייבוא הגדרות...");
    m.insert("Enable Autostart", "הפעל הפעלה אוטומטית");
    m.insert("Disable Autostart", "בטל הפעלה אוטומטית");
    m.insert("Open Logs", "פתח יומנים");
    m.insert("Quit", "יציאה");
    m.insert("Freshness", "טריות הנתונים");
    m.insert("Live", "בזמן אמת");
    m.insert("Cached", "מהמטמון / אין עדיין נתוני API");
    m.insert("api_error", "שגיאת API");
    m.insert("Refreshing", "מרענן...");
    m.insert("time_ago", "לפני {}");
    m.insert("latest_version", "אתה משתמש בגרסה העדכנית ביותר.");
    m.insert("update_check_failed", "לא ניתן היה לבדוק עדכונים.");
    m
}
