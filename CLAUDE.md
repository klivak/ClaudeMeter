# CLAUDE.md — Project Instructions for AI Assistants

> This file provides context and rules for Claude Code and other AI coding assistants
> working on the ClaudeMeter project.

## Project Summary

ClaudeMeter is an ultra-lightweight Windows system tray application written in Rust that monitors
Claude AI subscription usage limits in real-time. It uses under 10 MB RAM, compiles to a single
portable .exe with zero runtime dependencies, and targets Windows 10/11 (x86_64).

Author: klivak | License: MIT | Repo: github.com/klivak/claudemeter

## Build & Run

```bash
# Development build
cargo build

# Release build (optimized for size)
cargo build --release

# Run (development)
cargo run

# Run release binary directly
./target/release/claudemeter.exe
```

Output binary: `target/release/claudemeter.exe` (~3 MB)

## Lint & Format

```bash
# Format code
cargo fmt

# Lint with Clippy (must pass with zero warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Security audit
cargo audit
```

## Architecture Overview

- `src/main.rs` — Entry point, single-instance mutex, tray init, tokio runtime
- `src/config.rs` — JSON config read/write, defaults, hot-reload
- `src/credentials.rs` — Windows Credential Manager access (CredReadW)
- `src/db.rs` — SQLite history (rusqlite with bundled feature)
- `src/tray.rs` — System tray icon, tooltip, context menu (Win32 API)
- `src/popup.rs` — Dashboard popup window (Win32 API, GDI)
- `src/notifications.rs` — Windows toast notifications (PowerShell-based)
- `src/autostart.rs` — Registry-based auto-start
- `src/theme.rs` — Dark/Light/Auto theme detection from Windows registry
- `src/i18n/` — Localization: en, uk, es, de, fr, pt, ja, ko, zh, it (HashMap-based, compiled in)
- `src/widget.rs` — Mini floating always-on-top widget (Win32 API, GDI)
- `src/updater.rs` — GitHub Release checker, SHA-256 verification, and safe one-click Windows updater
- `src/providers/claude.rs` — Anthropic OAuth Usage API client
- `src/ui/` — Rendering logic, colors, progress bars

## Key Technical Decisions

1. **Native Win32 API only** — No Electron, no webview, no GTK, no Qt. All UI via `windows` crate.
2. **rustls-tls** — No OpenSSL dependency. TLS via rustls for zero native TLS deps.
3. **SQLite bundled** — `rusqlite` with `bundled` feature compiles SQLite into binary.
4. **Single .exe** — No DLLs, no config files required (config auto-created on first run).
5. **Async with tokio** — Minimal features: rt, macros, time, sync, net, io-util.
6. **Future-proof API parsing** — Unknown API fields with valid `{utilization, resets_at}` structure are auto-displayed.
7. **PowerShell notifications** — Toast notifications via PowerShell to avoid winrt-notification dependency.

## Conventions

- **Error handling:** Never panic in release. Use `Result<>` everywhere. Show last known data on API failures.
- **Logging:** Use `log` crate with `env_logger`. Default level: `warn`. Set `RUST_LOG=debug` for verbose.
- **Config:** All user-facing settings in `config.json` next to .exe. No registry for config. No AppData.
- **i18n:** All user-visible strings must go through `t("key")` function. Never hardcode display text.
- **i18n coverage:** Every locale must carry its **own** translation for every key in `en.rs` —
  the English fallback in `t()` is a safety net, never an accepted state. Whenever you add,
  rename, or remove a key in `src/i18n/en.rs`, update **all** locale files in the same change.
  Helpers: `python scripts/i18n_missing.py missing.json` reports per-locale gaps, and
  `scripts/i18n_fill.json` + `python scripts/i18n_fill.py` append new translations in bulk.
  The `test_all_locales_have_full_key_coverage` test in `src/i18n/mod.rs` enforces this — it
  fails the build if any locale is missing a key.
