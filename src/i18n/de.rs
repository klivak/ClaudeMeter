use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("5-hour session", "5-Stunden-Sitzung");
    m.insert("Weekly (7-day)", "Wöchentlich (7 Tage)");
    m.insert("Opus (7-day)", "Opus (7 Tage)");
    m.insert("Sonnet (7-day)", "Sonnet (7 Tage)");
    m.insert("OAuth Apps (7-day)", "OAuth Apps (7 Tage)");
    m.insert("resets in", "setzt zurück in");
    m.insert("Plan", "Plan");
    m.insert("Pro", "Pro");
    m.insert("Max", "Max");
    m.insert("Claude Code not detected", "Claude Code nicht erkannt");
    m.insert("credentials_not_found", "Anmeldedaten nicht gefunden");
    m.insert(
        "run_claude_login_desc",
        "Claude Code ist installiert, aber nicht angemeldet. Führen Sie `claude login` im Terminal aus, um Ihr Konto zu verbinden.",
    );
    m.insert(
        "install_claude_desc",
        "Installieren Sie Claude Code und führen Sie `claude login` aus, um die automatische Nutzungsverfolgung zu aktivieren.",
    );
    m.insert(
        "Install Claude Code \u{2192}",
        "Claude Code installieren \u{2192}",
    );
    m.insert(
        "openai_no_api",
        "OpenAI stellt keine API zur Verfügung, um die ChatGPT-Abonnementnutzung zu verfolgen.",
    );
    m.insert(
        "Check your usage manually:",
        "Überprüfen Sie Ihre Nutzung manuell:",
    );
    m.insert(
        "Open ChatGPT Usage \u{2192}",
        "ChatGPT-Nutzung öffnen \u{2192}",
    );
    m.insert("Refresh Now", "Jetzt aktualisieren");
    m.insert("Open Dashboard", "Dashboard öffnen");
    m.insert("Export History (CSV)", "Verlauf exportieren (CSV)");
    m.insert("Settings", "Einstellungen");
    m.insert("Start with Windows", "Mit Windows starten");
    m.insert("About", "Über");
    m.insert("Exit", "Beenden");
    m.insert("Last updated:", "Letzte Aktualisierung:");
    m.insert("Refresh", "Aktualisieren");
    m.insert("Status", "Status");
    m.insert("Usage Alert", "Nutzungswarnung");
    m.insert("Usage Critical", "Kritische Nutzung");
    m.insert(
        "Running in system tray. Click the icon for details.",
        "Läuft im Infobereich. Klicken Sie auf das Symbol für Details.",
    );
    m.insert("Compact mode", "Kompaktmodus");
    m.insert("Theme", "Thema");
    m.insert("Language", "Sprache");
    m.insert("Notifications", "Benachrichtigungen");
    m.insert("Dark", "Dunkel");
    m.insert("Light", "Hell");
    m.insert("Auto", "Auto");
    m.insert("Show ChatGPT section", "ChatGPT-Bereich anzeigen");
    m.insert("Enabled", "Aktiviert");
    m.insert("Sound", "Ton");
    m.insert("Thresholds", "Schwellenwerte");
    m.insert("Polling interval", "Aktualisierungsintervall");
    m.insert("seconds", "Sekunden");
    m.insert("Startup", "Autostart");
    m.insert("General", "Allgemein");
    m.insert("Back", "\u{2190} Zurück");
    m.insert("Open Claude.ai \u{2192}", "Claude.ai öffnen \u{2192}");
    m.insert("ClaudeMeter", "ClaudeMeter");
    m.insert("CLAUDE", "CLAUDE");
    m.insert("CHATGPT / CODEX", "CHATGPT / CODEX");
    m.insert("Usage History (24h)", "Nutzungsverlauf (24h)");
    m.insert("Auto (English)", "Auto (Deutsch)");
    m.insert("at", "um");
    m.insert("Resets in", "Zurückgesetzt in");
    m.insert("Tray icon colors:", "Tray-Icon-Farben:");
    m.insert("< 50% usage", "< 50% Nutzung");
    m.insert("50-79% usage", "50\u{2013}79% Nutzung");
    m.insert(">= 80% usage", "\u{2265} 80% Nutzung");
    m.insert("No data", "Keine Daten");
    m.insert("exceeded", "überschritten");
    m
}
