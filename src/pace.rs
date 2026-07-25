//! Weekly burn-rate projection.
//!
//! Given how much of the weekly quota is already spent and how much of the
//! window has elapsed, extrapolate linearly to the reset. Over 100% means the
//! current pace runs into the wall before the window rolls over.
//!
//! Mirrors the projection the macOS menu bar has shown since it shipped
//! (`weeklyProjection` in `macos/ClaudeMeterApp.swift`) so both platforms
//! report the same number.

use chrono::{DateTime, Utc};

const WEEK_SECONDS: f64 = 7.0 * 24.0 * 3600.0;

/// Fraction of the window that must have elapsed before a projection is
/// meaningful. Below this the divisor is tiny and the estimate explodes.
const MIN_ELAPSED_FRACTION: f64 = 0.05;

/// Projection is only called "hot" this far into the window — early spikes are
/// normal and would otherwise raise a false alarm on day one.
const HOT_MIN_ELAPSED_FRACTION: f64 = 0.20;

/// Projected utilization at which the pace counts as overshooting.
const HOT_PROJECTION_PERCENT: u32 = 115;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeeklyPace {
    /// Utilization the week lands on at reset if the current rate holds.
    pub projected_percent: u32,
    /// How much of the weekly window has elapsed, 0.0-1.0.
    pub elapsed_fraction: f64,
}

impl WeeklyPace {
    /// On track to blow through the weekly budget before it resets.
    pub fn is_hot(&self) -> bool {
        self.elapsed_fraction >= HOT_MIN_ELAPSED_FRACTION
            && self.projected_percent >= HOT_PROJECTION_PERCENT
    }

    /// Projected to reach the limit, but not yet in "hot" territory.
    pub fn is_over_budget(&self) -> bool {
        self.projected_percent >= 100
    }
}

/// Project the weekly metric forward from its current utilization.
///
/// Returns `None` when the reset time is unparseable, already past, or so
/// recent that less than [`MIN_ELAPSED_FRACTION`] of the window has elapsed.
pub fn weekly_pace(utilization: f64, resets_at: &str, now: DateTime<Utc>) -> Option<WeeklyPace> {
    let reset = DateTime::parse_from_rfc3339(resets_at)
        .ok()?
        .with_timezone(&Utc);
    let seconds_left = (reset - now).num_seconds() as f64;
    if seconds_left <= 0.0 {
        return None;
    }
    let elapsed_fraction = (WEEK_SECONDS - seconds_left) / WEEK_SECONDS;
    if elapsed_fraction <= MIN_ELAPSED_FRACTION {
        return None;
    }
    Some(WeeklyPace {
        projected_percent: (utilization / elapsed_fraction).round() as u32,
        elapsed_fraction,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-26T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    /// Half the week gone with 30% spent projects to 60% at reset.
    #[test]
    fn test_linear_projection() {
        let reset = "2026-07-30T00:00:00Z"; // 3.5 days out → half elapsed
        let pace = weekly_pace(30.0, reset, now()).unwrap();
        assert_eq!(pace.projected_percent, 60);
        assert!((pace.elapsed_fraction - 0.5).abs() < 0.01);
        assert!(!pace.is_hot());
        assert!(!pace.is_over_budget());
    }

    #[test]
    fn test_hot_when_projected_over_budget_late_enough() {
        let reset = "2026-07-30T00:00:00Z"; // 3.5 days out → half elapsed
        let pace = weekly_pace(60.0, reset, now()).unwrap();
        assert_eq!(pace.projected_percent, 120);
        assert!(pace.is_hot());
        assert!(pace.is_over_budget());
    }

    /// A spike in the first hours must not be called hot, however steep.
    #[test]
    fn test_early_spike_is_not_hot() {
        let reset = "2026-08-01T12:00:00Z"; // ~6.0 days left → ~14% elapsed
        let pace = weekly_pace(40.0, reset, now()).unwrap();
        assert!(pace.projected_percent > 200);
        assert!(!pace.is_hot());
    }

    #[test]
    fn test_none_before_projection_window() {
        // Reset almost a full week out: too little elapsed to project.
        assert!(weekly_pace(5.0, "2026-08-02T06:00:00Z", now()).is_none());
    }

    #[test]
    fn test_none_when_reset_passed_or_unparseable() {
        assert!(weekly_pace(50.0, "2026-07-25T12:00:00Z", now()).is_none());
        assert!(weekly_pace(50.0, "not-a-date", now()).is_none());
    }
}
