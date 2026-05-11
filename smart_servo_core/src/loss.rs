use std::collections::HashMap;

use crate::{Result, ServoId, SmartServoError};

/// Per-servo consecutive response loss tracker.
///
/// Calls [`record_ok`] on success and [`record_miss`] on timeout.
/// Returns [`SmartServoError::ConsecutiveLoss`] when a servo's consecutive
/// miss count reaches `threshold`. A `threshold` of `0` disables the check.
#[derive(Debug, Clone)]
pub struct LossTracker {
    counts: HashMap<ServoId, u32>,
    threshold: u32,
}

impl LossTracker {
    pub fn new(threshold: u32) -> Self {
        Self {
            counts: HashMap::new(),
            threshold,
        }
    }

    /// Reset miss counter for `id` after a successful response.
    pub fn record_ok(&mut self, id: ServoId) {
        self.counts.insert(id, 0);
    }

    /// Increment miss counter for `id`. Returns `Err(ConsecutiveLoss)` when
    /// `threshold > 0` and the count reaches the threshold.
    pub fn record_miss(&mut self, id: ServoId) -> Result<()> {
        let count = self.counts.entry(id).or_insert(0);
        *count += 1;
        if self.threshold > 0 && *count >= self.threshold {
            Err(SmartServoError::ConsecutiveLoss { id, count: *count })
        } else {
            Ok(())
        }
    }

    /// Current consecutive miss count for `id`.
    pub fn miss_count(&self, id: ServoId) -> u32 {
        self.counts.get(&id).copied().unwrap_or(0)
    }
}

impl Default for LossTracker {
    /// Default threshold: 0 disables hard-stop loss errors.
    ///
    /// Monitoring clients should keep returning held cached values with
    /// `reliable=false` during power loss. Applications that want a hard fault
    /// can opt in with `set_loss_threshold`.
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::LossTracker;
    use crate::SmartServoError;

    #[test]
    fn default_does_not_error_on_consecutive_misses() {
        let mut tracker = LossTracker::default();
        for _ in 0..100 {
            assert!(tracker.record_miss(1).is_ok());
        }
        assert_eq!(tracker.miss_count(1), 100);
    }

    #[test]
    fn explicit_threshold_errors_at_limit() {
        let mut tracker = LossTracker::new(2);
        assert!(tracker.record_miss(1).is_ok());
        let err = tracker.record_miss(1).unwrap_err();
        assert!(matches!(
            err,
            SmartServoError::ConsecutiveLoss { id: 1, count: 2 }
        ));
    }
}
