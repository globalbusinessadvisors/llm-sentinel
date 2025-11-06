//! Error types for Sentinel operations.
//!
//! This module provides a comprehensive error hierarchy for all Sentinel operations,
//! with proper error context and conversion support.

use std::fmt;

/// Result type alias for Sentinel operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Sentinel operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Network/connection errors
    #[error("Connection error: {0}")]
    Connection(String),

    /// Database/storage errors
    #[error("Storage error: {0}")]
    Storage(String),

    /// Ingestion errors
    #[error("Ingestion error: {0}")]
    Ingestion(String),

    /// Detection errors
    #[error("Detection error: {0}")]
    Detection(String),

    /// Alerting errors
    #[error("Alerting error: {0}")]
    Alerting(String),

    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// Not found errors
    #[error("{0} not found")]
    NotFound(String),

    /// Already exists errors
    #[error("{0} already exists")]
    AlreadyExists(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Rate limit errors
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Generic errors with context
    #[error("{context}: {source}")]
    WithContext {
        /// Error context
        context: String,
        /// Source error
        source: Box<Error>,
    },
}

impl Error {
    /// Add context to an error
    pub fn context<C: fmt::Display>(self, context: C) -> Self {
        Error::WithContext {
            context: context.to_string(),
            source: Box::new(self),
        }
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Error::Config(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Error::Validation(msg.into())
    }

    /// Create a connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Error::Connection(msg.into())
    }

    /// Create a storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Error::Storage(msg.into())
    }

    /// Create an ingestion error
    pub fn ingestion<S: Into<String>>(msg: S) -> Self {
        Error::Ingestion(msg.into())
    }

    /// Create a detection error
    pub fn detection<S: Into<String>>(msg: S) -> Self {
        Error::Detection(msg.into())
    }

    /// Create an alerting error
    pub fn alerting<S: Into<String>>(msg: S) -> Self {
        Error::Alerting(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Error::Internal(msg.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(item: S) -> Self {
        Error::NotFound(item.into())
    }

    /// Create an already exists error
    pub fn already_exists<S: Into<String>>(item: S) -> Self {
        Error::AlreadyExists(item.into())
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Error::Timeout(msg.into())
    }

    /// Create a rate limit error
    pub fn rate_limit<S: Into<String>>(msg: S) -> Self {
        Error::RateLimit(msg.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Connection(_) | Error::Timeout(_) | Error::Storage(_)
        )
    }

    /// Check if error is transient
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Error::Connection(_) | Error::Timeout(_) | Error::RateLimit(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test error");
        assert!(matches!(err, Error::Config(_)));
    }

    #[test]
    fn test_error_context() {
        let err = Error::storage("database error").context("Failed to save event");
        assert!(matches!(err, Error::WithContext { .. }));
    }

    #[test]
    fn test_error_retryable() {
        assert!(Error::connection("test").is_retryable());
        assert!(Error::timeout("test").is_retryable());
        assert!(!Error::validation("test").is_retryable());
    }

    #[test]
    fn test_error_transient() {
        assert!(Error::connection("test").is_transient());
        assert!(Error::rate_limit("test").is_transient());
        assert!(!Error::validation("test").is_transient());
    }
}
