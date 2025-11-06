//! Configuration management for Sentinel.
//!
//! This module provides configuration structures and loading from files/env.

use crate::error::Result;
use figment::{
    providers::{Env, Format, Toml, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use validator::Validate;

/// Main Sentinel configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Ingestion configuration
    pub ingestion: IngestionConfig,

    /// Detection configuration
    pub detection: DetectionConfig,

    /// Alerting configuration
    pub alerting: AlertingConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    /// Server host
    #[validate(length(min = 1))]
    pub host: String,

    /// Server port
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,

    /// Worker threads (0 = number of CPU cores)
    pub worker_threads: usize,

    /// Request timeout in seconds
    #[validate(range(min = 1))]
    pub request_timeout_secs: u64,

    /// Graceful shutdown timeout in seconds
    #[validate(range(min = 1))]
    pub shutdown_timeout_secs: u64,
}

/// Ingestion configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct IngestionConfig {
    /// Kafka configuration
    pub kafka: Option<KafkaConfig>,

    /// gRPC configuration
    pub grpc: Option<GrpcConfig>,

    /// Buffer size for incoming events
    #[validate(range(min = 100))]
    pub buffer_size: usize,

    /// Batch size for processing
    #[validate(range(min = 1))]
    pub batch_size: usize,

    /// Batch timeout in milliseconds
    #[validate(range(min = 1))]
    pub batch_timeout_ms: u64,
}

/// Kafka configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct KafkaConfig {
    /// Kafka brokers
    #[validate(length(min = 1))]
    pub brokers: Vec<String>,

    /// Topic to consume from
    #[validate(length(min = 1))]
    pub topic: String,

    /// Consumer group ID
    #[validate(length(min = 1))]
    pub consumer_group: String,

    /// Auto offset reset (earliest, latest)
    pub auto_offset_reset: String,

    /// Enable auto commit
    pub enable_auto_commit: bool,

    /// Session timeout in milliseconds
    #[validate(range(min = 1000))]
    pub session_timeout_ms: u32,
}

/// gRPC configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GrpcConfig {
    /// gRPC server address
    #[validate(length(min = 1))]
    pub address: String,

    /// Enable TLS
    pub enable_tls: bool,

    /// TLS certificate path
    pub cert_path: Option<String>,

    /// TLS key path
    pub key_path: Option<String>,
}

/// Detection configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DetectionConfig {
    /// Detection engines to enable
    pub engines: Vec<DetectionEngineConfig>,

    /// Number of detection workers
    #[validate(range(min = 1))]
    pub workers: usize,

    /// Detection timeout in milliseconds
    #[validate(range(min = 10))]
    pub timeout_ms: u64,

    /// Enable ML detection
    pub enable_ml: bool,

    /// ML model update interval in seconds
    #[validate(range(min = 60))]
    pub model_update_interval_secs: u64,
}

/// Detection engine configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DetectionEngineConfig {
    /// Engine type (statistical, ml, llm)
    #[validate(length(min = 1))]
    pub engine_type: String,

    /// Detection methods to enable
    pub methods: Vec<String>,

    /// Engine-specific settings
    pub settings: serde_json::Value,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AlertingConfig {
    /// RabbitMQ configuration
    pub rabbitmq: Option<RabbitMqConfig>,

    /// Webhook configuration
    pub webhook: Option<WebhookConfig>,

    /// Deduplication window in seconds
    #[validate(range(min = 1))]
    pub dedup_window_secs: u64,

    /// Alert batch size
    #[validate(range(min = 1))]
    pub batch_size: usize,

    /// Alert batch timeout in milliseconds
    #[validate(range(min = 100))]
    pub batch_timeout_ms: u64,
}

