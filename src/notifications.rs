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
