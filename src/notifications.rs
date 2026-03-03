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

    /// Returns the single highest threshold that should fire for this metric/utilization.
    /// Marks all exceeded thresholds as fired. Clears fired thresholds when utilization drops below them.
    /// When multiple thresholds are crossed at once, only the highest one is returned
    /// to avoid spamming the user with multiple notifications.
    pub fn check(&mut self, metric: &str, utilization: f64, thresholds: &[u8]) -> Vec<u8> {
        let mut highest_new: Option<u8> = None;

        for &threshold in thresholds {
            let key = (metric.to_string(), threshold);
            if utilization >= threshold as f64 {
                if !self.fired.contains(&key) {
                    self.fired.insert(key);
                    // Track the highest newly-exceeded threshold
                    if highest_new.map_or(true, |h| threshold > h) {
                        highest_new = Some(threshold);
                    }
                }
            } else {
                // Reset so it can fire again when usage rises again
                self.fired.remove(&key);
            }
        }

        highest_new.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_dedup() {
        let mut tracker = NotificationTracker::new();
        let thresholds = [50u8, 75, 90];

        // First time at 60% → fires only threshold 50
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

    #[test]
    fn test_only_highest_threshold_fires() {
        let mut tracker = NotificationTracker::new();
        let thresholds = [50u8, 75, 90];

        // Jump straight to 96% → only fires 90 (the highest), not all three
        let fired = tracker.check("five_hour", 96.0, &thresholds);
        assert_eq!(fired, vec![90]);

        // All thresholds are now marked as fired, so no new fires
        let fired = tracker.check("five_hour", 96.0, &thresholds);
        assert!(fired.is_empty());

        // Drop to 80% → clears 90, but 50 and 75 still fired
        let fired = tracker.check("five_hour", 80.0, &thresholds);
        assert!(fired.is_empty());

        // Back to 96% → only 90 fires again (50 and 75 still marked)
        let fired = tracker.check("five_hour", 96.0, &thresholds);
        assert_eq!(fired, vec![90]);
    }

    #[test]
    fn test_gradual_increase_fires_each_threshold_once() {
        let mut tracker = NotificationTracker::new();
        let thresholds = [50u8, 75, 90];

        // Gradually increasing usage fires each threshold individually
        let fired = tracker.check("five_hour", 55.0, &thresholds);
        assert_eq!(fired, vec![50]);

        let fired = tracker.check("five_hour", 78.0, &thresholds);
        assert_eq!(fired, vec![75]);

        let fired = tracker.check("five_hour", 95.0, &thresholds);
        assert_eq!(fired, vec![90]);
    }
}
