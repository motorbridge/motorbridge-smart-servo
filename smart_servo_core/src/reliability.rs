#[derive(Debug, Clone, Copy)]
pub struct AngleReliabilityConfig {
    pub zero_eps_deg: f32,
    pub zero_jump_min_deg: f32,
    pub zero_confirm_samples: u8,
}

impl Default for AngleReliabilityConfig {
    fn default() -> Self {
        Self {
            zero_eps_deg: 1.0,
            zero_jump_min_deg: 20.0,
            zero_confirm_samples: 30,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AngleReliabilityState {
    pub last_good_deg: Option<f32>,
    pub zero_bridge_active: bool,
    pub zero_candidate_count: u8,
}

#[derive(Debug, Clone)]
pub struct AngleReliability {
    pub config: AngleReliabilityConfig,
    pub state: AngleReliabilityState,
}

impl Default for AngleReliability {
    fn default() -> Self {
        Self {
            config: AngleReliabilityConfig::default(),
            state: AngleReliabilityState::default(),
        }
    }
}

impl AngleReliability {
    pub fn filter(&mut self, raw_deg: f32) -> (f32, bool) {
        let raw_is_zero = raw_deg.abs() <= self.config.zero_eps_deg;

        let Some(last_good) = self.state.last_good_deg else {
            if raw_is_zero {
                return (raw_deg, false);
            }
            self.state.last_good_deg = Some(raw_deg);
            return (raw_deg, true);
        };

        let zero_bridge = raw_is_zero && last_good.abs() >= self.config.zero_jump_min_deg;

        if zero_bridge {
            self.state.zero_bridge_active = true;
            self.state.zero_candidate_count = self.state.zero_candidate_count.saturating_add(1);
            if self.state.zero_candidate_count >= self.config.zero_confirm_samples {
                self.state.zero_bridge_active = false;
                self.state.last_good_deg = Some(raw_deg);
                return (raw_deg, true);
            }
            return (last_good, false);
        }

        if self.state.zero_bridge_active && raw_is_zero {
            self.state.zero_candidate_count = self.state.zero_candidate_count.saturating_add(1);
            if self.state.zero_candidate_count >= self.config.zero_confirm_samples {
                self.state.zero_bridge_active = false;
                self.state.last_good_deg = Some(raw_deg);
                return (raw_deg, true);
            }
            return (last_good, false);
        }

        if raw_is_zero {
            return (last_good, false);
        }

        self.state.zero_bridge_active = false;
        self.state.zero_candidate_count = 0;
        self.state.last_good_deg = Some(raw_deg);
        (raw_deg, true)
    }
}

#[cfg(test)]
mod tests {
    use super::AngleReliability;

    #[test]
    fn zero_does_not_become_good_before_nonzero_angle() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(0.0), (0.0, false));
        assert_eq!(filter.filter(0.0), (0.0, false));
        assert_eq!(filter.filter(-12.6), (-12.6, true));
    }

    #[test]
    fn suppresses_a_zero_b_bridge() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(-70.0), (-70.0, true));
        assert_eq!(filter.filter(0.0), (-70.0, false));
        assert_eq!(filter.filter(0.0), (-70.0, false));
        assert_eq!(filter.filter(-55.0), (-55.0, true));
    }

    #[test]
    fn confirmed_zero_can_become_real_angle() {
        let mut filter = AngleReliability::default();
        filter.config.zero_confirm_samples = 3;
        assert_eq!(filter.filter(-70.0), (-70.0, true));
        assert_eq!(filter.filter(0.0), (-70.0, false));
        assert_eq!(filter.filter(0.0), (-70.0, false));
        assert_eq!(filter.filter(0.0), (0.0, true));
    }

    #[test]
    fn normal_a_to_b_is_not_delayed() {
        let mut filter = AngleReliability::default();
        assert_eq!(filter.filter(-70.0), (-70.0, true));
        assert_eq!(filter.filter(-55.0), (-55.0, true));
        assert_eq!(filter.filter(-20.0), (-20.0, true));
    }
}
