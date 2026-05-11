use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub struct AngleReliabilityConfig {
    pub zero_eps_deg: f32,
    pub zero_confirm_duration_s: f32,
    /// Readings whose magnitude exceeds this value are treated as unreliable
    /// power-on garbage and held back just like near-zero glitches.
    /// Defaults to 3,686,400° (FashionStar ±1024-turn protocol limit).
    pub valid_range_deg: f32,
}

impl Default for AngleReliabilityConfig {
    fn default() -> Self {
        Self {
            zero_eps_deg: 0.2,
            zero_confirm_duration_s: 0.65,
            valid_range_deg: 3_686_400.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AngleReliabilityState {
    pub last_good_deg: Option<f32>,
    pub last_raw_deg: Option<f32>,
    pub last_filtered_deg: Option<f32>,
    pub zero_candidate_since_s: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct AngleReliability {
    pub config: AngleReliabilityConfig,
    pub state: AngleReliabilityState,
}

impl AngleReliability {
    pub fn filter(&mut self, raw_deg: f32) -> (f32, bool) {
        self.filter_at(raw_deg, Self::now_seconds())
    }

    pub fn filter_at(&mut self, raw_deg: f32, now_s: f64) -> (f32, bool) {
        self.state.last_raw_deg = Some(raw_deg);
        let is_near_zero = raw_deg.abs() <= self.config.zero_eps_deg;
        let is_in_range = raw_deg.abs() <= self.config.valid_range_deg;

        let Some(last_good) = self.state.last_good_deg else {
            // First reading ever.
            if !is_in_range {
                // Out-of-range garbage before any good reading — no value to hold.
                self.state.last_filtered_deg = Some(0.0);
                return (0.0, false);
            }
            if is_near_zero {
                let started = *self.state.zero_candidate_since_s.get_or_insert(now_s);
                if Self::elapsed_seconds(started, now_s) >= self.config.zero_confirm_duration_s {
                    self.state.zero_candidate_since_s = None;
                    self.state.last_good_deg = Some(raw_deg);
                    self.state.last_filtered_deg = Some(raw_deg);
                    return (raw_deg, true);
                }
                self.state.last_filtered_deg = Some(0.0);
                return (0.0, false);
            }
            self.state.last_good_deg = Some(raw_deg);
            self.state.last_filtered_deg = Some(raw_deg);
            return (raw_deg, true);
        };

        if !is_in_range {
            // Out-of-range (e.g. power-on firmware garbage) — hold last good.
            self.state.zero_candidate_since_s = None;
            self.state.last_filtered_deg = Some(last_good);
            return (last_good, false);
        }

        if is_near_zero {
            let started = *self.state.zero_candidate_since_s.get_or_insert(now_s);
            if Self::elapsed_seconds(started, now_s) >= self.config.zero_confirm_duration_s {
                self.state.zero_candidate_since_s = None;
                self.state.last_good_deg = Some(raw_deg);
                self.state.last_filtered_deg = Some(raw_deg);
                return (raw_deg, true);
            }
            self.state.last_filtered_deg = Some(last_good);
            return (last_good, false);
        }

        self.state.zero_candidate_since_s = None;
        self.state.last_good_deg = Some(raw_deg);
        self.state.last_filtered_deg = Some(raw_deg);
        (raw_deg, true)
    }

    fn now_seconds() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs_f64())
            .unwrap_or(0.0)
    }

    fn elapsed_seconds(started_s: f64, now_s: f64) -> f32 {
        (now_s - started_s).max(0.0) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::AngleReliability;

    #[test]
    fn startup_out_of_range_returns_zero_unreliable() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(-23_592_960.0), (0.0, false));
    }

    #[test]
    fn out_of_range_after_good_holds_last_good() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(5.0), (5.0, true));
        assert_eq!(filter.filter(-23_592_960.0), (5.0, false));
        assert_eq!(filter.filter(-23_592_960.0), (5.0, false));
        // Once back in range, accepted immediately.
        assert_eq!(filter.filter(5.1), (5.1, true));
    }

    #[test]
    fn startup_trusts_first_reading_nonzero() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(-12.6), (-12.6, true));
    }

    #[test]
    fn startup_zero_needs_confirm() {
        let mut filter = AngleReliability::default();
        let t0 = 100.0;
        assert_eq!(filter.filter_at(-0.1, t0), (0.0, false));
        assert_eq!(filter.filter_at(-0.1, t0 + 0.7), (-0.1, true));
    }

    #[test]
    fn startup_exact_zero_needs_confirm() {
        let mut filter = AngleReliability::default();
        let t0 = 100.0;
        assert_eq!(filter.filter_at(0.0, t0), (0.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 0.8), (0.0, true));
    }

    #[test]
    fn any_angle_to_zero_needs_confirm() {
        let mut filter = AngleReliability::default();
        filter.config.zero_confirm_duration_s = 0.65;
        let t0 = 100.0;
        assert_eq!(filter.filter_at(-70.0, t0), (-70.0, true));
        assert_eq!(filter.filter_at(0.0, t0 + 0.1), (-70.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 0.5), (-70.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 0.8), (0.0, true));
    }

    #[test]
    fn small_angle_to_zero_needs_confirm() {
        let mut filter = AngleReliability::default();
        filter.config.zero_confirm_duration_s = 0.65;
        let t0 = 100.0;
        assert_eq!(filter.filter_at(5.0, t0), (5.0, true));
        assert_eq!(filter.filter_at(0.0, t0 + 0.1), (5.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 0.5), (5.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 0.8), (0.0, true));
    }

    #[test]
    fn normal_a_to_b_is_not_delayed() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(-70.0), (-70.0, true));
        assert_eq!(filter.filter(-55.0), (-55.0, true));
        assert_eq!(filter.filter(-20.0), (-20.0, true));
    }

    #[test]
    fn zero_interrupted_resets_count() {
        let mut filter = AngleReliability::default();
        filter.config.zero_confirm_duration_s = 0.65;
        let t0 = 100.0;
        assert_eq!(filter.filter_at(-70.0, t0), (-70.0, true));
        assert_eq!(filter.filter_at(0.0, t0 + 0.1), (-70.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 0.5), (-70.0, false));
        assert_eq!(filter.filter_at(-55.0, t0 + 0.55), (-55.0, true));
        assert_eq!(filter.filter_at(0.0, t0 + 0.6), (-55.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 1.1), (-55.0, false));
        assert_eq!(filter.filter_at(0.0, t0 + 1.3), (0.0, true));
    }
}
