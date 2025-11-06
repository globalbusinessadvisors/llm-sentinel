//! Metrics definitions and constants for Sentinel.
//!
//! This module provides metric names and labels used throughout the system.

/// Metrics namespace
pub const METRICS_NAMESPACE: &str = "sentinel";

/// Counter metrics
pub mod counters {
    /// Total events ingested
    pub const EVENTS_INGESTED_TOTAL: &str = "events_ingested_total";

    /// Total events processed
    pub const EVENTS_PROCESSED_TOTAL: &str = "events_processed_total";

    /// Total events dropped
    pub const EVENTS_DROPPED_TOTAL: &str = "events_dropped_total";

    /// Total anomalies detected
    pub const ANOMALIES_DETECTED_TOTAL: &str = "anomalies_detected_total";

    /// Total alerts sent
    pub const ALERTS_SENT_TOTAL: &str = "alerts_sent_total";

    /// Total alerts failed
    pub const ALERTS_FAILED_TOTAL: &str = "alerts_failed_total";

    /// Total false positives
    pub const FALSE_POSITIVES_TOTAL: &str = "false_positives_total";

    /// Total errors
    pub const ERRORS_TOTAL: &str = "errors_total";
}

/// Histogram metrics
pub mod histograms {
    /// Event processing latency
    pub const EVENT_PROCESSING_DURATION_SECONDS: &str = "event_processing_duration_seconds";

    /// Detection latency
    pub const DETECTION_DURATION_SECONDS: &str = "detection_duration_seconds";

    /// Alert delivery latency
    pub const ALERT_DELIVERY_DURATION_SECONDS: &str = "alert_delivery_duration_seconds";

    /// LLM request latency from telemetry
    pub const LLM_REQUEST_LATENCY_MS: &str = "llm_request_latency_ms";

    /// LLM token count from telemetry
    pub const LLM_TOKEN_COUNT: &str = "llm_token_count";

    /// LLM cost from telemetry
    pub const LLM_COST_USD: &str = "llm_cost_usd";
}

/// Gauge metrics
pub mod gauges {
    /// Current queue depth
    pub const QUEUE_DEPTH: &str = "queue_depth";

    /// Active workers
    pub const ACTIVE_WORKERS: &str = "active_workers";

    /// Cache hit rate
    pub const CACHE_HIT_RATE: &str = "cache_hit_rate";

    /// Current anomaly rate (events/second)
    pub const ANOMALY_RATE: &str = "anomaly_rate";

    /// Current event rate (events/second)
    pub const EVENT_RATE: &str = "event_rate";

    /// Detection engine health (0-1)
    pub const DETECTION_ENGINE_HEALTH: &str = "detection_engine_health";
}

/// Metric labels
pub mod labels {
    /// Service name label
    pub const SERVICE: &str = "service";

    /// Model label
    pub const MODEL: &str = "model";

    /// Severity label
    pub const SEVERITY: &str = "severity";

    /// Anomaly type label
    pub const ANOMALY_TYPE: &str = "anomaly_type";

    /// Detection method label
    pub const METHOD: &str = "method";

    /// Engine type label
    pub const ENGINE: &str = "engine";

    /// Status label
    pub const STATUS: &str = "status";

    /// Error type label
    pub const ERROR_TYPE: &str = "error_type";
}

/// Histogram buckets for latency metrics (in seconds)
pub const LATENCY_BUCKETS: &[f64] = &[
    0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.5, 5.0, 10.0,
];

/// Histogram buckets for LLM latency (in milliseconds)
pub const LLM_LATENCY_BUCKETS: &[f64] = &[
    10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0, 30000.0, 60000.0,
];

/// Histogram buckets for token counts
pub const TOKEN_BUCKETS: &[f64] = &[
    10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0, 32000.0, 64000.0,
];

/// Histogram buckets for costs (in USD)
pub const COST_BUCKETS: &[f64] = &[
    0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_constants() {
        assert_eq!(METRICS_NAMESPACE, "sentinel");
        assert!(!counters::EVENTS_INGESTED_TOTAL.is_empty());
        assert!(!histograms::EVENT_PROCESSING_DURATION_SECONDS.is_empty());
        assert!(!gauges::QUEUE_DEPTH.is_empty());
        assert!(!labels::SERVICE.is_empty());
    }

    #[test]
    fn test_bucket_definitions() {
        assert!(!LATENCY_BUCKETS.is_empty());
        assert!(!LLM_LATENCY_BUCKETS.is_empty());
        assert!(!TOKEN_BUCKETS.is_empty());
        assert!(!COST_BUCKETS.is_empty());

        // Verify buckets are sorted
        for window in LATENCY_BUCKETS.windows(2) {
            assert!(window[0] < window[1]);
        }
    }
}
