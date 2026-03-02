<div align="center">

# ⚡ ClaudeMeter

**Real-time Claude AI subscription usage monitor for Windows**

Ultra-lightweight system tray app built in Rust.
Track your Claude Pro/Max limits without opening a browser.

**🦀 Purposefully built in Rust — uses under 10 MB RAM. Less than Notepad.**

[![Build](https://github.com/klivak/claudemeter/actions/workflows/build.yml/badge.svg)](https://github.com/klivak/claudemeter/actions/workflows/build.yml)
[![Audit](https://github.com/klivak/claudemeter/actions/workflows/audit.yml/badge.svg)](https://github.com/klivak/claudemeter/actions/workflows/audit.yml)
[![Release](https://img.shields.io/github/v/release/klivak/claudemeter)](https://github.com/klivak/claudemeter/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D6?logo=windows)](https://github.com/klivak/claudemeter/releases)
[![RAM](https://img.shields.io/badge/RAM-under%2010MB-brightgreen)](#-why-rust)

[Download](#-quick-start) · [Features](#-features) · [Usage](#-how-to-use) · [FAQ](#-faq)

</div>

---

## 🤔 Why ClaudeMeter?

Tired of hitting Claude usage limits mid-conversation? ClaudeMeter sits quietly in your Windows system tray and shows you **exactly** how much quota you have left — 5-hour session, weekly limits, Sonnet and Opus caps — all without opening a browser tab.

## 🦀 Why Rust?

ClaudeMeter is **purposefully built in Rust** to be as lightweight as physically possible. While most similar tools use Electron (which bundles an entire Chromium browser) or Python (which needs a runtime), ClaudeMeter compiles to a single native Windows binary with zero dependencies.

| App | RAM Usage | Binary Size | Dependencies |
|-----|-----------|-------------|-------------|
| **ClaudeMeter (Rust)** | **3–8 MB** | **~3 MB** | **None** |
| Windows Notepad | ~10 MB | built-in | — |
| Electron-based tray apps | 80–150 MB | ~80 MB | Chromium |
| Python-based monitors | 25–45 MB | ~15 MB | Python runtime |
| .NET-based monitors | 15–25 MB | ~1 MB | .NET runtime |

**Single portable `.exe`** — no installation, no runtime, no .NET, no Java, no Python, no Node.js. Download → run → done.

## ⬇ Quick Start

### Step 1: Install Claude Code (one-time)

ClaudeMeter reads your Claude credentials automatically. You need [Claude Code](https://claude.ai/download) installed and logged in:

```bash
# Install Claude Code (if not already)
# Download from https://claude.ai/download

# Log in (creates OAuth token that ClaudeMeter will use)
claude
```

### Step 2: Download & Run ClaudeMeter

1. **Download** [`claudemeter.exe`](https://github.com/klivak/claudemeter/releases/latest) from Releases
2. **Place** it anywhere — Desktop, tools folder, USB drive (it's portable)
3. **Double-click** to run
4. **Look** for the colored circle icon in your system tray (bottom-right near the clock)

That's it! No configuration needed. ClaudeMeter auto-detects your plan and starts monitoring.

### Step 3 (Optional): Enable Auto-Start

Right-click the tray icon → check ✅ **"Start with Windows"**

## ✨ Features

### Claude AI Monitoring (Automatic)

| Metric | Description |
|--------|-------------|
| 5-hour session | Rolling session utilization with countdown timer |
| 7-day weekly | Weekly usage cap with reset timer |
| 7-day Sonnet | Sonnet-specific limit (shown if applicable) |
| 7-day Opus | Opus-specific limit (Max plans only) |
| Plan detection | Automatically detects Pro vs Max subscription |
| Future metrics | Any new API fields are auto-displayed |

### ChatGPT / Codex (Optional)

OpenAI does not provide a public API for checking ChatGPT Plus/Pro subscription usage. ClaudeMeter includes an optional panel (disabled by default) with a direct link to your ChatGPT usage page. Enable it in Settings if you want quick access.

### System Tray

- **🟢🟡🔴 Dynamic icon** — color changes based on highest usage percentage
- **💬 Rich tooltip** — hover to see all metrics at a glance
- **📋 Context menu** — right-click for quick actions and links
- **📊 Dashboard** — left-click to open the detailed popup

### 🎨 Themes

- **Dark** — easy on the eyes (Catppuccin Mocha palette)
- **Light** — for bright environments (Catppuccin Latte palette)
- **Auto** (default) — follows your Windows system theme automatically

### 🌐 Languages

- 🇬🇧 English (default)
- 🇺🇦 Українська
- 🇪🇸 Español
- 🇩🇪 Deutsch
- 🇫🇷 Français

### 🔔 Smart Notifications

Windows toast notifications when usage crosses configurable thresholds (50%, 75%, 90% by default).

## ⚙ Configuration

`config.json` is auto-created next to the `.exe` on first launch:

```json
{
  "version": "1.0.0",
  "polling_interval_seconds": 120,
  "notifications": {
    "enabled": true,
    "thresholds": [50, 75, 90],
    "sound": true
  },
  "autostart": false,
  "compact_mode": false,
  "theme": "auto",
  "language": "auto",
  "show_chatgpt_section": false
}
```

## 🔨 Building from Source

```bash
git clone https://github.com/klivak/claudemeter.git
cd claudemeter
cargo build --release
# Output: target/release/claudemeter.exe (~3 MB)
```

**Requirements:** Rust 1.75+ and Windows SDK (included with [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)).

## 🔑 How Authentication Works

ClaudeMeter does **not** ask for your password or API key. It reuses the OAuth token that [Claude Code](https://claude.ai/download) already stores on your machine.

**Token lookup order:**

| # | Location | Used by |
|---|----------|---------|
| 1 | `~/.claude/.credentials.json` | Claude Code v2.x+ |
| 2 | Windows Credential Manager (`Claude Code-credentials`) | Claude Code v1.x (legacy) |

When you run `claude` and log in via the browser, Claude Code saves an OAuth token to `~/.claude/.credentials.json`. ClaudeMeter reads this file to authenticate with the Anthropic Usage API — no extra setup needed.

**What's stored in the file:**

```json
{
  "claudeAiOauth": {
    "accessToken": "sk-ant-oat01-...",
    "refreshToken": "sk-ant-ort01-...",
    "expiresAt": 1772467364905,
    "subscriptionType": "max"
  }
}
```

ClaudeMeter uses `accessToken` to fetch your usage data and `subscriptionType` to display your plan (Pro/Max). It never modifies this file.

> **Troubleshooting:** If ClaudeMeter shows "Credentials not found", run `claude` in a terminal and log in. Then click Refresh in ClaudeMeter.

## ❓ FAQ

**Q: Does it work without Claude Code installed?**
A: ClaudeMeter launches but shows a "Credentials not found" message with a link to claude.ai. You need Claude Code logged in so ClaudeMeter can read the OAuth token from `~/.claude/.credentials.json`.

**Q: How much RAM does it really use?**
A: Typically **3–8 MB**. Built in Rust with native Win32 API — no Electron, no browser engine.

**Q: Is it safe? Does it send my data anywhere?**
A: ClaudeMeter is fully open source. It only communicates with `api.anthropic.com` to fetch YOUR usage data using YOUR existing OAuth token. Zero telemetry.

**Q: Why isn't ChatGPT tracking automatic?**
A: OpenAI deliberately does not expose ChatGPT subscription usage via any public API.

## 📄 License

[MIT](LICENSE) — free for personal and commercial use.

---

<div align="center">

**🦀 Purposefully built in Rust for minimal footprint and maximum reliability**
**3–8 MB RAM · Single .exe · Zero dependencies · Open source**

Made by [klivak](https://github.com/klivak)

*Claude is a trademark of Anthropic. ChatGPT is a trademark of OpenAI.*
*ClaudeMeter is an independent open-source project with no official affiliation.*

</div>

<!-- SEO keywords: claude usage monitor, claude ai usage tracker, claude limits monitor windows, claude pro usage limits tracker, claude max usage monitor, anthropic claude usage, claude code usage monitor windows, claude subscription limits, claude tray app windows, rust windows tray application -->
