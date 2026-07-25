use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut strings = super::en::strings();
    strings.insert("5-hour session", "5-tunnine seanss");
    strings.insert("Weekly (7-day)", "Nädalane (7 päeva)");
    strings.insert("Opus (7-day)", "Opus (7 päeva)");
    strings.insert("Sonnet (7-day)", "Sonnet (7 päeva)");
    strings.insert("OAuth Apps (7-day)", "OAuthi rakendused (7 päeva)");
    strings.insert("resets in", "lähtestub aja pärast");
    strings.insert("Plan", "Plaan");
    strings.insert("Claude Code not detected", "Claude Code'i ei tuvastatud");
    strings.insert("credentials_not_found", "Kasutajatunnuseid ei leitud");
    strings.insert("connection_error", "Ühenduse viga");
    strings.insert("token_expired", "Luba on aegunud");
    strings.insert("stale_token_expired", "Aegunud — luba on aegunud");
    strings.insert("stale_data", "Aegunud — viimased teadaolevad andmed");
    strings.insert(
        "token_expired_desc",
        "OAuthi luba on aegunud. Selle uuendamiseks käivita terminalis `claude login`.",
    );
    strings.insert("rate_limited", "Päringulimiit ületatud");
    strings.insert("server_error", "Serveri viga");
    strings.insert(
        "server_error_desc",
        "Anthropicu API pole ajutiselt saadaval. Uuesti proovitakse automaatselt.",
    );
    strings.insert("run_claude_login_desc", "Claude Code on installitud, kuid sa pole sisse logitud. Konto ühendamiseks käivita terminalis `claude login`.");
    strings.insert(
        "install_claude_desc",
        "Automaatse kasutusjälgimise lubamiseks installi Claude Code ja käivita `claude login`.",
    );
    strings.insert("Install Claude Code →", "Installi Claude Code →");
    strings.insert(
        "codex_no_data",
        "Aktiivset Codexi limiidiakent pole. Kasutus loetakse kohalikest Codexi seansilogidest \u{2014} alusta või jätka seanssi ja see ilmub siia.",
    );
    strings.insert("Check your usage manually:", "Kontrolli kasutust käsitsi:");
    strings.insert("Open Codex Usage →", "Ava Codexi kasutus →");
    strings.insert("Refresh Now", "Värskenda kohe");
    strings.insert("Open Dashboard", "Ava juhtpaneel");
    strings.insert("Export History (CSV)", "Ekspordi ajalugu (CSV)");
    strings.insert("Export History (JSON)", "Ekspordi ajalugu (JSON)");
    strings.insert("Settings", "Seaded");
    strings.insert("Start with Windows", "Käivita koos Windowsiga");
    strings.insert("About", "Teave");
    strings.insert("Exit", "Välju");
    strings.insert("Last updated:", "Viimati uuendatud:");
    strings.insert("Refresh", "Värskenda");
    strings.insert("Status", "Olek");
    strings.insert("Usage Alert", "Kasutuse hoiatus");
    strings.insert("Usage Critical", "Kriitiline kasutus");
    strings.insert(
        "Running in system tray. Click the icon for details.",
        "Töötab süsteemisalves. Üksikasjade vaatamiseks klõpsa ikoonil.",
    );
    strings.insert("Compact mode", "Kompaktrežiim");
    strings.insert("Theme", "Teema");
    strings.insert("Language", "Keel");
    strings.insert("Notifications", "Teavitused");
    strings.insert("Dark", "Tume");
    strings.insert("Light", "Hele");
    strings.insert("Auto", "Automaatne");
    strings.insert("Midnight", "Kesköö");
    strings.insert("Sunset", "Päikeseloojang");
    strings.insert("Show Codex section", "Kuva Codexi jaotis");
    strings.insert(
        "Reopen the tray popup to refresh",
        "Värskendamiseks ava süsteemisalve aken uuesti",
    );
    strings.insert("Enabled", "Lubatud");
    strings.insert("Sound", "Heli");
    strings.insert("Thresholds", "Lävendid");
    strings.insert("Polling interval", "Kontrollimise intervall");
    strings.insert("seconds", "sekundit");
    strings.insert("Startup", "Käivitamine");
    strings.insert("General", "Üldine");
    strings.insert("Back", "← Tagasi");
    strings.insert("Open Claude.ai →", "Ava Claude.ai →");
    strings.insert("Usage link icons", "Kasutuslinkide ikoonid");
    strings.insert("Open usage", "Ava kasutus");
    strings.insert("Service status", "Teenuse olek");
    strings.insert("Usage History", "Kasutusajalugu");
    strings.insert("Usage History (24h)", "Kasutusajalugu (24 h)");
    strings.insert("Auto (English)", "Automaatne (Eesti)");
    strings.insert("at", "kell");
    strings.insert("Resets in", "Lähtestub aja pärast");
    strings.insert("Tray icon colors:", "Süsteemisalve ikooni värvid:");
    strings.insert("< 50% usage", "< 50% kasutust");
    strings.insert("50-79% usage", "50–79% kasutust");
    strings.insert(">= 80% usage", "≥ 80% kasutust");
    strings.insert("No data", "Andmed puuduvad");
    strings.insert("exceeded", "ületatud");
    strings.insert("Show widget", "Kuva vidin");
    strings.insert("Check for updates", "Kontrolli uuendusi");
    strings.insert("Accessibility patterns", "Hõlbustusmustrid");
    strings.insert("Update available", "Uuendus on saadaval");
    strings.insert(
        "is available. Click to download.",
        "on saadaval. Allalaadimiseks klõpsa.",
    );
    strings.insert("Icon style", "Ikooni stiil");
    strings.insert("Number", "Number");
    strings.insert("Ring", "Ring");
    strings.insert("Bar", "Riba");
    strings.insert("Pie", "Sektor");
    strings.insert("Dashboard layout", "Juhtpaneeli paigutus");
    strings.insert("Minimal", "Minimaalne");
    strings.insert("Standard", "Standardne");
    strings.insert("Detailed", "Üksikasjalik");
    strings.insert("Hide Extra Usage", "Peida lisakasutus");
    strings.insert("Show extra usage", "Kuva lisakasutus");
    strings.insert("Show model limits", "Kuva mudelite piirid");
    strings.insert("Show startup notification", "Kuva käivitusteavitus");
    strings.insert(
        "Show login expiry warning",
        "Kuva sisselogimise aegumise hoiatus",
    );
    strings.insert("Alert thresholds", "Hoiatuse lävendid");
    strings.insert("Notification sound", "Teavituse heli");
    strings.insert("Test notification", "Testteavitus");
    strings.insert("Send", "Saada");
    strings.insert("Pro", "Pro");
    strings.insert("Max", "Max");
    strings.insert("ClaudeMeter", "ClaudeMeter");
    strings.insert("CLAUDE", "CLAUDE");
    strings.insert("CODEX", "CODEX");
    strings
}
