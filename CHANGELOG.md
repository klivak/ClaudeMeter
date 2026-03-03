# Changelog

All notable changes to ClaudeMeter will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.1] - 2026-03-03

### Fixed
- **Single notification per threshold jump** — when usage crosses multiple thresholds at once (e.g., 0% → 96%), only the highest applicable threshold fires a notification instead of all three

## [1.6.0] - 2026-03-03

### Added
- **Configurable tray icon style** — choose between Number (default), Ring (circular progress), or Bar (vertical fill) via Settings or `tray_icon_style` in config.json
- **i18n for icon style** — "Icon style", "Number", "Ring", "Bar" translated in all 15 languages
- **Missing i18n keys for Ukrainian** — added Show widget, Check for updates, Accessibility patterns, Update available translations

### Changed
- Settings panel expanded from 8 to 9 rows (added icon style selector)

## [1.5.0] - 2026-03-03

### Added
- **5 new languages** — Hindi, Turkish, Dutch, Polish, Vietnamese (15 languages total)
- **Notification screenshot** in README for Smart Notifications section

### Changed
- Language cycling now covers all 15 languages in order

## [1.4.0] - 2026-03-03

### Added
- **5 new languages** — Portuguese, Japanese, Korean, Chinese (Simplified), Italian (10 languages total)
- **Better language picker** — settings panel shows full native language name (e.g. "Português", "日本語") instead of language code
- **Mini floating widget** — always-on-top PiP window showing current usage % with color-coded background. Draggable, click to open dashboard. Disabled by default; enable in Settings
- **Auto-update checker** — checks GitHub Releases on startup for newer versions, shows balloon notification if available. Enabled by default; toggle in Settings
- **Accessibility patterns** — colorblind-friendly overlays on progress bars: dots (green), diagonal stripes (yellow), cross-hatch (red). Disabled by default; enable in Settings
- **Custom color themes** — override any theme color via `custom_colors` in config.json (hex values like `"#ff0000"`)

### Changed
- Settings panel expanded from 5 to 8 rows (added widget toggle, update checker toggle, accessibility patterns toggle)
- Language cycling now covers all 10 languages in order

## [1.3.6] - 2026-03-03

### Added
- **VirusTotal scanning** — every release binary is automatically scanned by VirusTotal (60+ antivirus engines) with a link to the full report in release notes
- **VirusTotal badge** in README for transparency and trust

## [1.3.0] - 2026-03-03

### Added
- **Claude Status link** — "Status" link on the Claude section header opens https://status.claude.com/

### Fixed
- **Tray icon shows session %, not weekly max** — when a 5-hour session is active, the tray icon now shows the session utilization (e.g. 7%) instead of the weekly maximum (e.g. 41%)
- **Tray icon shows "..." when no active session** — instead of showing the weekly limit number, the icon displays "..." when no 5-hour session is running
- **Chart no longer shows phantom activity** — usage history chart now filters out records with no active session; old invalid data is cleaned up on startup
- **Tray icon text readability** — white text on green background (was black, hard to read)
- **Notifications use native balloon tips** — replaced unreliable PowerShell toast notifications with Win32 balloon tips that always show both title and body text

## [1.2.0] - 2026-03-03

### Added
- **Dynamic tray icon with % number** — shows actual utilization percentage on the tray icon
- **Gradient progress bars** — smooth color gradients on metric bars
- **Animated progress bars** — bars smoothly fill on popup open (~60fps lerp)
- **Popup fade-in animation** — smooth opacity transition when opening dashboard
- **Chart bar hover tooltip** — hover chart bars to see exact % and time
- **CSV export** — export full usage history from context menu
- **Mica backdrop** — Windows 11 translucent Mica effect on popup
- **Keyboard shortcuts** — ESC to close popup, F5 to refresh
- **Notification sound** — system beep with notifications (configurable)
- **Informative notifications** — shows metric name, current %, exceeded threshold, and reset time
- **Startup notification** — "Running in system tray" toast on launch
- **Auto-refresh on popup open** — triggers poll if data is older than 60 seconds
- **Tray icon blink** — icon blinks when usage exceeds 90% until popup is opened
- **Idle detection** — pauses API polling when PC is idle for 5+ minutes (saves bandwidth)
- **Retry with exponential backoff** — on API errors, poll interval doubles (up to 10 min cap)
- **Rate-limit (429) handling** — graceful retry-after parsing for Anthropic API
- **Config validation** — sanitizes polling interval, thresholds, theme, and language on load

### Fixed
- PowerShell notification window no longer flashes on startup (CREATE_NO_WINDOW flag)
- Tray icon text contrast — black text on green/yellow, white text on red/gray for readability

## [1.1.0] - 2026-03-02

### Added
- Direct2D + DirectWrite hardware-accelerated rendering (replaces GDI)
- DWM dark title bar integration
- Session reset lines (dashed vertical) on 24h usage chart
- Tooltip spacing between metric values
- Screenshots in README (dashboard, tooltip, settings)

### Fixed
- DPI scaling at 125%/150% — popup no longer clips content
- Memory reclaimed when popup is closed (D2D resources released + working set trimmed)
- Settings gear and close button visibility at non-100% DPI
- Credential error display improvements

## [1.0.0] - 2026-02-25

### Added
- Initial release
- Claude usage monitoring (5-hour, 7-day, Sonnet, Opus + dynamic metrics)
- Auto-detection of Claude Pro/Max plan
- Future-proof API parsing (unknown metrics auto-displayed)
- OAuth token retrieval from Windows Credential Manager
- System tray with dynamic color-coded icons (green/yellow/red/gray)
- Rich tooltip with full usage summary on hover
- Dashboard popup with progress bars and countdown timers
- ChatGPT/Codex info section with link to usage page (hidden by default)
- 24-hour usage history chart from SQLite database
- Windows toast notifications at configurable thresholds (50%, 75%, 90%)
- Auto-start with Windows (registry + batch script)
- Compact mode toggle
- Theme: Dark / Light / Auto (follows Windows system theme)
- Languages: English, Українська, Español, Deutsch, Français
- Auto language detection from Windows settings
- Portable config.json next to .exe
- Single .exe, zero dependencies, under 10 MB RAM
- Built with Rust for minimal memory footprint