/// RabbitMQ configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RabbitMqConfig {
    /// RabbitMQ URL
    #[validate(length(min = 1))]
    pub url: String,

    /// Exchange name
    #[validate(length(min = 1))]
    pub exchange: String,

    /// Exchange type (topic, direct, fanout)
    pub exchange_type: String,

    /// Durable exchange
    pub durable: bool,

    /// Retry attempts
    #[validate(range(min = 0))]
    pub retry_attempts: u32,

    /// Retry delay in milliseconds
    #[validate(range(min = 100))]
    pub retry_delay_ms: u64,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WebhookConfig {
    /// Webhook URL
    #[validate(url)]
    pub url: String,

    /// Timeout in seconds
    #[validate(range(min = 1))]
    pub timeout_secs: u64,

    /// Retry attempts
    #[validate(range(min = 0))]
    pub retry_attempts: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StorageConfig {
    /// InfluxDB configuration
    pub influxdb: Option<InfluxDbConfig>,

    /// Redis configuration
    pub redis: Option<RedisConfig>,

    /// Cache configuration
    pub cache: CacheConfig,
}

/// InfluxDB configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InfluxDbConfig {
    /// InfluxDB URL
    #[validate(url)]
    pub url: String,

    /// Organization
    #[validate(length(min = 1))]
    pub org: String,

    /// Bucket name
    #[validate(length(min = 1))]
    pub bucket: String,

    /// Auth token
    #[validate(length(min = 1))]
    pub token: String,

    /// Connection timeout in seconds
    #[validate(range(min = 1))]
    pub timeout_secs: u64,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RedisConfig {
    /// Redis URL
    #[validate(length(min = 1))]
    pub url: String,

    /// Enable cluster mode
    pub cluster: bool,

    /// Connection pool size
    #[validate(range(min = 1))]
    pub pool_size: usize,

    /// Connection timeout in seconds
    #[validate(range(min = 1))]
    pub timeout_secs: u64,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheConfig {
    /// Cache type (moka, redis)
    #[validate(length(min = 1))]
    pub cache_type: String,

    /// Max capacity
    #[validate(range(min = 100))]
    pub max_capacity: usize,

    /// TTL in seconds
    #[validate(range(min = 1))]
    pub ttl_secs: u64,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ObservabilityConfig {
    /// Enable metrics
    pub enable_metrics: bool,

    /// Metrics port
    #[validate(range(min = 1, max = 65535))]
    pub metrics_port: u16,

    /// Enable tracing
    pub enable_tracing: bool,

    /// Tracing endpoint
    pub tracing_endpoint: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Log format (json, text)
    pub log_format: String,
}

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Figment::new()
            .merge(Yaml::file(path))
            .merge(Env::prefixed("SENTINEL_"))
            .extract()
            .map_err(|e| crate::Error::config(format!("Failed to load config: {}", e)))?;

        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn from_toml<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("SENTINEL_"))
            .extract()
            .map_err(|e| crate::Error::config(format!("Failed to load config: {}", e)))?;

        Ok(config)
    }

    /// Create default configuration for testing
    pub fn default_test() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                worker_threads: 4,
                request_timeout_secs: 30,
                shutdown_timeout_secs: 10,
            },
            ingestion: IngestionConfig {
                kafka: Some(KafkaConfig {
                    brokers: vec!["localhost:9092".to_string()],
                    topic: "llm.telemetry".to_string(),
                    consumer_group: "sentinel-anomaly".to_string(),
                    auto_offset_reset: "latest".to_string(),
                    enable_auto_commit: true,
                    session_timeout_ms: 30000,
                }),
                grpc: None,
                buffer_size: 10000,
                batch_size: 100,
                batch_timeout_ms: 1000,
            },
            detection: DetectionConfig {
                engines: vec![DetectionEngineConfig {
                    engine_type: "statistical".to_string(),
                    methods: vec!["z_score".to_string(), "iqr".to_string()],
                    settings: serde_json::json!({
                        "window_size": 1000,
                        "threshold_sigma": 3.0
                    }),
                }],
                workers: 4,
                timeout_ms: 500,
                enable_ml: false,
                model_update_interval_secs: 3600,
            },
            alerting: AlertingConfig {
                rabbitmq: Some(RabbitMqConfig {
                    url: "amqp://localhost:5672".to_string(),
                    exchange: "incidents".to_string(),
                    exchange_type: "topic".to_string(),
                    durable: true,
                    retry_attempts: 3,
                    retry_delay_ms: 1000,
                }),
                webhook: None,
                dedup_window_secs: 300,
                batch_size: 10,
                batch_timeout_ms: 1000,
            },
            storage: StorageConfig {
                influxdb: Some(InfluxDbConfig {
                    url: "http://localhost:8086".to_string(),
                    org: "sentinel".to_string(),
                    bucket: "sentinel-metrics".to_string(),
                    token: "test-token".to_string(),
                    timeout_secs: 10,
                }),
                redis: None,
                cache: CacheConfig {
                    cache_type: "moka".to_string(),
                    max_capacity: 10000,
                    ttl_secs: 300,
                },
            },
            observability: ObservabilityConfig {
                enable_metrics: true,
                metrics_port: 9090,
                enable_tracing: true,
                tracing_endpoint: Some("http://localhost:4317".to_string()),
                log_level: "info".to_string(),
                log_format: "json".to_string(),
            },
        }
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        self.validate()
            .map_err(|e| crate::Error::validation(format!("Config validation failed: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = Config::default_test();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default_test();
        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default_test();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.server.port, deserialized.server.port);
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = Config::default_test();
        config.server.port = 0; // Invalid port

        assert!(config.validate_config().is_err());
    }
}
