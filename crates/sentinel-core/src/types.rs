//! Core type definitions for Sentinel.
//!
//! This module provides fundamental types used throughout the Sentinel system.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level for anomalies and alerts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Low severity - informational
    Low,
    /// Medium severity - warning
    Medium,
    /// High severity - requires attention
    High,
    /// Critical severity - requires immediate action
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Low => write!(f, "low"),
            Severity::Medium => write!(f, "medium"),
            Severity::High => write!(f, "high"),
            Severity::Critical => write!(f, "critical"),
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Medium
    }
}

/// Type of anomaly detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    /// Latency spike detected
    LatencySpike,
    /// Throughput degradation
    ThroughputDegradation,
    /// Error rate increase
    ErrorRateIncrease,
    /// Token usage spike
    TokenUsageSpike,
    /// Cost anomaly
    CostAnomaly,
    /// Input distribution drift
    InputDrift,
    /// Output distribution drift
    OutputDrift,
    /// Concept drift
    ConceptDrift,
    /// Embedding drift
    EmbeddingDrift,
    /// Hallucination detected
    Hallucination,
    /// Quality degradation
    QualityDegradation,
    /// Security threat
    SecurityThreat,
    /// Custom anomaly type
    Custom(String),
}

impl fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnomalyType::LatencySpike => write!(f, "latency_spike"),
            AnomalyType::ThroughputDegradation => write!(f, "throughput_degradation"),
            AnomalyType::ErrorRateIncrease => write!(f, "error_rate_increase"),
            AnomalyType::TokenUsageSpike => write!(f, "token_usage_spike"),
            AnomalyType::CostAnomaly => write!(f, "cost_anomaly"),
            AnomalyType::InputDrift => write!(f, "input_drift"),
            AnomalyType::OutputDrift => write!(f, "output_drift"),
            AnomalyType::ConceptDrift => write!(f, "concept_drift"),
            AnomalyType::EmbeddingDrift => write!(f, "embedding_drift"),
            AnomalyType::Hallucination => write!(f, "hallucination"),
            AnomalyType::QualityDegradation => write!(f, "quality_degradation"),
            AnomalyType::SecurityThreat => write!(f, "security_threat"),
            AnomalyType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Detection method used to identify anomaly
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionMethod {
    /// Z-Score statistical method
    ZScore,
    /// Interquartile Range (IQR)
    Iqr,
    /// Median Absolute Deviation
    Mad,
    /// Cumulative Sum (CUSUM)
    Cusum,
    /// Isolation Forest ML algorithm
    IsolationForest,
    /// LSTM Autoencoder
    LstmAutoencoder,
    /// One-Class SVM
    OneClassSvm,
    /// Population Stability Index
    Psi,
    /// KL Divergence
    KlDivergence,
    /// LLM-Check hallucination detection
    LlmCheck,
    /// RAG-based detection
    Rag,
    /// Custom detection method
    Custom(String),
}

impl fmt::Display for DetectionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DetectionMethod::ZScore => write!(f, "z_score"),
            DetectionMethod::Iqr => write!(f, "iqr"),
            DetectionMethod::Mad => write!(f, "mad"),
            DetectionMethod::Cusum => write!(f, "cusum"),
            DetectionMethod::IsolationForest => write!(f, "isolation_forest"),
            DetectionMethod::LstmAutoencoder => write!(f, "lstm_autoencoder"),
            DetectionMethod::OneClassSvm => write!(f, "one_class_svm"),
            DetectionMethod::Psi => write!(f, "psi"),
            DetectionMethod::KlDivergence => write!(f, "kl_divergence"),
            DetectionMethod::LlmCheck => write!(f, "llm_check"),
            DetectionMethod::Rag => write!(f, "rag"),
            DetectionMethod::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Service identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(String);

impl ServiceId {
    /// Create a new service ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the service ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ServiceId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ServiceId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Model identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(String);

impl ModelId {
    /// Create a new model ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the model ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ModelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ModelId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::High;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"high\"");

        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, severity);
    }

    #[test]
    fn test_anomaly_type_display() {
        assert_eq!(AnomalyType::LatencySpike.to_string(), "latency_spike");
        assert_eq!(AnomalyType::Custom("test".to_string()).to_string(), "test");
    }

    #[test]
    fn test_service_id_creation() {
        let id = ServiceId::new("test-service");
        assert_eq!(id.as_str(), "test-service");
        assert_eq!(id.to_string(), "test-service");
    }

    #[test]
    fn test_model_id_from_string() {
        let id: ModelId = "gpt-4".into();
        assert_eq!(id.as_str(), "gpt-4");
    }
}
