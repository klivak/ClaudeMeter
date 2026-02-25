use std::collections::HashSet;

/// Tracks which (metric, threshold) pairs have already been notified
/// to avoid spamming the user.
pub struct NotificationTracker {
    fired: HashSet<(String, u8)>,
}

impl NotificationTracker {
    pub fn new() -> Self {
        Self {
            fired: HashSet::new(),
        }
    }

    /// Returns the thresholds that should fire for this metric/utilization.
    /// Marks them as fired. Clears fired thresholds when utilization drops below them.
    pub fn check(&mut self, metric: &str, utilization: f64, thresholds: &[u8]) -> Vec<u8> {
        let mut to_fire = Vec::new();

        for &threshold in thresholds {
            let key = (metric.to_string(), threshold);
            if utilization >= threshold as f64 {
                if !self.fired.contains(&key) {
                    self.fired.insert(key);
                    to_fire.push(threshold);
                }
            } else {
                // Reset so it can fire again when usage rises again
                self.fired.remove(&key);
            }
        }

        to_fire
    }
}

/// Send a Windows toast notification.
///
/// Uses a simple PowerShell-based notification as a fallback that
/// works without the winrt-notification dependency complexities.
pub fn send_toast(title: &str, body: &str) {
    let title = title.replace('"', "'");
    let body = body.replace('"', "'");

    // Use PowerShell to show a toast notification (works on Windows 10/11)
    let script = format!(
        r#"
$title = "{title}"
$body = "{body}"
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$template = [Windows.UI.Notifications.ToastTemplateType]::ToastText02
$xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent($template)
$xml.GetElementsByTagName("text")[0].AppendChild($xml.CreateTextNode($title)) | Out-Null
$xml.GetElementsByTagName("text")[1].AppendChild($xml.CreateTextNode($body)) | Out-Null
$toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
$notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("ClaudeMeter")
$notifier.Show($toast)
"#
    );

    let _ = std::process::Command::new("powershell")
        .args([
            "-WindowStyle",
            "Hidden",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_dedup() {
        let mut tracker = NotificationTracker::new();
        let thresholds = [50u8, 75, 90];

        // First time at 60% → fires threshold 50
        let fired = tracker.check("five_hour", 60.0, &thresholds);
        assert_eq!(fired, vec![50]);

        // Second time at 60% → no new fires
        let fired = tracker.check("five_hour", 60.0, &thresholds);
        assert!(fired.is_empty());

        // Drops below 50% → clears the 50% threshold
        let fired = tracker.check("five_hour", 40.0, &thresholds);
        assert!(fired.is_empty());

        // Back to 60% → fires 50% again
        let fired = tracker.check("five_hour", 60.0, &thresholds);
        assert_eq!(fired, vec![50]);
    }
}
