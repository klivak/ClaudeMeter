use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut strings = super::en::strings();
    strings.insert("5-hour session", "Sessió de 5 hores");
    strings.insert("Weekly (7-day)", "Setmanal (7 dies)");
    strings.insert("Opus (7-day)", "Opus (7 dies)");
    strings.insert("Sonnet (7-day)", "Sonnet (7 dies)");
    strings.insert("OAuth Apps (7-day)", "Aplicacions OAuth (7 dies)");
    strings.insert("resets in", "es reinicia d'aquí a");
    strings.insert("Plan", "Pla");
    strings.insert("Claude Code not detected", "Claude Code no detectat");
    strings.insert("credentials_not_found", "No s'han trobat les credencials");
    strings.insert("connection_error", "Error de connexió");
    strings.insert("token_expired", "El token ha caducat");
    strings.insert("stale_token_expired", "Obsolet — el token ha caducat");
    strings.insert("stale_data", "Obsolet — últimes dades conegudes");
    strings.insert(
        "token_expired_desc",
        "El token OAuth ha caducat. Executa `claude login` al terminal per renovar-lo.",
    );
    strings.insert("rate_limited", "Límit de sol·licituds assolit");
    strings.insert("server_error", "Error del servidor");
    strings.insert(
        "server_error_desc",
        "L'API d'Anthropic no està disponible temporalment. Es tornarà a provar automàticament.",
    );
    strings.insert("run_claude_login_desc", "Claude Code està instal·lat però no has iniciat sessió. Executa `claude login` al terminal per connectar el compte.");
    strings.insert("install_claude_desc", "Instal·la Claude Code i executa `claude login` per activar el seguiment automàtic de l'ús.");
    strings.insert("Install Claude Code →", "Instal·la Claude Code →");
    strings.insert("codex_no_data", "No hi ha cap finestra de límits de Codex activa. L'ús es llegeix dels registres locals de sessions de Codex: inicia o continua una sessió i apareixerà aquí.");
    strings.insert("Check your usage manually:", "Consulta l'ús manualment:");
    strings.insert("Open Codex Usage →", "Obre l'ús de Codex →");
    strings.insert("Refresh Now", "Actualitza ara");
    strings.insert("Open Dashboard", "Obre el tauler");
    strings.insert("Export History (CSV)", "Exporta l'historial (CSV)");
    strings.insert("Export History (JSON)", "Exporta l'historial (JSON)");
    strings.insert("Settings", "Configuració");
    strings.insert("Start with Windows", "Inicia amb Windows");
    strings.insert("About", "Quant a");
    strings.insert("Exit", "Surt");
    strings.insert("Last updated:", "Última actualització:");
    strings.insert("Refresh", "Actualitza");
    strings.insert("Status", "Estat");
    strings.insert("Usage Alert", "Avís d'ús");
    strings.insert("Usage Critical", "Ús crític");
    strings.insert(
        "Running in system tray. Click the icon for details.",
        "S'està executant a la safata del sistema. Fes clic a la icona per veure'n els detalls.",
    );
    strings.insert("Compact mode", "Mode compacte");
    strings.insert("Theme", "Tema");
    strings.insert("Language", "Idioma");
    strings.insert("Notifications", "Notificacions");
    strings.insert("Dark", "Fosc");
    strings.insert("Light", "Clar");
    strings.insert("Auto", "Automàtic");
    strings.insert("Midnight", "Mitjanit");
    strings.insert("Sunset", "Posta de sol");
    strings.insert("Show Codex section", "Mostra la secció de Codex");
    strings.insert(
        "Reopen the tray popup to refresh",
        "Torna a obrir la finestra de la safata per actualitzar",
    );
    strings.insert("Enabled", "Activat");
    strings.insert("Sound", "So");
    strings.insert("Thresholds", "Llindars");
    strings.insert("Polling interval", "Interval de consulta");
    strings.insert("seconds", "segons");
    strings.insert("Startup", "Inici");
    strings.insert("General", "General");
    strings.insert("Back", "← Enrere");
    strings.insert("Open Claude.ai →", "Obre Claude.ai →");
    strings.insert("Usage link icons", "Icones d'enllaç d'ús");
    strings.insert("Open usage", "Obre l'ús");
    strings.insert("Service status", "Estat del servei");
    strings.insert("Usage History", "Historial d'ús");
    strings.insert("Usage History (24h)", "Historial d'ús (24 h)");
    strings.insert("Auto (English)", "Automàtic (Català)");
    strings.insert("at", "a les");
    strings.insert("Resets in", "Es reinicia d'aquí a");
    strings.insert("Tray icon colors:", "Colors de la icona de la safata:");
    strings.insert("< 50% usage", "< 50% d'ús");
    strings.insert("50-79% usage", "50–79% d'ús");
    strings.insert(">= 80% usage", "≥ 80% d'ús");
    strings.insert("No data", "Sense dades");
    strings.insert("exceeded", "superat");
    strings.insert("Show widget", "Mostra el giny");
    strings.insert("Check for updates", "Comprova si hi ha actualitzacions");
    strings.insert("Accessibility patterns", "Patrons d'accessibilitat");
    strings.insert("Update available", "Actualització disponible");
    strings.insert(
        "is available. Click to download.",
        "està disponible. Fes clic per baixar-la.",
    );
    strings.insert("Icon style", "Estil de la icona");
    strings.insert("Number", "Número");
    strings.insert("Ring", "Anell");
    strings.insert("Bar", "Barra");
    strings.insert("Pie", "Circular");
    strings.insert("Dashboard layout", "Disseny del tauler");
    strings.insert("Minimal", "Mínim");
    strings.insert("Standard", "Estàndard");
    strings.insert("Detailed", "Detallat");
    strings.insert("Hide Extra Usage", "Amaga l'ús addicional");
    strings.insert("Show extra usage", "Mostra l'ús addicional");
    strings.insert("Show model limits", "Mostra els límits per model");
    strings.insert("Show startup notification", "Mostra la notificació d'inici");
    strings.insert(
        "Show login expiry warning",
        "Mostra l'avís de caducitat de la sessió",
    );
    strings.insert("Alert thresholds", "Llindars d'alerta");
    strings.insert("Notification sound", "So de notificació");
    strings.insert("Test notification", "Notificació de prova");
    strings.insert("Send", "Envia");
    strings.insert("Pro", "Pro");
    strings.insert("Max", "Max");
    strings.insert("ClaudeMeter", "ClaudeMeter");
    strings.insert("CLAUDE", "CLAUDE");
    strings.insert("CODEX", "CODEX");
    strings.insert("pace_projection", "Ritme: cap a ~{}% en reiniciar-se");
    strings.insert("pace_too_early", "Ritme: encara és aviat per projectar");
    strings.insert("Open Claude Usage", "Obre l'ús de Claude");
    strings.insert("Open Config", "Obre la configuració");
    strings.insert("Export Config", "Exporta la configuració...");
    strings.insert("Import Config", "Importa la configuració...");
    strings.insert("Enable Autostart", "Activa l'inici automàtic");
    strings.insert("Disable Autostart", "Desactiva l'inici automàtic");
    strings.insert("Open Logs", "Obre els registres");
    strings.insert("Quit", "Surt");
    strings.insert("Freshness", "Actualitat");
    strings.insert("Live", "En directe");
    strings.insert("Cached", "De la memòria cau / encara sense dades de l'API");
    strings.insert("api_error", "Error de l'API");
    strings.insert("Refreshing", "Actualitzant...");
    strings.insert("time_ago", "fa {}");
    strings.insert("latest_version", "Tens la darrera versió.");
    strings.insert(
        "update_check_failed",
        "No s'han pogut comprovar les actualitzacions.",
    );
    strings
}
