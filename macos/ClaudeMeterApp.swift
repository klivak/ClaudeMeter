import AppKit
import Foundation
import UniformTypeIdentifiers

struct Metric {
    let key: String
    let name: String
    let percent: Int
    let resetsAt: String?
}

struct Status {
    let state: String
    let title: String
    let detail: String
    let percent: Int?
    let metrics: [Metric]
    let tierNote: String?
    let lastApiUpdate: String?
    let error: String?
    // 24h usage history, 48 buckets oldest-first (0 = 24h ago, 47 = now), 0-100.
    let chart: [Int]
    // Past 5-hour session reset points as "hours ago" (0-24), for dashed lines.
    let chartResets: [Double]
    // Localized menu strings and current setting values, published by the Rust
    // agent. Missing keys fall back to the English literals below.
    let labels: [String: String]

    /// Localized string for `key`, or `fallback` when the agent has not
    /// published it (older agent, or first launch before the first poll).
    func label(_ key: String, _ fallback: String) -> String {
        let value = labels[key]
        return (value?.isEmpty == false) ? value! : fallback
    }

    /// Boolean setting value published alongside the labels.
    func flag(_ key: String) -> Bool {
        labels["value_" + key] == "true"
    }

    static let loading = Status(
        state: "refreshing",
        title: "...",
        detail: "Starting ClaudeMeter",
        percent: nil,
        metrics: [],
        tierNote: nil,
        lastApiUpdate: nil,
        error: nil,
        chart: [],
        chartResets: [],
        labels: [:]
    )
}

