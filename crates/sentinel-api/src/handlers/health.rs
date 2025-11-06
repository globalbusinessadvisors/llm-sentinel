//! Health check endpoints.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error};

use crate::{ErrorResponse, SuccessResponse};

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: ServiceStatus,
    /// Service version
    pub version: String,
    /// Component health
    pub components: Vec<ComponentHealth>,
}

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Status
    pub status: ServiceStatus,
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ComponentHealth {
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ServiceStatus::Healthy,
            error: None,
        }
    }

    pub fn unhealthy(name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: ServiceStatus::Unhealthy,
            error: Some(error.into()),
        }
    }
}

/// Application state for health checks
#[derive(Clone)]
pub struct HealthState {
    pub version: String,
    pub storage_health: Arc<dyn Fn() -> Result<(), String> + Send + Sync>,
}

impl HealthState {
    pub fn new(
        version: String,
        storage_health: Arc<dyn Fn() -> Result<(), String> + Send + Sync>,
    ) -> Self {
        Self {
            version,
            storage_health,
        }
    }
}

/// Liveness probe - returns 200 if service is running
pub async fn liveness() -> StatusCode {
    debug!("Liveness probe called");
    StatusCode::OK
}

/// Readiness probe - returns 200 if service is ready to accept traffic
pub async fn readiness(
    State(state): State<Arc<HealthState>>,
) -> Result<Json<SuccessResponse<HealthResponse>>, (StatusCode, Json<ErrorResponse>)> {
    debug!("Readiness probe called");

    let mut components = Vec::new();
    let mut overall_status = ServiceStatus::Healthy;

    // Check storage
    match (state.storage_health)() {
        Ok(_) => {
            components.push(ComponentHealth::healthy("storage"));
        }
        Err(e) => {
            error!("Storage health check failed: {}", e);
            components.push(ComponentHealth::unhealthy("storage", e));
            overall_status = ServiceStatus::Unhealthy;
        }
    }

    let response = HealthResponse {
        status: overall_status,
        version: state.version.clone(),
        components,
    };

    if overall_status == ServiceStatus::Unhealthy {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new("unhealthy", "Service is unhealthy")),
        ));
    }

    Ok(Json(SuccessResponse::new(response)))
}

/// Full health check with all component statuses
pub async fn health(
    State(state): State<Arc<HealthState>>,
) -> Json<SuccessResponse<HealthResponse>> {
    debug!("Health check called");

    let mut components = Vec::new();
    let mut overall_status = ServiceStatus::Healthy;

    // Check storage
    match (state.storage_health)() {
        Ok(_) => {
            components.push(ComponentHealth::healthy("storage"));
        }
        Err(e) => {
            error!("Storage health check failed: {}", e);
            components.push(ComponentHealth::unhealthy("storage", e));
            overall_status = ServiceStatus::Degraded;
        }
    }

    let response = HealthResponse {
        status: overall_status,
        version: state.version.clone(),
        components,
    };

    Json(SuccessResponse::new(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_health_creation() {
        let healthy = ComponentHealth::healthy("test");
        assert_eq!(healthy.status, ServiceStatus::Healthy);
        assert!(healthy.error.is_none());

        let unhealthy = ComponentHealth::unhealthy("test", "error message");
        assert_eq!(unhealthy.status, ServiceStatus::Unhealthy);
        assert_eq!(unhealthy.error, Some("error message".to_string()));
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: ServiceStatus::Healthy,
            version: "0.1.0".to_string(),
            components: vec![ComponentHealth::healthy("storage")],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"0.1.0\""));
    }
}
