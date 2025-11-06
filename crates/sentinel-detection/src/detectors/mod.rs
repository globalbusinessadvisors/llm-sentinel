//! Anomaly detection implementations.

pub mod cusum;
pub mod iqr;
pub mod mad;
pub mod zscore;

/// Common detection configuration
#[derive(Debug, Clone)]
pub struct DetectionConfig {
    /// Minimum samples required before detection
    pub min_samples: usize,
    /// Update baseline with every event
    pub update_baseline: bool,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            min_samples: 10,
            update_baseline: true,
        }
    }
}