@main
enum ClaudeMeterMain {
    // Strong reference: NSApplication.delegate is weak, so the delegate must be
    // retained for the lifetime of the run loop. main() never returns, so this
    // stack-held reference keeps it alive.
    static func main() {
        let app = NSApplication.shared
        let delegate = AppDelegate()
        app.delegate = delegate
        app.setActivationPolicy(.accessory)
        app.run()
        _ = delegate
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    private var menu = NSMenu()
    private var agent: Process?
    private var timer: Timer?
    private var currentStatus = Status.loading

    private let appSupport: URL = {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        return base.appendingPathComponent("ClaudeMeter", isDirectory: true)
    }()

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        try? FileManager.default.createDirectory(at: appSupport, withIntermediateDirectories: true)
        startAgent()
        reloadStatus()
        timer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { [weak self] _ in
            self?.reloadStatus()
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
        agent?.terminate()
    }

    private func startAgent() {
        guard agent?.isRunning != true else { return }
        // Kill any orphaned agents left behind by a prior app instance or a
        // LaunchAgent relaunch. Without this they keep polling forever, and the
        // combined poll rate from stacked agents trips the API rate limiter.
        killOrphanedAgents()
        let process = Process()
        process.executableURL = agentURL()
        process.arguments = ["--agent"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        do {
            try process.run()
            agent = process
        } catch {
            writeLog("Failed to start agent: \(error)")
        }
    }

    private func killOrphanedAgents() {
        let pkill = Process()
        pkill.executableURL = URL(fileURLWithPath: "/usr/bin/pkill")
        // Match the full command line so we only target the polling agent.
        pkill.arguments = ["-f", "claudemeter-agent --agent"]
        pkill.standardOutput = FileHandle.nullDevice
        pkill.standardError = FileHandle.nullDevice
        do {
            try pkill.run()
            pkill.waitUntilExit()
        } catch {
            writeLog("Failed to kill orphaned agents: \(error)")
        }
    }

    private func refreshNow() {
        currentStatus = Status(state: "refreshing", title: "...", detail: currentStatus.label("refreshing", "Refreshing..."), percent: nil, metrics: [], tierNote: nil, lastApiUpdate: nil, error: nil, chart: currentStatus.chart, chartResets: currentStatus.chartResets, labels: currentStatus.labels)
        renderMenu()

        DispatchQueue.global(qos: .utility).async {
            let process = Process()
            process.executableURL = self.agentURL()
            process.arguments = ["--once"]
            process.standardOutput = FileHandle.nullDevice
            process.standardError = FileHandle.nullDevice
            do {
                try process.run()
                process.waitUntilExit()
            } catch {
                self.writeLog("Manual refresh failed: \(error)")
            }
            DispatchQueue.main.async {
                self.reloadStatus()
            }
        }
    }

    private func reloadStatus() {
        let path = appSupport.appendingPathComponent("status.json")
        guard let data = try? Data(contentsOf: path),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            currentStatus = Status.loading
            renderMenu()
            return
        }

        var metrics: [Metric] = []
        if let rawMetrics = object["metrics"] as? [[String: Any]] {
            for entry in rawMetrics {
                guard let name = entry["name"] as? String,
                      let percent = entry["percent"] as? Int else { continue }
                let key = entry["key"] as? String ?? ""
                let resetsAt = entry["resets_at"] as? String
                metrics.append(Metric(key: key, name: name, percent: percent, resetsAt: resetsAt))
            }
        }

        currentStatus = Status(
            state: object["state"] as? String ?? "unknown",
            title: object["title"] as? String ?? "...",
            detail: object["detail"] as? String ?? "No data yet",
            percent: object["percent"] as? Int,
            metrics: metrics,
            tierNote: object["tier_note"] as? String,
            lastApiUpdate: object["last_api_update"] as? String,
            error: object["error"] as? String,
            chart: (object["chart"] as? [Int]) ?? [],
            chartResets: (object["chart_resets"] as? [Double]) ?? [],
            labels: (object["labels"] as? [String: String]) ?? [:]
        )
        renderMenu()
    }

    private func renderMenu() {
        // Fire means "on track to blow the weekly budget" — but being nearly
        // maxed right now (raw red) is more urgent, so that wins.
        let rawCritical = (currentStatus.percent ?? 0) >= 90
        if !rawCritical && paceIsHot() {
            statusItem.button?.image = nil
            statusItem.button?.title = "🔥 " + menuBarTitle()
        } else {
            statusItem.button?.image = dotImage(dotName())
            statusItem.button?.imagePosition = .imageLeading
            statusItem.button?.title = menuBarTitle()
        }
        statusItem.button?.toolTip = currentStatus.detail

        menu = NSMenu()
        menu.addItem(disabled("ClaudeMeter — \(currentStatus.detail)"))
        if !currentStatus.metrics.isEmpty {
            menu.addItem(NSMenuItem.separator())
            var codexSeparatorAdded = false
            for metric in currentStatus.metrics {
                // Codex rows (optional panel) are grouped after the Claude ones.
                if metric.key.hasPrefix("codex_"), !codexSeparatorAdded {
                    menu.addItem(NSMenuItem.separator())
                    codexSeparatorAdded = true
                }
                if let left = timeLeftLong(metric.resetsAt) {
                    menu.addItem(disabled("\(metric.name): \(metric.percent)%  ·  \(currentStatus.label("resets_in", "resets in")) \(left)"))
                } else {
                    menu.addItem(disabled("\(metric.name): \(metric.percent)%"))
                }
                if metric.key == "seven_day", let pace = weeklyPaceLine() {
                    menu.addItem(disabled("   \(pace)"))
                }
            }
        }
        if let note = currentStatus.tierNote, !note.isEmpty {
            menu.addItem(NSMenuItem.separator())
            menu.addItem(disabled(note))
        }
        if currentStatus.chart.contains(where: { $0 > 0 }) {
            menu.addItem(NSMenuItem.separator())
            menu.addItem(disabled(currentStatus.label("usage_history_24h", "Usage History (24h)")))
            let chartItem = NSMenuItem()
            chartItem.view = ChartView(data: currentStatus.chart, resets: currentStatus.chartResets)
            menu.addItem(chartItem)
        }
        menu.addItem(NSMenuItem.separator())
        menu.addItem(disabled("\(currentStatus.label("freshness", "Freshness")): \(freshnessText())"))
        if let err = currentStatus.error, !err.isEmpty {
            menu.addItem(disabled("\u{26A0}\u{FE0E} \(err)"))
        }
        menu.addItem(NSMenuItem.separator())
        menu.addItem(item(currentStatus.label("refresh_now", "Refresh Now"), #selector(refreshAction)))
        menu.addItem(item(currentStatus.label("open_claude_usage", "Open Claude Usage"), #selector(openClaude)))
        menu.addItem(item(currentStatus.label("check_for_updates", "Check for Updates"), #selector(checkForUpdates)))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(settingsMenuItem())
        menu.addItem(item(currentStatus.label("export_csv", "Export History (CSV)"), #selector(exportHistoryCSV)))
        menu.addItem(item(currentStatus.label("export_json", "Export History (JSON)"), #selector(exportHistoryJSON)))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(item(currentStatus.label("open_config", "Open Config"), #selector(openConfig)))
        menu.addItem(item(currentStatus.label("export_config", "Export Config..."), #selector(exportConfig)))
        menu.addItem(item(currentStatus.label("import_config", "Import Config..."), #selector(importConfig)))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(item(autostartInstalled()
            ? currentStatus.label("disable_autostart", "Disable Autostart")
            : currentStatus.label("enable_autostart", "Enable Autostart"), #selector(toggleAutostart)))
        menu.addItem(item(currentStatus.label("open_logs", "Open Logs"), #selector(openLogs)))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(item(currentStatus.label("quit", "Quit"), #selector(quit)))

        statusItem.menu = menu
    }

    /// Settings submenu. Every row shells out to the agent's `--set` command so
    /// validation and defaults stay in one place (Rust), then restarts the
    /// agent to pick the change up.
    private func settingsMenuItem() -> NSMenuItem {
        let root = NSMenuItem(title: currentStatus.label("settings", "Settings"), action: nil, keyEquivalent: "")
        let submenu = NSMenu()

        let toggles: [(String, String, String)] = [
            ("show_codex_section", "show_chatgpt_section", "Show Codex section"),
            ("show_model_limits", "show_model_limits", "Show model limits"),
            ("show_extra_usage", "show_extra_usage", "Show extra usage"),
            ("notification_sound", "notifications.sound", "Notification sound"),
            ("show_startup_notification", "show_startup_notification", "Show startup notification"),
            ("show_login_expiry_warning", "token_expiry_warning", "Show login expiry warning"),
        ]
        for (labelKey, configKey, fallback) in toggles {
            let entry = NSMenuItem(title: currentStatus.label(labelKey, fallback),
                                   action: #selector(toggleSetting(_:)),
                                   keyEquivalent: "")
            entry.target = self
            entry.state = currentStatus.flag(labelKey) ? .on : .off
            entry.representedObject = "\(configKey)=\(currentStatus.flag(labelKey) ? "false" : "true")"
            submenu.addItem(entry)
        }

        submenu.addItem(NSMenuItem.separator())

        // Cycles through the same threshold presets as the Windows settings.
        let thresholds = NSMenuItem(
            title: "\(currentStatus.label("alert_thresholds", "Alert thresholds")): \(currentStatus.label("value_thresholds", "50% / 75% / 90%"))",
            action: #selector(toggleSetting(_:)),
            keyEquivalent: "")
        thresholds.target = self
        thresholds.representedObject = "notifications.thresholds=cycle"
        submenu.addItem(thresholds)

        submenu.addItem(NSMenuItem.separator())

        // Language: Auto plus every locale the agent ships.
        let language = NSMenuItem(title: currentStatus.label("language", "Language"), action: nil, keyEquivalent: "")
        let languageMenu = NSMenu()
        let current = currentStatus.label("value_language", "auto")
        for (code, name) in Self.languages {
            let entry = NSMenuItem(title: name, action: #selector(toggleSetting(_:)), keyEquivalent: "")
            entry.target = self
            entry.state = (code == current) ? .on : .off
            entry.representedObject = "language=\(code)"
            languageMenu.addItem(entry)
        }
        language.submenu = languageMenu
        submenu.addItem(language)

        root.submenu = submenu
        return root
    }

    /// Locale codes accepted by the agent's config, with native display names.
    private static let languages: [(String, String)] = [
        ("auto", "Auto"), ("en", "English"), ("uk", "Українська"), ("es", "Español"),
        ("de", "Deutsch"), ("fr", "Français"), ("pt", "Português"), ("it", "Italiano"),
        ("nl", "Nederlands"), ("pl", "Polski"), ("cs", "Čeština"), ("sk", "Slovenčina"),
        ("hr", "Hrvatski"), ("sr", "Српски"), ("ro", "Română"), ("hu", "Magyar"),
        ("bg", "Български"), ("el", "Ελληνικά"), ("sv", "Svenska"), ("da", "Dansk"),
        ("no", "Norsk"), ("fi", "Suomi"), ("et", "Eesti"), ("lv", "Latviešu"),
        ("lt", "Lietuvių"), ("ca", "Català"), ("tr", "Türkçe"), ("ru", "Русский"),
        ("he", "עברית"), ("ar", "العربية"), ("fa", "فارسی"), ("hi", "हिन्दी"),
        ("bn", "বাংলা"), ("th", "ภาษาไทย"), ("vi", "Tiếng Việt"), ("id", "Indonesia"),
        ("ms", "Melayu"), ("tl", "Filipino"), ("ja", "日本語"), ("ko", "한국어"),
        ("zh", "中文"),
    ]

    /// Menu bar text: 5-hour session usage with time left, then the weekly
    /// percent (no label), e.g. "30% · 1h03m | 24%". The leading severity dot
    /// is set as the button image.
    private func menuBarTitle() -> String {
        guard currentStatus.percent != nil else {
            return currentStatus.state == "error" ? "!" : "…"
        }
        guard let session = sessionMetric() else {
            return "\(currentStatus.percent ?? 0)%"
        }
        var text = "\(session.percent)%"
        if let left = timeLeftShort(session.resetsAt) {
            text += " · \(left)"
        }
        if let weekly = weeklyMetric() {
            text += " | \(weekly.percent)%"
        }
        return text
    }

    /// The 5-hour session metric drives the menu bar; fall back to the first.
    private func sessionMetric() -> Metric? {
        currentStatus.metrics.first(where: { $0.key == "five_hour" })
            ?? currentStatus.metrics.first
    }

    /// The weekly (all-models) metric, shown as the trailing percent.
    private func weeklyMetric() -> Metric? {
        currentStatus.metrics.first(where: { $0.key == "seven_day" })
    }

    /// Linear projection of where the weekly limit lands at reset, assuming
    /// usage continues at the average rate since the week began, along with
    /// the fraction of the week elapsed. Over 100% means on pace to hit the
    /// wall before reset. Nil before ~8h in, when the projection is too noisy.
    private func weeklyProjection() -> (proj: Int, frac: Double)? {
        guard let weekly = weeklyMetric(),
              let iso = weekly.resetsAt,
              let reset = parseISO(iso) else { return nil }
        let weekLen = 7.0 * 24 * 3600
        let secsLeft = reset.timeIntervalSinceNow
        guard secsLeft > 0 else { return nil }
        let frac = (weekLen - secsLeft) / weekLen
        guard frac > 0.05 else { return nil }
        return (Int((Double(weekly.percent) / frac).rounded()), frac)
    }

    /// True when on track to overshoot the weekly budget: projected >=115%,
    /// but only after ~1.5 days into the window so early noise does not trip it.
    private func paceIsHot() -> Bool {
        guard let pj = weeklyProjection() else { return false }
        return pj.frac >= 0.20 && pj.proj >= 115
    }

    private func weeklyPaceLine() -> String? {
        guard weeklyMetric() != nil else { return nil }
        guard let pj = weeklyProjection() else {
            return currentStatus.label("pace_too_early", "Pace: too early to project")
        }
        let indicator = paceIsHot() ? "🔥" : (pj.proj >= 100 ? "🟡" : "🟢")
        let text = currentStatus.label("pace_projection", "Pace: on track for ~{}% by reset")
            .replacingOccurrences(of: "{}", with: "\(pj.proj)")
        return "\(text)  \(indicator)"
    }

    /// Current-level dot by the worst (max) raw limit. Pace/trajectory is shown
    /// separately as the 🔥 indicator, not folded into the dot color.
    private func dotName() -> String {
        switch currentStatus.percent {
        case .some(let p) where p >= 90: return "dot_red"
        case .some(let p) where p >= 60: return "dot_yellow"
        case .some: return "dot_green"
        case .none: return "dot_gray"
        }
    }

    private func dotImage(_ name: String) -> NSImage? {
        guard let url = Bundle.main.url(forResource: name, withExtension: "png"),
              let img = NSImage(contentsOf: url) else { return nil }
        img.size = NSSize(width: 14, height: 14)
        return img
    }

    private func parseISO(_ iso: String) -> Date? {
        let frac = ISO8601DateFormatter()
        frac.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let d = frac.date(from: iso) { return d }
        let plain = ISO8601DateFormatter()
        plain.formatOptions = [.withInternetDateTime]
        return plain.date(from: iso)
    }

    /// Compact countdown for the menu bar: "45m", "1h03m", "5d10h".
    private func timeLeftShort(_ iso: String?) -> String? {
        guard let iso = iso, let date = parseISO(iso) else { return nil }
        let secs = Int(date.timeIntervalSinceNow)
        if secs <= 0 { return nil }
        let h = secs / 3600, m = (secs % 3600) / 60
        if h >= 24 { return "\(h / 24)d\(h % 24)h" }
        if h >= 1 { return String(format: "%dh%02dm", h, m) }
        return "\(m)m"
    }

    /// Readable countdown for the dropdown: "45m", "1h 3m", "5d 10h".
    private func timeLeftLong(_ iso: String?) -> String? {
        guard let iso = iso, let date = parseISO(iso) else { return nil }
        let secs = Int(date.timeIntervalSinceNow)
        if secs <= 0 { return nil }
        let h = secs / 3600, m = (secs % 3600) / 60
        if h >= 24 { return "\(h / 24)d \(h % 24)h" }
        if h >= 1 { return "\(h)h \(m)m" }
        return "\(m)m"
    }

    private func freshnessText() -> String {
        guard let iso = currentStatus.lastApiUpdate,
              let date = parseISO(iso) else {
            return currentStatus.state == "error"
                ? currentStatus.label("api_error", "API error")
                : currentStatus.label("cached", "Cached / no API data yet")
        }
        let age = max(0, Int(Date().timeIntervalSince(date)))
        if age < 20 { return currentStatus.label("live", "Live") }
        let amount = age < 60 ? "\(age)s" : "\(age / 60)m"
        return currentStatus.label("time_ago", "{} ago").replacingOccurrences(of: "{}", with: amount)
    }

    private func disabled(_ title: String) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: nil, keyEquivalent: "")
        item.isEnabled = false
        return item
    }

    private func item(_ title: String, _ action: Selector) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
        item.target = self
        return item
    }

    @objc private func refreshAction() {
        refreshNow()
    }

    @objc private func openClaude() {
        NSWorkspace.shared.open(URL(string: "https://claude.ai/settings/usage")!)
    }

    @objc private func checkForUpdates() {
        let url = URL(string: "https://api.github.com/repos/klivak/claude-meter/releases/latest")!
        URLSession.shared.dataTask(with: url) { data, _, _ in
            guard let data,
                  let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let tag = object["tag_name"] as? String,
                  let html = object["html_url"] as? String else {
                self.notify("ClaudeMeter", self.currentStatus.label("update_check_failed", "Could not check for updates."))
                return
            }
            let current = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "0.0.0"
            let remote = tag.trimmingCharacters(in: CharacterSet(charactersIn: "v"))
            if !Self.isNewerVersion(remote, than: current) {
                self.notify("ClaudeMeter", self.currentStatus.label("latest_version", "You are running the latest version."))
            } else {
                self.notify(self.currentStatus.label("update_available", "Update available"), "\(tag) is available.")
                if let url = URL(string: html) {
                    NSWorkspace.shared.open(url)
                }
            }
        }.resume()
    }

    private static func isNewerVersion(_ remote: String, than current: String) -> Bool {
        let remoteParts = remote.split(separator: ".").compactMap { Int($0) }
        let currentParts = current.split(separator: ".").compactMap { Int($0) }
        for index in 0..<3 {
            let remotePart = index < remoteParts.count ? remoteParts[index] : 0
            let currentPart = index < currentParts.count ? currentParts[index] : 0
            if remotePart != currentPart { return remotePart > currentPart }
        }
        return false
    }

    @objc private func openConfig() {
        openFile(appSupport.appendingPathComponent("config.json"))
    }

    /// Apply a `key=value` change through the agent, then restart it so the new
    /// config takes effect and a fresh status.json (with new labels) is written.
    @objc private func toggleSetting(_ sender: NSMenuItem) {
        guard let assignment = sender.representedObject as? String else { return }
        DispatchQueue.global(qos: .utility).async {
            self.runAgent(["--set", assignment])
            DispatchQueue.main.async {
                self.agent?.terminate()
                self.agent = nil
                self.startAgent()
                self.refreshNow()
            }
        }
    }

    @objc private func exportHistoryCSV() {
        exportHistory(flag: "--export-csv", suggestedName: "claudemeter-history.csv")
    }

    @objc private func exportHistoryJSON() {
        exportHistory(flag: "--export-json", suggestedName: "claudemeter-history.json")
    }

    /// The agent owns the SQLite history, so exporting means asking it to write
    /// the file the user picked.
    private func exportHistory(flag: String, suggestedName: String) {
        let panel = NSSavePanel()
        panel.nameFieldStringValue = suggestedName
        panel.begin { response in
            guard response == .OK, let url = panel.url else { return }
            DispatchQueue.global(qos: .utility).async {
                let output = self.runAgent([flag, url.path])
                DispatchQueue.main.async {
                    self.notify("ClaudeMeter", output.isEmpty ? "Export finished." : output)
                }
            }
        }
    }

    /// Run the agent binary with arguments and return its trimmed stdout.
    @discardableResult
    private func runAgent(_ arguments: [String]) -> String {
        let process = Process()
        process.executableURL = agentURL()
        process.arguments = arguments
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = FileHandle.nullDevice
        do {
            try process.run()
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            process.waitUntilExit()
            return String(data: data, encoding: .utf8)?
                .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        } catch {
            writeLog("Agent command failed: \(error)")
            return ""
        }
    }

    @objc private func exportConfig() {
        let panel = NSSavePanel()
        panel.nameFieldStringValue = "claudemeter-config.json"
        panel.begin { response in
            guard response == .OK, let url = panel.url else { return }
            let source = self.appSupport.appendingPathComponent("config.json")
            try? FileManager.default.removeItem(at: url)
            do {
                try FileManager.default.copyItem(at: source, to: url)
            } catch {
                self.notify("ClaudeMeter", "Config export failed.")
            }
        }
    }

    @objc private func importConfig() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.canChooseDirectories = false
        panel.begin { response in
            guard response == .OK, let url = panel.url else { return }
            let dest = self.appSupport.appendingPathComponent("config.json")
            do {
                try? FileManager.default.removeItem(at: dest)
                try FileManager.default.copyItem(at: url, to: dest)
                self.notify("ClaudeMeter", "Config imported. Restarting agent.")
                self.agent?.terminate()
                self.agent = nil
                self.startAgent()
            } catch {
                self.notify("ClaudeMeter", "Config import failed.")
            }
        }
    }

    @objc private func toggleAutostart() {
        if autostartInstalled() {
            removeAutostart()
        } else {
            installAutostart()
        }
        renderMenu()
    }

    @objc private func openLogs() {
        openFile(appSupport.appendingPathComponent("claudemeter.log"))
    }

    @objc private func quit() {
        NSApp.terminate(nil)
    }

    private func agentURL() -> URL {
        Bundle.main.resourceURL!.appendingPathComponent("claudemeter-agent")
    }

    private func openFile(_ url: URL) {
        if !FileManager.default.fileExists(atPath: url.path) {
            FileManager.default.createFile(atPath: url.path, contents: Data())
        }
        NSWorkspace.shared.open(url)
    }

    private func launchAgentPath() -> URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/LaunchAgents/com.klivak.claudemeter.plist")
    }

    private func autostartInstalled() -> Bool {
        FileManager.default.fileExists(atPath: launchAgentPath().path)
    }

    private func installAutostart() {
        // Keep this plist identical to scripts/install-macos-launchagent.sh.
        // Launch the bundle executable directly, NOT via /usr/bin/open: open
        // exits immediately, which with KeepAlive made launchd relaunch in a
        // tight loop. KeepAlive=false lets the app start at login yet still be
        // quit from its menu. See commit 4769be3.
        let binPath = Bundle.main.executablePath ?? Bundle.main.bundleURL
            .appendingPathComponent("Contents/MacOS/ClaudeMeter").path
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        let plist = """
        <?xml version="1.0" encoding="UTF-8"?>
        <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
        <plist version="1.0">
        <dict>
          <key>Label</key>
          <string>com.klivak.claudemeter</string>
          <key>ProgramArguments</key>
          <array>
            <string>\(binPath)</string>
          </array>
          <key>RunAtLoad</key>
          <true/>
          <key>KeepAlive</key>
          <false/>
          <key>StandardOutPath</key>
          <string>\(home)/Library/Logs/claudemeter.out.log</string>
          <key>StandardErrorPath</key>
          <string>\(home)/Library/Logs/claudemeter.err.log</string>
        </dict>
        </plist>
        """
        let path = launchAgentPath()
        try? FileManager.default.createDirectory(at: path.deletingLastPathComponent(), withIntermediateDirectories: true)
        try? plist.write(to: path, atomically: true, encoding: .utf8)
        // Register with launchd now so it's active without needing a re-login.
        runLaunchctl(["load", "-w", path.path])
    }

    private func removeAutostart() {
        let path = launchAgentPath()
        // Unregister from launchd first, then delete the file. Without the
        // unload the job stays live in the current session and only stops
        // autostarting after the next login.
        runLaunchctl(["unload", "-w", path.path])
        try? FileManager.default.removeItem(at: path)
    }

    private func runLaunchctl(_ args: [String]) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        process.arguments = args
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            writeLog("launchctl \(args.joined(separator: " ")) failed: \(error)")
        }
    }

    private func notify(_ title: String, _ message: String) {
        let script = "display notification \"\(escapeAppleScript(message))\" with title \"\(escapeAppleScript(title))\""
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        process.arguments = ["-e", script]
        try? process.run()
    }

    private func writeLog(_ message: String) {
        let path = appSupport.appendingPathComponent("claudemeter.log")
        let line = "\(Date()) \(message)\n"
        if let data = line.data(using: .utf8) {
            if FileManager.default.fileExists(atPath: path.path),
               let handle = try? FileHandle(forWritingTo: path) {
                _ = try? handle.seekToEnd()
                try? handle.write(contentsOf: data)
                try? handle.close()
            } else {
                try? data.write(to: path)
            }
        }
    }

    private func escapeAppleScript(_ value: String) -> String {
        value.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
    }
}

/// "Usage History (24h)" bar chart, ported from the Windows popup design
/// (src/ui/render.rs draw_chart): surface background, 25/50/75 grid lines,
/// and one bar per bucket colored by threshold (>=80 red, >=50 yellow, else
/// green). Data is 48 buckets oldest-first, each 0-100.
final class ChartView: NSView {
    private let data: [Int]
    private let resets: [Double]

    // Catppuccin palette, matching src/ui/colors.rs. Resolved per-appearance
    // so the chart tracks the menu's light/dark mode.
    private func palette() -> (surface: NSColor, grid: NSColor, green: NSColor, yellow: NSColor, red: NSColor, accent: NSColor) {
        let dark = effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
        let green = hex(0x40a02b)
        let yellow = hex(0xdf8e1d)
        let red = hex(0xd20f39)
        if dark {
            return (hex(0x313244), hex(0x45475a), green, yellow, red, hex(0x89b4fa))
        } else {
            return (hex(0xdce0e8), hex(0xbcc0cc), green, yellow, red, hex(0x1e66f5))
        }
    }

    private func hex(_ rgb: Int) -> NSColor {
        NSColor(
            srgbRed: CGFloat((rgb >> 16) & 0xff) / 255.0,
            green: CGFloat((rgb >> 8) & 0xff) / 255.0,
            blue: CGFloat(rgb & 0xff) / 255.0,
            alpha: 1.0
        )
    }

    init(data: [Int], resets: [Double]) {
        self.data = data
        self.resets = resets
        // Width is a starting size; autoresizing stretches it to the menu width.
        super.init(frame: NSRect(x: 0, y: 0, width: 260, height: 96))
        autoresizingMask = [.width]
    }

    required init?(coder: NSCoder) { fatalError("not used") }

    override func draw(_ dirtyRect: NSRect) {
        let pad: CGFloat = 14
        let labelH: CGFloat = 14
        let chartX = pad
        let chartW = bounds.width - pad * 2
        let chartTop = bounds.height - 6
        let chartBottom = labelH + 4
        let chartH = chartTop - chartBottom
        guard chartW > 0, chartH > 0 else { return }

        let colors = palette()

        // Chart background
        colors.surface.setFill()
        NSBezierPath(rect: NSRect(x: chartX, y: chartBottom, width: chartW, height: chartH)).fill()

        // Grid lines at 25/50/75%
        colors.grid.setStroke()
        for pct in [25, 50, 75] {
            let gy = chartBottom + chartH * CGFloat(pct) / 100.0
            let line = NSBezierPath()
            line.lineWidth = 1
            line.move(to: NSPoint(x: chartX, y: gy))
            line.line(to: NSPoint(x: chartX + chartW, y: gy))
            line.stroke()
        }

        // Bars (note: AppKit y-axis is bottom-up, so bars grow upward directly)
        if !data.isEmpty {
            let barW = max(chartW / CGFloat(data.count), 2.0)
            let gap = max(1.0, barW / 6.0)
            for (i, val) in data.enumerated() {
                let h = CGFloat(val) / 100.0 * chartH
                guard h > 0.5 else { continue }
                let x = chartX + CGFloat(i) * barW
                let color = val >= 80 ? colors.red : (val >= 50 ? colors.yellow : colors.green)
                color.setFill()
                NSBezierPath(rect: NSRect(x: x + gap, y: chartBottom, width: barW - gap * 2, height: h)).fill()
            }
        }

        // Dashed vertical lines at past 5-hour session resets. hours_ago=0 is
        // "now" (right edge), 24 is the left edge: x = left + w * (1 - ago/24).
        colors.accent.setStroke()
        for ago in resets where ago >= 0 && ago <= 24 {
            let rx = chartX + chartW * CGFloat(1.0 - ago / 24.0)
            let line = NSBezierPath()
            line.lineWidth = 1
            line.setLineDash([3, 3], count: 2, phase: 0)
            line.move(to: NSPoint(x: rx, y: chartBottom))
            line.line(to: NSPoint(x: rx, y: chartTop))
            line.stroke()
        }

        // Axis labels: "24h ago" left, "now" right
        let labelAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 9),
            .foregroundColor: NSColor.secondaryLabelColor,
        ]
        let left = NSAttributedString(string: "24h ago", attributes: labelAttrs)
        left.draw(at: NSPoint(x: chartX, y: 0))
        let right = NSAttributedString(string: "now", attributes: labelAttrs)
        right.draw(at: NSPoint(x: chartX + chartW - right.size().width, y: 0))
    }
}
