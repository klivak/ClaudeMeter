use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut strings = super::en::strings();
    strings.insert("5-hour session", "Sesija od 5 sati");
    strings.insert("Weekly (7-day)", "Tjedno (7 dana)");
    strings.insert("Opus (7-day)", "Opus (7 dana)");
    strings.insert("Sonnet (7-day)", "Sonnet (7 dana)");
    strings.insert("OAuth Apps (7-day)", "OAuth aplikacije (7 dana)");
    strings.insert("resets in", "poništava se za");
    strings.insert("Plan", "Plan");
    strings.insert("Claude Code not detected", "Claude Code nije otkriven");
    strings.insert("credentials_not_found", "Vjerodajnice nisu pronađene");
    strings.insert("connection_error", "Pogreška veze");
    strings.insert("token_expired", "Token je istekao");
    strings.insert("stale_token_expired", "Zastarjelo — token je istekao");
    strings.insert("stale_data", "Zastarjelo — zadnji poznati podaci");
    strings.insert(
        "token_expired_desc",
        "Vaš OAuth token je istekao. Pokrenite `claude login` u terminalu da biste ga obnovili.",
    );
    strings.insert("rate_limited", "Dosegnuto ograničenje zahtjeva");
    strings.insert("server_error", "Pogreška poslužitelja");
    strings.insert(
        "server_error_desc",
        "Anthropic API privremeno nije dostupan. Pokušaj će se automatski ponoviti.",
    );
    strings.insert("run_claude_login_desc", "Claude Code je instaliran, ali niste prijavljeni. Pokrenite `claude login` u terminalu da biste povezali račun.");
    strings.insert(
        "install_claude_desc",
        "Instalirajte Claude Code i pokrenite `claude login` za automatsko praćenje potrošnje.",
    );
    strings.insert("Install Claude Code →", "Instaliraj Claude Code →");
    strings.insert(
        "codex_no_data",
        "Nema aktivnog prozora ograničenja za Codex. Potrošnja se čita iz lokalnih zapisa Codex sesija \u{2014} pokrenite ili nastavite sesiju i pojavit će se ovdje.",
    );
    strings.insert("Check your usage manually:", "Ručno provjerite potrošnju:");
    strings.insert("Open Codex Usage →", "Otvori potrošnju Codexa →");
    strings.insert("Refresh Now", "Osvježi sada");
    strings.insert("Open Dashboard", "Otvori nadzornu ploču");
    strings.insert("Export History (CSV)", "Izvezi povijest (CSV)");
    strings.insert("Export History (JSON)", "Izvezi povijest (JSON)");
    strings.insert("Settings", "Postavke");
    strings.insert("Start with Windows", "Pokreni sa sustavom Windows");
    strings.insert("About", "O programu");
    strings.insert("Exit", "Izlaz");
    strings.insert("Last updated:", "Zadnje ažuriranje:");
    strings.insert("Refresh", "Osvježi");
    strings.insert("Status", "Status");
    strings.insert("Usage Alert", "Upozorenje o potrošnji");
    strings.insert("Usage Critical", "Kritična potrošnja");
    strings.insert(
        "Running in system tray. Click the icon for details.",
        "Aplikacija radi u sistemskoj traci. Kliknite ikonu za detalje.",
    );
    strings.insert("Compact mode", "Kompaktni način");
    strings.insert("Theme", "Tema");
    strings.insert("Language", "Jezik");
    strings.insert("Notifications", "Obavijesti");
    strings.insert("Dark", "Tamna");
    strings.insert("Light", "Svijetla");
    strings.insert("Auto", "Automatski");
    strings.insert("Midnight", "Ponoć");
    strings.insert("Sunset", "Zalazak sunca");
    strings.insert("Show Codex section", "Prikaži odjeljak Codex");
    strings.insert(
        "Reopen the tray popup to refresh",
        "Ponovno otvorite prozor iz sistemske trake za osvježavanje",
    );
    strings.insert("Enabled", "Omogućeno");
    strings.insert("Sound", "Zvuk");
    strings.insert("Thresholds", "Pragovi");
    strings.insert("Polling interval", "Interval provjere");
    strings.insert("seconds", "sekundi");
    strings.insert("Startup", "Pokretanje");
    strings.insert("General", "Općenito");
    strings.insert("Back", "← Natrag");
    strings.insert("Open Claude.ai →", "Otvori Claude.ai →");
    strings.insert("Usage link icons", "Ikone poveznica potrošnje");
    strings.insert("Open usage", "Otvori potrošnju");
    strings.insert("Service status", "Status usluge");
    strings.insert("Usage History", "Povijest potrošnje");
    strings.insert("Usage History (24h)", "Povijest potrošnje (24 h)");
    strings.insert("Auto (English)", "Automatski (Hrvatski)");
    strings.insert("at", "u");
    strings.insert("Resets in", "Poništava se za");
    strings.insert("Tray icon colors:", "Boje ikone u sistemskoj traci:");
    strings.insert("< 50% usage", "< 50% potrošnje");
    strings.insert("50-79% usage", "50–79% potrošnje");
    strings.insert(">= 80% usage", "≥ 80% potrošnje");
    strings.insert("No data", "Nema podataka");
    strings.insert("exceeded", "prekoračeno");
    strings.insert("Show widget", "Prikaži widget");
    strings.insert("Check for updates", "Provjeri ažuriranja");
    strings.insert("Accessibility patterns", "Uzorci pristupačnosti");
    strings.insert("Update available", "Dostupno ažuriranje");
    strings.insert(
        "is available. Click to download.",
        "je dostupno. Kliknite za preuzimanje.",
    );
    strings.insert("Icon style", "Stil ikone");
    strings.insert("Number", "Broj");
    strings.insert("Ring", "Prsten");
    strings.insert("Bar", "Traka");
    strings.insert("Pie", "Tortni");
    strings.insert("Dashboard layout", "Izgled nadzorne ploče");
    strings.insert("Minimal", "Minimalni");
    strings.insert("Standard", "Standardni");
    strings.insert("Detailed", "Detaljni");
    strings.insert("Hide Extra Usage", "Sakrij dodatnu potrošnju");
    strings.insert("Show extra usage", "Prikaži dodatnu potrošnju");
    strings.insert("Show model limits", "Prikaži ograničenja modela");
    strings.insert(
        "Show startup notification",
        "Prikaži obavijest pri pokretanju",
    );
    strings.insert(
        "Show login expiry warning",
        "Prikaži upozorenje o isteku prijave",
    );
    strings.insert("Alert thresholds", "Pragovi upozorenja");
    strings.insert("Notification sound", "Zvuk obavijesti");
    strings.insert("Test notification", "Probna obavijest");
    strings.insert("Send", "Pošalji");
    strings.insert("Pro", "Pro");
    strings.insert("Max", "Max");
    strings.insert("ClaudeMeter", "ClaudeMeter");
    strings.insert("CLAUDE", "CLAUDE");
    strings.insert("CODEX", "CODEX");
    strings
}
