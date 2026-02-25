# Changelog

All notable changes to ClaudeMeter will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
