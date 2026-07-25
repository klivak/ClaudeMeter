use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut strings = super::en::strings();
    strings.insert("5-hour session", "5 stundu sesija");
    strings.insert("Weekly (7-day)", "Nedēļas (7 dienas)");
    strings.insert("Opus (7-day)", "Opus (7 dienas)");
    strings.insert("Sonnet (7-day)", "Sonnet (7 dienas)");
    strings.insert("OAuth Apps (7-day)", "OAuth lietotnes (7 dienas)");
    strings.insert("resets in", "atiestatīsies pēc");
    strings.insert("Plan", "Plāns");
    strings.insert("Claude Code not detected", "Claude Code nav atrasts");
    strings.insert("credentials_not_found", "Akreditācijas dati nav atrasti");
    strings.insert("connection_error", "Savienojuma kļūda");
    strings.insert("token_expired", "Pilnvara ir beigusies");
    strings.insert("stale_token_expired", "Novecojis — pilnvara ir beigusies");
    strings.insert("stale_data", "Novecojis — pēdējie zināmie dati");
    strings.insert(
        "token_expired_desc",
        "OAuth pilnvara ir beigusies. Lai to atjaunotu, terminālī palaidiet `claude login`.",
    );
    strings.insert("rate_limited", "Sasniegts pieprasījumu ierobežojums");
    strings.insert("server_error", "Servera kļūda");
    strings.insert(
        "server_error_desc",
        "Anthropic API īslaicīgi nav pieejama. Mēģinājums tiks automātiski atkārtots.",
    );
    strings.insert("run_claude_login_desc", "Claude Code ir instalēts, bet neesat pieteicies. Lai savienotu kontu, terminālī palaidiet `claude login`.");
    strings.insert("install_claude_desc", "Instalējiet Claude Code un palaidiet `claude login`, lai iespējotu automātisku lietojuma uzskaiti.");
    strings.insert("Install Claude Code →", "Instalēt Claude Code →");
    strings.insert(
        "codex_no_data",
        "Nav aktīva Codex limitu loga. Lietojums tiek nolasīts no vietējiem Codex sesiju žurnāliem \u{2014} sāciet vai turpiniet sesiju, un tas parādīsies šeit.",
    );
    strings.insert(
        "Check your usage manually:",
        "Pārbaudiet lietojumu manuāli:",
    );
    strings.insert("Open Codex Usage →", "Atvērt Codex lietojumu →");
    strings.insert("Refresh Now", "Atsvaidzināt tagad");
    strings.insert("Open Dashboard", "Atvērt informācijas paneli");
    strings.insert("Export History (CSV)", "Eksportēt vēsturi (CSV)");
    strings.insert("Export History (JSON)", "Eksportēt vēsturi (JSON)");
    strings.insert("Settings", "Iestatījumi");
    strings.insert("Start with Windows", "Palaist kopā ar Windows");
    strings.insert("About", "Par programmu");
    strings.insert("Exit", "Iziet");
    strings.insert("Last updated:", "Pēdējoreiz atjaunināts:");
    strings.insert("Refresh", "Atsvaidzināt");
    strings.insert("Status", "Statuss");
    strings.insert("Usage Alert", "Lietojuma brīdinājums");
    strings.insert("Usage Critical", "Kritisks lietojums");
    strings.insert(
        "Running in system tray. Click the icon for details.",
        "Darbojas sistēmas teknē. Noklikšķiniet uz ikonas, lai skatītu informāciju.",
    );
    strings.insert("Compact mode", "Kompaktais režīms");
    strings.insert("Theme", "Dizains");
    strings.insert("Language", "Valoda");
    strings.insert("Notifications", "Paziņojumi");
    strings.insert("Dark", "Tumšs");
    strings.insert("Light", "Gaišs");
    strings.insert("Auto", "Automātiski");
    strings.insert("Midnight", "Pusnakts");
    strings.insert("Sunset", "Saulriets");
    strings.insert("Show Codex section", "Rādīt Codex sadaļu");
    strings.insert(
        "Reopen the tray popup to refresh",
        "Lai atsvaidzinātu, atkārtoti atveriet sistēmas teknes logu",
    );
    strings.insert("Enabled", "Iespējots");
    strings.insert("Sound", "Skaņa");
    strings.insert("Thresholds", "Sliekšņi");
    strings.insert("Polling interval", "Pārbaudes intervāls");
    strings.insert("seconds", "sekundes");
    strings.insert("Startup", "Palaišana");
    strings.insert("General", "Vispārīgi");
    strings.insert("Back", "← Atpakaļ");
    strings.insert("Open Claude.ai →", "Atvērt Claude.ai →");
    strings.insert("Usage link icons", "Lietojuma saišu ikonas");
    strings.insert("Open usage", "Atvērt lietojumu");
    strings.insert("Service status", "Pakalpojuma statuss");
    strings.insert("Usage History", "Lietojuma vēsture");
    strings.insert("Usage History (24h)", "Lietojuma vēsture (24 h)");
    strings.insert("Auto (English)", "Automātiski (Latviešu)");
    strings.insert("at", "plkst.");
    strings.insert("Resets in", "Atiestatīsies pēc");
    strings.insert("Tray icon colors:", "Sistēmas teknes ikonas krāsas:");
    strings.insert("< 50% usage", "< 50% lietojums");
    strings.insert("50-79% usage", "50–79% lietojums");
    strings.insert(">= 80% usage", "≥ 80% lietojums");
    strings.insert("No data", "Nav datu");
    strings.insert("exceeded", "pārsniegts");
    strings.insert("Show widget", "Rādīt logrīku");
    strings.insert("Check for updates", "Pārbaudīt atjauninājumus");
    strings.insert("Accessibility patterns", "Pieejamības raksti");
    strings.insert("Update available", "Pieejams atjauninājums");
    strings.insert(
        "is available. Click to download.",
        "ir pieejams. Noklikšķiniet, lai lejupielādētu.",
    );
    strings.insert("Icon style", "Ikonas stils");
    strings.insert("Number", "Skaitlis");
    strings.insert("Ring", "Gredzens");
    strings.insert("Bar", "Josla");
    strings.insert("Pie", "Sektoru");
    strings.insert("Dashboard layout", "Informācijas paneļa izkārtojums");
    strings.insert("Minimal", "Minimāls");
    strings.insert("Standard", "Standarta");
    strings.insert("Detailed", "Detalizēts");
    strings.insert("Hide Extra Usage", "Slēpt papildu lietojumu");
    strings.insert("Show extra usage", "Rādīt papildu lietojumu");
    strings.insert("Show model limits", "Rādīt modeļu limitus");
    strings.insert("Show startup notification", "Rādīt palaišanas paziņojumu");
    strings.insert(
        "Show login expiry warning",
        "Rādīt pieteikšanās termiņa brīdinājumu",
    );
    strings.insert("Alert thresholds", "Brīdinājumu sliekšņi");
    strings.insert("Notification sound", "Paziņojuma skaņa");
    strings.insert("Test notification", "Testa paziņojums");
    strings.insert("Send", "Sūtīt");
    strings.insert("Pro", "Pro");
    strings.insert("Max", "Max");
    strings.insert("ClaudeMeter", "ClaudeMeter");
    strings.insert("CLAUDE", "CLAUDE");
    strings.insert("CODEX", "CODEX");
    strings.insert(
        "pace_projection",
        "Temps: līdz atiestatīšanai sasniegs ~{}%",
    );
    strings.insert("pace_too_early", "Temps: vēl par agru prognozēt");
    strings.insert("Open Claude Usage", "Atvērt Claude lietojumu");
    strings.insert("Open Config", "Atvērt konfigurāciju");
    strings.insert("Export Config", "Eksportēt konfigurāciju...");
    strings.insert("Import Config", "Importēt konfigurāciju...");
    strings.insert("Enable Autostart", "Ieslēgt automātisko startēšanu");
    strings.insert("Disable Autostart", "Izslēgt automātisko startēšanu");
    strings.insert("Open Logs", "Atvērt žurnālus");
    strings.insert("Quit", "Iziet");
    strings.insert("Freshness", "Datu svaigums");
    strings.insert("Live", "Tiešraidē");
    strings.insert("Cached", "No kešatmiņas / API datu vēl nav");
    strings.insert("api_error", "API kļūda");
    strings.insert("Refreshing", "Atsvaidzina...");
    strings.insert("time_ago", "pirms {}");
    strings.insert("latest_version", "Jums ir jaunākā versija.");
    strings.insert("update_check_failed", "Neizdevās pārbaudīt atjauninājumus.");
    strings
}