- **Cross-platform gating:** These modules compile on **both** Windows and macOS: `config`, `credentials`,
  `db`, `errors`, `i18n`, `pace`, `providers`, `theme`. Every use of the `windows` crate inside them must sit
  behind `#[cfg(windows)]`, with a `#[cfg(not(windows))]` counterpart when the function is called from shared
  code. The Windows-only modules (`ui`, `tray`, `popup`, `widget`, `windows_app`, `autostart`, `updater`,
  `notifications`) have no such restriction. A local `cargo build` cannot catch a violation — the macOS target
  needs a C cross-compiler for bundled SQLite — so the macOS CI job is the check. Before un-gating a module
  (removing its `#[cfg(windows)]` in `main.rs`), grep it for `windows::` first.
- **Theme:** All colors must come from `colors.rs` theme palette. Never hardcode color values in rendering code.
- **Memory:** Keep allocations minimal. No caching of large data structures. Target: under 10 MB RSS.
- **Naming:** snake_case for files/functions, PascalCase for types/structs, SCREAMING_SNAKE for constants.

## API Reference

### Claude Usage API
```
GET https://api.anthropic.com/api/oauth/usage
Authorization: Bearer <oauth_token>
anthropic-beta: oauth-2025-04-20
```
Returns JSON with keys like `five_hour`, `seven_day`, `seven_day_sonnet`, `seven_day_opus`.
Each non-null key has `{utilization: f64, resets_at: Option<String>}`.

### OAuth Token Location
Windows Credential Manager → target: `"Claude Code-credentials"`
Stored as JSON: `{"claudeAiOauth": {"accessToken": "<oauth-token>"}}`

## Testing Notes

- No test suite yet (GUI-heavy app). Manual testing workflow:
  1. Build release: `cargo build --release`
  2. Run the .exe
  3. Verify tray icon appears
  4. Hover → check tooltip
  5. Left-click → check popup renders correctly
  6. Right-click → check context menu
  7. Change theme/language in settings → verify immediate effect
  8. Kill network → verify graceful fallback ("Last updated X min ago")
- CI runs: clippy, fmt, audit, deny (see `.github/workflows/audit.yml`)

## Release Process

```bash
# 1. Run the version automation script
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bump-version.ps1 5.3.0
# 2. Review CHANGELOG.md and update README.md for user-facing changes
# 3. Commit
git add -A && git commit -m "release: v1.x.x"

# 4. Tag and push — GitHub Actions builds, publishes, and deploys the website automatically
git tag v1.x.x
git push origin main --tags
```

The website's `site/index.html` fallback release value must always match the new tag. The Pages
workflow refreshes `site/metrics.json` from the latest GitHub Release, but the checked-in fallback
must be updated in the same release commit so the site never displays an older version before the
metrics request completes.

**Pre-release checklist:**
- [ ] README.md reflects all new features, settings, config fields, and language count
- [ ] CHANGELOG.md has an entry for the new version
- [ ] Version updated in Cargo.toml, Cargo.lock, and VERSION
- [ ] Website fallback version updated in `site/index.html` and `site/metrics.json`
- [ ] GitHub Pages deployment is triggered after the release push

## Common Pitfalls

- **Win32 tooltip limit:** `NOTIFYICONDATA.szTip` max 128 chars. Truncate gracefully.
- **DPI scaling:** Always use `app.manifest` with PerMonitorV2. Test on 150%/200% scaling.
- **Credential Manager encoding:** `CredReadW` blob may be UTF-16 or UTF-8. Handle both.
- **Single instance:** Named mutex `"ClaudeMeter-SingleInstance"` prevents duplicate processes.
- **Config path:** Use `std::env::current_exe()` parent dir, NOT working directory.
- **PCWSTR vs PWSTR:** Use PCWSTR for read-only string params in Win32 registry/cred APIs.
- **windows crate 0.58:** RegQueryValueExW returns WIN32_ERROR, use `.is_ok()` or `.ok()`.
