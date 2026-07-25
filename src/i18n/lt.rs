use std::collections::HashMap;

pub fn strings() -> HashMap<&'static str, &'static str> {
    let mut strings = super::en::strings();
    strings.insert("5-hour session", "5 valandų sesija");
    strings.insert("Weekly (7-day)", "Savaitinis (7 dienos)");
    strings.insert("Opus (7-day)", "Opus (7 dienos)");
    strings.insert("Sonnet (7-day)", "Sonnet (7 dienos)");
    strings.insert("OAuth Apps (7-day)", "OAuth programos (7 dienos)");
    strings.insert("resets in", "bus atstatyta po");
    strings.insert("Plan", "Planas");
    strings.insert("Claude Code not detected", "Claude Code neaptiktas");
    strings.insert("credentials_not_found", "Prisijungimo duomenys nerasti");
    strings.insert("connection_error", "Ryšio klaida");
    strings.insert("token_expired", "Prieigos raktas nebegalioja");
    strings.insert(
        "stale_token_expired",
        "Pasenę — prieigos raktas nebegalioja",
    );
    strings.insert("stale_data", "Pasenę — paskutiniai žinomi duomenys");
    strings.insert("token_expired_desc", "OAuth prieigos raktas nebegalioja. Paleiskite `claude login` terminale, kad jį atnaujintumėte.");
    strings.insert("rate_limited", "Pasiektas užklausų limitas");
    strings.insert("server_error", "Serverio klaida");
    strings.insert(
        "server_error_desc",
        "Anthropic API laikinai nepasiekiama. Bandymas bus automatiškai pakartotas.",
    );
    strings.insert("run_claude_login_desc", "Claude Code įdiegtas, bet neprisijungta. Paleiskite `claude login` terminale, kad prijungtumėte paskyrą.");
    strings.insert("install_claude_desc", "Įdiekite Claude Code ir paleiskite `claude login`, kad įjungtumėte automatinį naudojimo stebėjimą.");
    strings.insert("Install Claude Code →", "Įdiegti Claude Code →");
    strings.insert(
        "codex_no_data",
        "Nėra aktyvaus Codex limitų lango. Naudojimas skaitomas iš vietinių Codex sesijų žurnalų \u{2014} pradėkite arba tęskite sesiją ir jis pasirodys čia.",
    );
    strings.insert(
        "Check your usage manually:",
        "Patikrinkite naudojimą rankiniu būdu:",
    );
    strings.insert("Open Codex Usage →", "Atidaryti Codex naudojimą →");
    strings.insert("Refresh Now", "Atnaujinti dabar");
    strings.insert("Open Dashboard", "Atidaryti skydelį");
    strings.insert("Export History (CSV)", "Eksportuoti istoriją (CSV)");
    strings.insert("Export History (JSON)", "Eksportuoti istoriją (JSON)");
    strings.insert("Settings", "Nustatymai");
    strings.insert("Start with Windows", "Paleisti su Windows");
    strings.insert("About", "Apie");
    strings.insert("Exit", "Išeiti");
    strings.insert("Last updated:", "Paskutinį kartą atnaujinta:");
    strings.insert("Refresh", "Atnaujinti");
    strings.insert("Status", "Būsena");
    strings.insert("Usage Alert", "Naudojimo įspėjimas");
    strings.insert("Usage Critical", "Kritinis naudojimas");
    strings.insert(
        "Running in system tray. Click the icon for details.",
        "Veikia sistemos dėkle. Spustelėkite piktogramą, kad peržiūrėtumėte informaciją.",
    );
    strings.insert("Compact mode", "Kompaktiškas režimas");
    strings.insert("Theme", "Tema");
    strings.insert("Language", "Kalba");
    strings.insert("Notifications", "Pranešimai");
    strings.insert("Dark", "Tamsi");
    strings.insert("Light", "Šviesi");
    strings.insert("Auto", "Automatiškai");
    strings.insert("Midnight", "Vidurnaktis");
    strings.insert("Sunset", "Saulėlydis");
    strings.insert("Show Codex section", "Rodyti Codex skiltį");
    strings.insert(
        "Reopen the tray popup to refresh",
        "Atnaujinimui iš naujo atidarykite sistemos dėklo langą",
    );
    strings.insert("Enabled", "Įjungta");
    strings.insert("Sound", "Garsas");
    strings.insert("Thresholds", "Slenksčiai");
    strings.insert("Polling interval", "Tikrinimo intervalas");
    strings.insert("seconds", "sekundės");
    strings.insert("Startup", "Paleidimas");
    strings.insert("General", "Bendra");
    strings.insert("Back", "← Atgal");
    strings.insert("Open Claude.ai →", "Atidaryti Claude.ai →");
    strings.insert("Usage link icons", "Naudojimo nuorodų piktogramos");
    strings.insert("Open usage", "Atidaryti naudojimą");
    strings.insert("Service status", "Paslaugos būsena");
    strings.insert("Usage History", "Naudojimo istorija");
    strings.insert("Usage History (24h)", "Naudojimo istorija (24 val.)");
    strings.insert("Auto (English)", "Automatiškai (Lietuvių)");
    strings.insert("at", "val.");
    strings.insert("Resets in", "Bus atstatyta po");
    strings.insert("Tray icon colors:", "Sistemos dėklo piktogramos spalvos:");
    strings.insert("< 50% usage", "< 50% naudojimo");
    strings.insert("50-79% usage", "50–79% naudojimo");
    strings.insert(">= 80% usage", "≥ 80% naudojimo");
    strings.insert("No data", "Nėra duomenų");
    strings.insert("exceeded", "viršyta");
    strings.insert("Show widget", "Rodyti valdiklį");
    strings.insert("Check for updates", "Tikrinti naujinimus");
    strings.insert("Accessibility patterns", "Prieinamumo raštai");
    strings.insert("Update available", "Galimas naujinimas");
    strings.insert(
        "is available. Click to download.",
        "yra pasiekiamas. Spustelėkite, kad atsisiųstumėte.",
    );
    strings.insert("Icon style", "Piktogramos stilius");
    strings.insert("Number", "Skaičius");
    strings.insert("Ring", "Žiedas");
    strings.insert("Bar", "Juosta");
    strings.insert("Pie", "Skritulinė");
    strings.insert("Dashboard layout", "Skydelio išdėstymas");
    strings.insert("Minimal", "Minimalus");
    strings.insert("Standard", "Standartinis");
    strings.insert("Detailed", "Išsamus");
    strings.insert("Hide Extra Usage", "Slėpti papildomą naudojimą");
    strings.insert("Show extra usage", "Rodyti papildomą naudojimą");
    strings.insert("Show model limits", "Rodyti modelių limitus");
    strings.insert("Show startup notification", "Rodyti paleidimo pranešimą");
    strings.insert(
        "Show login expiry warning",
        "Rodyti prisijungimo galiojimo įspėjimą",
    );
    strings.insert("Alert thresholds", "Įspėjimų slenksčiai");
    strings.insert("Notification sound", "Pranešimo garsas");
    strings.insert("Test notification", "Bandomasis pranešimas");
    strings.insert("Send", "Siųsti");
    strings.insert("Pro", "Pro");
    strings.insert("Max", "Max");
    strings.insert("ClaudeMeter", "ClaudeMeter");
    strings.insert("CLAUDE", "CLAUDE");
    strings.insert("CODEX", "CODEX");
    strings.insert("pace_projection", "Tempas: iki atstatymo pasieks ~{}%");
    strings.insert("pace_too_early", "Tempas: dar per anksti prognozuoti");
    strings.insert("Open Claude Usage", "Atverti Claude naudojimą");
    strings.insert("Open Config", "Atverti konfigūraciją");
    strings.insert("Export Config", "Eksportuoti konfigūraciją...");
    strings.insert("Import Config", "Importuoti konfigūraciją...");
    strings.insert("Enable Autostart", "Įjungti automatinį paleidimą");
    strings.insert("Disable Autostart", "Išjungti automatinį paleidimą");
    strings.insert("Open Logs", "Atverti žurnalus");
    strings.insert("Quit", "Išeiti");
    strings.insert("Freshness", "Duomenų šviežumas");
    strings.insert("Live", "Tiesiogiai");
    strings.insert("Cached", "Iš podėlio / API duomenų dar nėra");
    strings.insert("api_error", "API klaida");
    strings.insert("Refreshing", "Atnaujinama...");
    strings.insert("time_ago", "prieš {}");
    strings.insert("latest_version", "Naudojate naujausią versiją.");
    strings.insert("update_check_failed", "Nepavyko patikrinti atnaujinimų.");
    strings
}
