# LLM-Sentinel System Architecture

## Table of Contents

1. [System Overview](#system-overview)
2. [High-Level Architecture](#high-level-architecture)
3. [Core Components](#core-components)
4. [Data Schemas](#data-schemas)
5. [Processing Pipeline](#processing-pipeline)
6. [Deployment Topologies](#deployment-topologies)
7. [Integration Patterns](#integration-patterns)
8. [Scalability and Fault Tolerance](#scalability-and-fault-tolerance)

---

## System Overview

LLM-Sentinel is a real-time anomaly detection and alerting system for Large Language Model deployments. It monitors telemetry data from LLM-Observatory, detects anomalous behavior using configurable detection engines, and triggers appropriate responses through LLM-Shield and alerting mechanisms.

### Design Principles

- **Real-time Processing**: Sub-second latency for critical anomaly detection
- **Horizontal Scalability**: Support for distributed processing across multiple instances
- **Fault Tolerance**: No single point of failure, graceful degradation
- **Pluggable Detection**: Extensible anomaly detection engine architecture
- **Multi-tenancy**: Support for multiple LLM applications with isolated configurations
- **Observable**: Comprehensive metrics, logging, and tracing

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           LLM Ecosystem                                  │
│                                                                           │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐          │
│  │ LLM Services │─────▶│    LLM-      │─────▶│    LLM-      │          │
│  │              │      │ Observatory  │      │  Sentinel    │          │
│  └──────────────┘      └──────────────┘      └──────┬───────┘          │
│                                                      │                   │
│                              ┌───────────────────────┼──────────────┐   │
│                              │                       │              │   │
│                              ▼                       ▼              ▼   │
│                        ┌──────────┐          ┌──────────┐   ┌──────────┐│
│                        │   LLM-   │          │   LLM-   │   │Governance││
│                        │  Shield  │          │ Incident │   │Dashboard ││
│                        │          │          │ Manager  │   │          ││
│                        └──────────┘          └──────────┘   └──────────┘│
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Interactions

```
┌──────────────┐
│ Observatory  │ Telemetry Stream (gRPC/HTTP/Kafka)
└──────┬───────┘
       │
       ▼
┌────────────────────────────────────────────────────────┐
│                  LLM-SENTINEL                          │
│                                                        │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────┐ │
│  │  Telemetry  │───▶│   Anomaly    │───▶│  Alert   │ │
│  │  Ingestion  │    │   Detection  │    │ Manager  │ │
│  │   Service   │    │    Engine    │    │          │ │
│  └─────────────┘    └──────────────┘    └────┬─────┘ │
│         │                   │                 │       │
│         │                   │                 │       │
│  ┌──────▼───────────────────▼─────────────────▼────┐ │
│  │           Storage Layer (Time-Series DB)        │ │
│  └─────────────────────────────────────────────────┘ │
│                                                        │
│  ┌─────────────────────────────────────────────────┐ │
│  │        Configuration Service (etcd/Consul)      │ │
│  └─────────────────────────────────────────────────┘ │
│                                                        │
│  ┌─────────────────────────────────────────────────┐ │
│  │             API Gateway (REST/gRPC)             │ │
│  └─────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌──────────┐      ┌─────────────┐    ┌─────────────┐
│  Shield  │      │  Incident   │    │ Governance  │
│          │      │   Manager   │    │  Dashboard  │
└──────────┘      └─────────────┘    └─────────────┘
```

---

## Core Components

### 1. Telemetry Ingestion Service

**Responsibilities:**
- Receive telemetry events from LLM-Observatory
- Protocol translation (gRPC, HTTP, Kafka, AMQP)
- Data validation and normalization
- Rate limiting and backpressure management
- Buffering and batching for downstream processing

**Technical Stack:**
- Protocol: gRPC (primary), HTTP/2, Kafka consumer
- Language: Go or Rust for high performance
- Queue: In-memory ring buffer with disk overflow

**Architecture:**

```
┌────────────────────────────────────────────────────┐
│       Telemetry Ingestion Service                  │
│                                                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │  gRPC    │  │   HTTP   │  │  Kafka   │       │
│  │ Listener │  │ Listener │  │ Consumer │       │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘       │
│       │            │             │               │
│       └────────────┼─────────────┘               │
│                    ▼                              │
│         ┌──────────────────┐                     │
│         │   Validator &    │                     │
│         │   Normalizer     │                     │
│         └────────┬─────────┘                     │
│                  ▼                                │
│         ┌──────────────────┐                     │
│         │  Rate Limiter &  │                     │
│         │  Backpressure    │                     │
│         └────────┬─────────┘                     │
│                  ▼                                │
│         ┌──────────────────┐                     │
│         │  Ring Buffer     │                     │
│         │  (with overflow) │                     │
│         └────────┬─────────┘                     │
│                  ▼                                │
│         ┌──────────────────┐                     │
│         │   Event Router   │                     │
│         │  (to Detection)  │                     │
│         └──────────────────┘                     │
└────────────────────────────────────────────────────┘
```

**Configuration:**
```yaml
ingestion:
  grpc:
    port: 9090
    maxConcurrentStreams: 1000
    maxReceiveMessageSize: 4MB
  http:
    port: 8080
    maxBodySize: 1MB
  kafka:
    brokers: ["kafka-1:9092", "kafka-2:9092"]
    topics: ["llm-telemetry"]
    consumerGroup: "sentinel-ingest"
  buffer:
    size: 100000
    overflowToDisk: true
    overflowPath: /var/sentinel/overflow
  rateLimiting:
    enabled: true
    maxEventsPerSecond: 50000
    burstSize: 10000
```

### 2. Anomaly Detection Engine

**Responsibilities:**
- Real-time event processing
- Statistical anomaly detection
- Pattern matching and rule evaluation
- Machine learning model inference
- Correlation analysis across multiple events
- Anomaly scoring and prioritization

**Detection Strategies:**

1. **Statistical Detectors**
   - Z-score analysis
   - Moving average deviation
   - Percentile-based thresholds
   - Time-series forecasting (ARIMA, Prophet)

2. **Rule-Based Detectors**
   - Threshold violations
   - Pattern matching (regex, CEP)
   - Composite conditions
   - Time-window aggregations

3. **ML-Based Detectors**
   - Isolation Forest
   - Autoencoders
   - LSTM for sequence anomalies
   - Clustering (DBSCAN, K-means)

4. **Behavioral Detectors**
   - User behavior profiling
   - Entity relationship analysis
   - Access pattern anomalies

**Architecture:**

```
┌────────────────────────────────────────────────────┐
│         Anomaly Detection Engine                   │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │       Event Stream Processor             │    │
│  │    (Apache Flink / Kafka Streams)        │    │
│  └──────────────┬───────────────────────────┘    │
│                 │                                 │
│     ┌───────────┼───────────┐                    │
│     │           │           │                     │
│     ▼           ▼           ▼                     │
│ ┌────────┐ ┌────────┐ ┌────────┐                │
│ │Statistical│Rule-Based│ML-Based │               │
│ │Detector │ │Detector│ │Detector│                │
│ └───┬────┘ └───┬────┘ └───┬────┘                │
│     │          │          │                       │
│     └──────────┼──────────┘                       │
│                ▼                                   │
│      ┌──────────────────┐                        │
│      │  Correlation &   │                        │
│      │   Aggregation    │                        │
│      └────────┬─────────┘                        │
│               ▼                                   │
│      ┌──────────────────┐                        │
│      │     Scoring &    │                        │
│      │  Prioritization  │                        │
│      └────────┬─────────┘                        │
│               ▼                                   │
│      ┌──────────────────┐                        │
│      │  Anomaly Event   │                        │
│      │    Publisher     │                        │
│      └──────────────────┘                        │
└────────────────────────────────────────────────────┘
```

**Detector Configuration Example:**

```yaml
detectors:
  - id: token-rate-spike
    type: statistical
    strategy: zscore
    config:
      metric: tokens_per_second
      window: 5m
      threshold: 3.0
      sensitivity: high
    actions:
      - alert: token-rate-anomaly
        severity: warning

  - id: cost-threshold
    type: rule
    strategy: threshold
    config:
      metric: cost_per_request
      condition: greater_than
      value: 1.0
      window: 1m
      consecutive: 3
    actions:
      - alert: high-cost-request
        severity: critical
      - shield: rate_limit

  - id: prompt-injection
    type: ml
    strategy: classifier
    config:
      model: prompt-injection-detector-v2
      threshold: 0.85
      features: ["prompt_text", "embedding"]
    actions:
      - alert: security-threat
        severity: critical
      - shield: block
```

### 3. Alert Manager

**Responsibilities:**
- Alert routing and notification
- Alert deduplication and grouping
- Escalation policies
- Notification channel management
- Alert suppression and maintenance windows
- Alert history and acknowledgment

**Architecture:**

```
┌────────────────────────────────────────────────────┐
│            Alert Manager                           │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │       Alert Receiver                     │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │    Deduplication & Grouping              │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │    Routing & Escalation Engine           │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │    Notification Dispatcher               │    │
│  │  ┌─────┐ ┌──────┐ ┌──────┐ ┌────────┐   │    │
│  │  │Email│ │Slack │ │PagerD│ │Webhook │   │    │
│  │  └─────┘ └──────┘ └──────┘ └────────┘   │    │
│  └──────────────────────────────────────────┘    │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │      Alert History & State Store         │    │
│  └──────────────────────────────────────────┘    │
└────────────────────────────────────────────────────┘
```

**Alert Routing Configuration:**

```yaml
alerting:
  routing:
    - match:
        severity: critical
      receivers:
        - pagerduty-oncall
        - slack-incidents
      repeat_interval: 5m

    - match:
        severity: warning
        team: ml-platform
      receivers:
        - slack-ml-platform
      repeat_interval: 1h

  receivers:
    - name: pagerduty-oncall
      type: pagerduty
      config:
        serviceKey: ${PAGERDUTY_SERVICE_KEY}

    - name: slack-incidents
      type: slack
      config:
        webhookUrl: ${SLACK_WEBHOOK_URL}
        channel: "#incidents"

  grouping:
    groupBy: ["application", "detector_id"]
    groupWait: 30s
    groupInterval: 5m

  suppression:
    maintenanceWindows:
      - name: weekly-maintenance
        schedule: "0 2 * * 0"  # Sunday 2 AM
        duration: 2h
```

### 4. Configuration Service

**Responsibilities:**
- Centralized configuration management
- Dynamic configuration updates
- Configuration versioning
- Multi-tenant configuration isolation
- Configuration validation
- Feature flags and gradual rollouts

**Architecture:**

```
┌────────────────────────────────────────────────────┐
│         Configuration Service                      │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │         Config API (gRPC/REST)           │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │      Validation & Schema Engine          │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │    Version Control & Rollback            │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │  Backend Store (etcd/Consul/DynamoDB)    │    │
│  └──────────────┬───────────────────────────┘    │
│                 │                                 │
│  ┌──────────────▼───────────────────────────┐    │
│  │     Change Notification (Watch API)      │    │
│  └──────────────────────────────────────────┘    │
└────────────────────────────────────────────────────┘
```

**Configuration Structure:**

```yaml
# /config/tenants/{tenant_id}/detectors
detectors:
  version: v1.2.0
  lastUpdated: "2025-11-06T12:00:00Z"
  items:
    - id: token-rate-spike
      enabled: true
      # ... detector config

# /config/tenants/{tenant_id}/alerts
alerts:
  version: v1.1.0
  routing:
    # ... routing config

# /config/global/features
features:
  ml_detectors:
    enabled: true
    rollout: 50  # percentage
  advanced_correlation:
    enabled: false
```

### 5. Storage Layer

**Responsibilities:**
- Time-series data storage for metrics
- Event storage for audit trail
- Alert history persistence
- Configuration backup
- Query optimization for dashboards

**Storage Architecture:**

```
┌────────────────────────────────────────────────────┐
│              Storage Layer                         │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │   Time-Series Database                   │    │
│  │   (InfluxDB / TimescaleDB / Prometheus)  │    │
│  │   - Telemetry metrics                    │    │
│  │   - Aggregated statistics                │    │
│  │   - Detection scores                     │    │
│  └──────────────────────────────────────────┘    │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │   Event Store                            │    │
│  │   (Elasticsearch / PostgreSQL)           │    │
│  │   - Raw telemetry events                 │    │
│  │   - Anomaly events                       │    │
│  │   - Audit logs                           │    │
│  └──────────────────────────────────────────┘    │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │   Alert Store                            │    │
│  │   (PostgreSQL / MongoDB)                 │    │
│  │   - Alert history                        │    │
│  │   - Alert state                          │    │
│  │   - Acknowledgments                      │    │
│  └──────────────────────────────────────────┘    │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │   Object Storage                         │    │
│  │   (S3 / MinIO)                           │    │
│  │   - ML models                            │    │
│  │   - Configuration backups                │    │
│  │   - Long-term archives                   │    │
│  └──────────────────────────────────────────┘    │
└────────────────────────────────────────────────────┘
```

**Data Retention Policies:**

```yaml
retention:
  raw_telemetry:
    hot: 7d      # Full-resolution
    warm: 30d    # 1-minute rollup
    cold: 90d    # 5-minute rollup
    archive: 1y  # 1-hour rollup (S3)

  anomaly_events:
    hot: 30d
    warm: 90d
    archive: 2y

  alerts:
    active: infinite
    resolved: 1y

  audit_logs:
    hot: 90d
    archive: 7y  # compliance requirement
```

### 6. API Gateway

**Responsibilities:**
- Unified API endpoint for external systems
- Authentication and authorization
- Rate limiting and quota management
- API versioning
- Request/response transformation
- API documentation (OpenAPI/gRPC reflection)

**Architecture:**

```
┌────────────────────────────────────────────────────┐
│              API Gateway                           │
│                                                    │
│  ┌──────────────────────────────────────────┐    │
│  │   Protocol Layer (REST/gRPC/GraphQL)     │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │   Authentication & Authorization         │    │
│  │   (JWT / OAuth2 / mTLS)                  │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │   Rate Limiting & Quotas                 │    │
│  └──────────────┬───────────────────────────┘    │
│                 ▼                                 │
│  ┌──────────────────────────────────────────┐    │
│  │        Request Router                    │    │
│  └──────────────┬───────────────────────────┘    │
│                 │                                 │
│     ┌───────────┼───────────┐                    │
│     ▼           ▼           ▼                     │
│ ┌────────┐ ┌────────┐ ┌────────┐                │
│ │Telemetry│ │Anomaly│ │Config │                 │
│ │  API   │ │  API  │ │  API  │                  │
│ └────────┘ └────────┘ └────────┘                 │
└────────────────────────────────────────────────────┘
```

**API Endpoints:**

```yaml
# REST API v1
/api/v1/telemetry
  POST   /events              # Ingest telemetry events
  GET    /metrics             # Query metrics

/api/v1/anomalies
  GET    /                    # List anomalies
  GET    /{id}                # Get anomaly details
  POST   /{id}/acknowledge    # Acknowledge anomaly

/api/v1/alerts
  GET    /                    # List alerts
  GET    /{id}                # Get alert details
  PUT    /{id}                # Update alert
  DELETE /{id}                # Silence alert

/api/v1/detectors
  GET    /                    # List detectors
  POST   /                    # Create detector
  PUT    /{id}                # Update detector
  DELETE /{id}                # Delete detector
  POST   /{id}/test           # Test detector

/api/v1/config
  GET    /                    # Get configuration
  PUT    /                    # Update configuration
  GET    /versions            # List config versions
  POST   /rollback/{version}  # Rollback to version
```

---

## Data Schemas

### 1. Telemetry Event Schema

**Purpose:** Standard format for LLM telemetry data received from Observatory.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "TelemetryEvent",
  "type": "object",
  "required": [
    "event_id",
    "timestamp",
    "source",
    "event_type",
    "application_id",
    "data"
  ],
  "properties": {
    "event_id": {
      "type": "string",
      "description": "Unique identifier for the event",
      "format": "uuid",
      "example": "550e8400-e29b-41d4-a716-446655440000"
    },
    "timestamp": {
      "type": "string",
      "description": "ISO 8601 timestamp",
      "format": "date-time",
      "example": "2025-11-06T12:34:56.789Z"
    },
    "source": {
      "type": "object",
      "required": ["service", "host"],
      "properties": {
        "service": {
          "type": "string",
          "example": "llm-observatory"
        },
        "host": {
          "type": "string",
          "example": "obs-worker-01"
        },
        "version": {
          "type": "string",
          "example": "1.2.3"
        }
      }
    },
    "event_type": {
      "type": "string",
      "enum": [
        "llm.request",
        "llm.response",
        "llm.error",
        "llm.metric",
        "llm.trace"
      ],
      "example": "llm.request"
    },
    "application_id": {
      "type": "string",
      "description": "Identifier for the LLM application",
      "example": "chatbot-prod"
    },
    "tenant_id": {
      "type": "string",
      "description": "Multi-tenant identifier",
      "example": "acme-corp"
    },
    "correlation_id": {
      "type": "string",
      "description": "Request correlation ID for tracing",
      "example": "req-abc123"
    },
    "data": {
      "type": "object",
      "description": "Event-specific payload",
      "oneOf": [
        {"$ref": "#/definitions/RequestData"},
        {"$ref": "#/definitions/ResponseData"},
        {"$ref": "#/definitions/MetricData"}
      ]
    },
    "metadata": {
      "type": "object",
      "description": "Additional context",
      "properties": {
        "user_id": {"type": "string"},
        "session_id": {"type": "string"},
        "environment": {"type": "string", "enum": ["dev", "staging", "prod"]},
        "tags": {
          "type": "object",
          "additionalProperties": {"type": "string"}
        }
      }
    }
  },
  "definitions": {
    "RequestData": {
      "type": "object",
      "required": ["model", "prompt"],
      "properties": {
        "model": {
          "type": "string",
          "example": "gpt-4"
        },
        "prompt": {
          "type": "string",
          "description": "User prompt (may be hashed for privacy)"
        },
        "prompt_tokens": {
          "type": "integer",
          "minimum": 0,
          "example": 150
        },
        "parameters": {
          "type": "object",
          "properties": {
            "temperature": {"type": "number"},
            "max_tokens": {"type": "integer"},
            "top_p": {"type": "number"}
          }
        },
        "context": {
          "type": "object",
          "description": "Additional request context"
        }
      }
    },
    "ResponseData": {
      "type": "object",
      "required": ["status", "latency_ms"],
      "properties": {
        "status": {
          "type": "string",
          "enum": ["success", "error", "timeout"],
          "example": "success"
        },
        "response": {
          "type": "string",
          "description": "LLM response (may be hashed)"
        },
        "completion_tokens": {
          "type": "integer",
          "minimum": 0,
          "example": 75
        },
        "total_tokens": {
          "type": "integer",
          "minimum": 0,
          "example": 225
        },
        "latency_ms": {
          "type": "number",
          "minimum": 0,
          "example": 1234.56
        },
        "cost": {
          "type": "number",
          "minimum": 0,
          "description": "Estimated cost in USD",
          "example": 0.045
        },
        "finish_reason": {
          "type": "string",
          "enum": ["stop", "length", "content_filter", "error"]
        }
      }
    },
    "MetricData": {
      "type": "object",
      "required": ["metric_name", "value"],
      "properties": {
        "metric_name": {
          "type": "string",
          "example": "requests_per_second"
        },
        "value": {
          "type": "number",
          "example": 42.5
        },
        "unit": {
          "type": "string",
          "example": "req/s"
        },
        "dimensions": {
          "type": "object",
          "additionalProperties": {"type": "string"}
        }
      }
    }
  }
}
```

**Example Telemetry Event:**

```json
{
  "event_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-11-06T12:34:56.789Z",
  "source": {
    "service": "llm-observatory",
    "host": "obs-worker-01",
    "version": "1.2.3"
  },
  "event_type": "llm.response",
  "application_id": "chatbot-prod",
  "tenant_id": "acme-corp",
  "correlation_id": "req-abc123",
  "data": {
    "status": "success",
    "completion_tokens": 75,
    "total_tokens": 225,
    "latency_ms": 1234.56,
    "cost": 0.045,
    "finish_reason": "stop"
  },
  "metadata": {
    "user_id": "user-456",
    "session_id": "sess-789",
    "environment": "prod",
    "tags": {
      "region": "us-east-1",
      "version": "v2.1.0"
    }
  }
}
```

### 2. Anomaly Event Schema

**Purpose:** Format for detected anomalies emitted by the Detection Engine.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AnomalyEvent",
  "type": "object",
  "required": [
    "anomaly_id",
    "timestamp",
    "detector_id",
    "severity",
    "score",
    "status"
  ],
  "properties": {
    "anomaly_id": {
      "type": "string",
      "format": "uuid",
      "example": "660e8400-e29b-41d4-a716-446655440001"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "When the anomaly was detected",
      "example": "2025-11-06T12:35:00.000Z"
    },
    "detector_id": {
      "type": "string",
      "description": "ID of the detector that found this anomaly",
      "example": "token-rate-spike"
    },
    "detector_type": {
      "type": "string",
      "enum": ["statistical", "rule", "ml", "behavioral"],
      "example": "statistical"
    },
    "severity": {
      "type": "string",
      "enum": ["info", "warning", "critical"],
      "example": "warning"
    },
    "score": {
      "type": "number",
      "minimum": 0,
      "maximum": 1,
      "description": "Anomaly confidence score",
      "example": 0.87
    },
    "status": {
      "type": "string",
      "enum": ["active", "acknowledged", "resolved", "suppressed"],
      "example": "active"
    },
    "application_id": {
      "type": "string",
      "example": "chatbot-prod"
    },
    "tenant_id": {
      "type": "string",
      "example": "acme-corp"
    },
    "title": {
      "type": "string",
      "description": "Human-readable anomaly title",
      "example": "Token rate spike detected"
    },
    "description": {
      "type": "string",
      "description": "Detailed description of the anomaly",
      "example": "Token processing rate increased by 450% above baseline"
    },
    "affected_entity": {
      "type": "object",
      "properties": {
        "type": {
          "type": "string",
          "enum": ["application", "model", "user", "endpoint"],
          "example": "model"
        },
        "id": {
          "type": "string",
          "example": "gpt-4"
        },
        "name": {
          "type": "string",
          "example": "GPT-4 Turbo"
        }
      }
    },
    "metrics": {
      "type": "object",
      "description": "Metrics related to the anomaly",
      "properties": {
        "baseline_value": {"type": "number", "example": 1000},
        "observed_value": {"type": "number", "example": 4500},
        "deviation": {"type": "number", "example": 3.5},
        "threshold": {"type": "number", "example": 3.0}
      }
    },
    "context": {
      "type": "object",
      "description": "Additional context for investigation",
      "properties": {
        "window_start": {"type": "string", "format": "date-time"},
        "window_end": {"type": "string", "format": "date-time"},
        "sample_events": {
          "type": "array",
          "items": {"type": "string"},
          "description": "Sample event IDs for investigation"
        },
        "related_anomalies": {
          "type": "array",
          "items": {"type": "string"},
          "description": "Related anomaly IDs"
        }
      }
    },
    "actions_taken": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "action": {
            "type": "string",
            "enum": ["alert", "shield_block", "shield_rate_limit", "log"],
            "example": "alert"
          },
          "target": {"type": "string", "example": "pagerduty-oncall"},
          "status": {
            "type": "string",
            "enum": ["pending", "success", "failed"],
            "example": "success"
          },
          "timestamp": {"type": "string", "format": "date-time"}
        }
      }
    }
  }
}
```

**Example Anomaly Event:**

```json
{
  "anomaly_id": "660e8400-e29b-41d4-a716-446655440001",
  "timestamp": "2025-11-06T12:35:00.000Z",
  "detector_id": "token-rate-spike",
  "detector_type": "statistical",
  "severity": "warning",
  "score": 0.87,
  "status": "active",
  "application_id": "chatbot-prod",
  "tenant_id": "acme-corp",
  "title": "Token rate spike detected",
  "description": "Token processing rate increased by 450% above baseline",
  "affected_entity": {
    "type": "model",
    "id": "gpt-4",
    "name": "GPT-4 Turbo"
  },
  "metrics": {
    "baseline_value": 1000,
    "observed_value": 4500,
    "deviation": 3.5,
    "threshold": 3.0
  },
  "context": {
    "window_start": "2025-11-06T12:30:00.000Z",
    "window_end": "2025-11-06T12:35:00.000Z",
    "sample_events": [
      "550e8400-e29b-41d4-a716-446655440000",
      "550e8400-e29b-41d4-a716-446655440002"
    ],
    "related_anomalies": []
  },
  "actions_taken": [
    {
      "action": "alert",
      "target": "slack-ml-platform",
      "status": "success",
      "timestamp": "2025-11-06T12:35:01.000Z"
    }
  ]
}
```

### 3. Alert Definition Schema

**Purpose:** Configuration format for alert routing and notification rules.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AlertDefinition",
  "type": "object",
  "required": ["alert_id", "name", "conditions", "actions"],
  "properties": {
    "alert_id": {
      "type": "string",
      "example": "high-latency-alert"
    },
    "name": {
      "type": "string",
      "example": "High Latency Alert"
    },
    "description": {
      "type": "string",
      "example": "Alert when average latency exceeds threshold"
    },
    "enabled": {
      "type": "boolean",
      "default": true
    },
    "conditions": {
      "type": "object",
      "required": ["trigger"],
      "properties": {
        "trigger": {
          "type": "string",
          "enum": ["anomaly", "metric_threshold", "event_pattern"],
          "example": "anomaly"
        },
        "filters": {
          "type": "object",
          "properties": {
            "severity": {
              "type": "array",
              "items": {
                "type": "string",
                "enum": ["info", "warning", "critical"]
              }
            },
            "detector_ids": {
              "type": "array",
              "items": {"type": "string"}
            },
            "application_ids": {
              "type": "array",
              "items": {"type": "string"}
            },
            "score_threshold": {
              "type": "number",
              "minimum": 0,
              "maximum": 1
            }
          }
        },
        "aggregation": {
          "type": "object",
          "properties": {
            "window": {
              "type": "string",
              "pattern": "^[0-9]+(s|m|h|d)$",
              "example": "5m"
            },
            "function": {
              "type": "string",
              "enum": ["count", "avg", "sum", "max", "min"],
              "example": "count"
            },
            "threshold": {
              "type": "number",
              "example": 5
            }
          }
        }
      }
    },
    "actions": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "object",
        "required": ["type"],
        "properties": {
          "type": {
            "type": "string",
            "enum": ["notification", "shield", "webhook", "incident"],
            "example": "notification"
          },
          "config": {
            "type": "object",
            "description": "Action-specific configuration"
          }
        }
      }
    },
    "throttling": {
      "type": "object",
      "properties": {
        "max_frequency": {
          "type": "string",
          "pattern": "^[0-9]+(s|m|h)$",
          "example": "15m",
          "description": "Minimum time between repeated alerts"
        },
        "grouping": {
          "type": "array",
          "items": {"type": "string"},
          "example": ["application_id", "detector_id"],
          "description": "Fields to group alerts by"
        }
      }
    },
    "schedule": {
      "type": "object",
      "properties": {
        "timezone": {
          "type": "string",
          "example": "America/New_York"
        },
        "active_hours": {
          "type": "object",
          "properties": {
            "start": {"type": "string", "pattern": "^([01]?[0-9]|2[0-3]):[0-5][0-9]$"},
            "end": {"type": "string", "pattern": "^([01]?[0-9]|2[0-3]):[0-5][0-9]$"}
          }
        },
        "days_of_week": {
          "type": "array",
          "items": {
            "type": "string",
            "enum": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
          }
        }
      }
    }
  }
}
```

**Example Alert Definition:**

```json
{
  "alert_id": "critical-anomalies",
  "name": "Critical Anomaly Alerts",
  "description": "Immediate notification for critical anomalies",
  "enabled": true,
  "conditions": {
    "trigger": "anomaly",
    "filters": {
      "severity": ["critical"],
      "score_threshold": 0.8
    },
    "aggregation": {
      "window": "1m",
      "function": "count",
      "threshold": 1
    }
  },
  "actions": [
    {
      "type": "notification",
      "config": {
        "receivers": ["pagerduty-oncall", "slack-incidents"],
        "priority": "high",
        "template": "critical-anomaly"
      }
    },
    {
      "type": "incident",
      "config": {
        "auto_create": true,
        "severity": "P1"
      }
    }
  ],
  "throttling": {
    "max_frequency": "5m",
    "grouping": ["application_id", "detector_id"]
  },
  "schedule": {
    "timezone": "UTC",
    "active_hours": {
      "start": "00:00",
      "end": "23:59"
    },
    "days_of_week": ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
  }
}
```

### 4. Configuration Schema

**Purpose:** Master configuration format for Sentinel deployment.

```yaml
# sentinel-config.yaml
version: "v1"
metadata:
  name: sentinel-production
  environment: production
  region: us-east-1

# Ingestion configuration
ingestion:
  protocols:
    grpc:
      enabled: true
      port: 9090
      tls:
        enabled: true
        cert: /etc/sentinel/certs/server.crt
        key: /etc/sentinel/certs/server.key
    http:
      enabled: true
      port: 8080
      cors:
        enabled: true
        allowedOrigins: ["https://dashboard.example.com"]
    kafka:
      enabled: true
      brokers:
        - kafka-1.example.com:9092
        - kafka-2.example.com:9092
      topics:
        - llm-telemetry
        - llm-metrics
      consumerGroup: sentinel-prod
      sasl:
        enabled: true
        mechanism: SCRAM-SHA-512
        username: sentinel
        passwordSecret: kafka-password

  buffer:
    type: ring
    size: 100000
    overflow:
      enabled: true
      path: /var/sentinel/overflow
      maxSize: 10GB

  rateLimiting:
    enabled: true
    maxEventsPerSecond: 50000
    burstSize: 10000
    perTenant: true

# Detection engine configuration
detection:
  workers: 8
  batchSize: 100
  processingTimeout: 5s

  detectors:
    # Load from configuration service
    source: config-service
    syncInterval: 30s

  mlModels:
    path: /var/sentinel/models
    autoReload: true
    reloadInterval: 5m

  correlation:
    enabled: true
    window: 10m
    maxRelatedEvents: 50

# Alert manager configuration
alerting:
  workers: 4
  deduplication:
    enabled: true
    window: 5m
    fields: ["detector_id", "application_id", "severity"]

  grouping:
    enabled: true
    groupBy: ["application_id", "detector_id"]
    groupWait: 30s
    groupInterval: 5m

  routing:
    source: config-service
    defaultReceiver: default-notifications

# Storage configuration
storage:
  timeseries:
    type: influxdb
    url: http://influxdb.example.com:8086
    database: sentinel
    retention:
      raw: 7d
      rollup_1m: 30d
      rollup_5m: 90d

  events:
    type: elasticsearch
    urls:
      - http://es-1.example.com:9200
      - http://es-2.example.com:9200
    index: sentinel-events
    retention: 90d

  alerts:
    type: postgresql
    host: postgres.example.com
    port: 5432
    database: sentinel
    sslMode: require

  objectStorage:
    type: s3
    bucket: sentinel-artifacts
    region: us-east-1
    endpoint: https://s3.amazonaws.com

# Configuration service
configService:
  type: etcd
  endpoints:
    - etcd-1.example.com:2379
    - etcd-2.example.com:2379
    - etcd-3.example.com:2379
  prefix: /sentinel
  tls:
    enabled: true
    certFile: /etc/sentinel/certs/etcd-client.crt
    keyFile: /etc/sentinel/certs/etcd-client.key
    caFile: /etc/sentinel/certs/ca.crt

# API Gateway configuration
api:
  rest:
    enabled: true
    port: 8443
    basePath: /api/v1
    tls:
      enabled: true
      cert: /etc/sentinel/certs/api.crt
      key: /etc/sentinel/certs/api.key

  grpc:
    enabled: true
    port: 9443
    reflection: true

  authentication:
    type: jwt
    issuer: https://auth.example.com
    audience: sentinel-api
    jwksUrl: https://auth.example.com/.well-known/jwks.json

  authorization:
    type: rbac
    policyFile: /etc/sentinel/rbac.yaml

  rateLimiting:
    enabled: true
    defaultLimit: 1000
    window: 1m

# Observability configuration
observability:
  metrics:
    enabled: true
    port: 9090
    path: /metrics

  logging:
    level: info
    format: json
    output: stdout

  tracing:
    enabled: true
    type: jaeger
    endpoint: http://jaeger.example.com:14268/api/traces
    samplingRate: 0.1

# Integration configuration
integrations:
  observatory:
    endpoint: grpc://observatory.example.com:9090
    tls:
      enabled: true
      caFile: /etc/sentinel/certs/ca.crt

  shield:
    endpoint: grpc://shield.example.com:9090
    timeout: 5s
    retries: 3

  incidentManager:
    endpoint: http://incident-manager.example.com:8080
    apiKey: ${INCIDENT_MANAGER_API_KEY}

  governanceDashboard:
    endpoint: http://dashboard.example.com:8080
    apiKey: ${DASHBOARD_API_KEY}

# High availability configuration
ha:
  enabled: true
  leader:
    election: etcd
    leaseDuration: 15s
    renewDeadline: 10s
    retryPeriod: 2s
```

### 5. Metrics Aggregation Schema

**Purpose:** Format for aggregated metrics used in dashboards and analysis.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MetricsAggregation",
  "type": "object",
  "required": ["metric_name", "timestamp", "value", "aggregation"],
  "properties": {
    "metric_name": {
      "type": "string",
      "example": "anomaly_rate"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "Start of the aggregation window",
      "example": "2025-11-06T12:00:00.000Z"
    },
    "window_duration": {
      "type": "string",
      "pattern": "^[0-9]+(s|m|h|d)$",
      "example": "5m"
    },
    "aggregation": {
      "type": "string",
      "enum": ["avg", "sum", "min", "max", "count", "p50", "p95", "p99"],
      "example": "avg"
    },
    "value": {
      "type": "number",
      "example": 42.5
    },
    "unit": {
      "type": "string",
      "example": "anomalies/min"
    },
    "dimensions": {
      "type": "object",
      "description": "Grouping dimensions",
      "properties": {
        "application_id": {"type": "string"},
        "tenant_id": {"type": "string"},
        "detector_type": {"type": "string"},
        "severity": {"type": "string"}
      }
    },
    "metadata": {
      "type": "object",
      "properties": {
        "sample_count": {
          "type": "integer",
          "description": "Number of samples in this aggregation"
        },
        "completeness": {
          "type": "number",
          "minimum": 0,
          "maximum": 1,
          "description": "Data completeness ratio"
        }
      }
    }
  }
}
```

**Example Aggregated Metrics:**

```json
{
  "metric_name": "detector_latency_p95",
  "timestamp": "2025-11-06T12:00:00.000Z",
  "window_duration": "5m",
  "aggregation": "p95",
  "value": 234.5,
  "unit": "ms",
  "dimensions": {
    "application_id": "chatbot-prod",
    "detector_type": "ml",
    "detector_id": "prompt-injection"
  },
  "metadata": {
    "sample_count": 15432,
    "completeness": 0.998
  }
}
```

---

## Processing Pipeline

### Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    PROCESSING PIPELINE                          │
│                                                                 │
│  ┌──────────┐      ┌──────────┐      ┌──────────┐            │
│  │          │      │          │      │          │            │
│  │ Ingest   │─────▶│  Detect  │─────▶│  Alert   │            │
│  │          │      │          │      │          │            │
│  └────┬─────┘      └────┬─────┘      └────┬─────┘            │
│       │                 │                 │                   │
│       │                 │                 │                   │
│       ▼                 ▼                 ▼                   │
│  ┌─────────────────────────────────────────────────┐         │
│  │           Storage & Persistence                  │         │
│  └─────────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────────────┘
```

### Stage 1: Ingestion

**Input:** Raw telemetry from Observatory
**Output:** Validated, normalized events
**Latency Target:** < 10ms p99

```
Observatory Events
      │
      ▼
┌─────────────┐
│  Protocol   │  - Receive gRPC/HTTP/Kafka
│  Handler    │  - Parse payload
└─────┬───────┘
      ▼
┌─────────────┐
│ Validation  │  - Schema validation
│             │  - Required field check
│             │  - Data type validation
└─────┬───────┘
      ▼
┌─────────────┐
│Normalization│  - Timestamp standardization
│             │  - Field mapping
│             │  - Unit conversion
└─────┬───────┘
      ▼
┌─────────────┐
│  Enrichment │  - Tenant lookup
│             │  - Metadata addition
│             │  - Correlation ID injection
└─────┬───────┘
      ▼
┌─────────────┐
│   Buffer    │  - Rate smoothing
│             │  - Batch formation
└─────┬───────┘
      ▼
To Detection Engine
```

### Stage 2: Detection

**Input:** Normalized telemetry events
**Output:** Anomaly events
**Latency Target:** < 100ms p99

```
Normalized Events
      │
      ▼
┌─────────────┐
│   Router    │  - Detector selection
│             │  - Event fan-out
└──────┬──────┘
       │
       ├─────────────┬─────────────┬──────────────┐
       ▼             ▼             ▼              ▼
┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│Statistical│  │   Rule   │  │    ML    │  │Behavioral│
│ Detector │  │ Detector │  │ Detector │  │ Detector │
└────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘
     │             │             │              │
     └─────────────┴─────────────┴──────────────┘
                   │
                   ▼
            ┌──────────────┐
            │  Correlation │  - Event correlation
            │   Engine     │  - Pattern matching
            └──────┬───────┘
                   ▼
            ┌──────────────┐
            │    Scoring   │  - Confidence scoring
            │              │  - Severity assignment
            └──────┬───────┘
                   ▼
            ┌──────────────┐
            │ Deduplication│  - Remove duplicates
            │              │  - Merge related anomalies
            └──────┬───────┘
                   ▼
           Anomaly Events
```

### Stage 3: Alerting

**Input:** Anomaly events
**Output:** Notifications, Shield actions
**Latency Target:** < 500ms p99

```
Anomaly Events
      │
      ▼
┌─────────────┐
│   Routing   │  - Match alert rules
│   Engine    │  - Apply filters
└─────┬───────┘
      ▼
┌─────────────┐
│  Grouping   │  - Group by dimensions
│             │  - Aggregate similar alerts
└─────┬───────┘
      ▼
┌─────────────┐
│  Throttling │  - Rate limiting
│             │  - Suppression rules
└─────┬───────┘
      ▼
┌─────────────┐
│  Escalation │  - Apply escalation policy
│             │  - Priority assignment
└─────┬───────┘
      ▼
┌─────────────┐
│ Notification│  - Format message
│ Dispatcher  │  - Send to receivers
└─────┬───────┘
      │
      ├────────┬──────────┬──────────┐
      ▼        ▼          ▼          ▼
  ┌──────┐ ┌──────┐  ┌──────┐  ┌──────┐
  │Email │ │Slack │  │PagerD│  │Shield│
  └──────┘ └──────┘  └──────┘  └──────┘
```

### Pipeline Performance Characteristics

| Stage | Throughput Target | Latency p50 | Latency p99 | CPU Bound | I/O Bound |
|-------|------------------|-------------|-------------|-----------|-----------|
| Ingestion | 100K events/s | 2ms | 10ms | Low | High |
| Detection | 50K events/s | 20ms | 100ms | High | Low |
| Alerting | 10K alerts/s | 50ms | 500ms | Low | High |

---

## Deployment Topologies

### 1. Standalone Binary Deployment

**Overview:** Single-process deployment suitable for small-scale deployments and development.

```
┌──────────────────────────────────────┐
│         Sentinel Binary              │
│                                      │
│  ┌────────────────────────────────┐ │
│  │   All Components in One        │ │
│  │   Process                      │ │
│  │                                │ │
│  │  - Ingestion                   │ │
│  │  - Detection                   │ │
│  │  - Alerting                    │ │
│  │  - API Gateway                 │ │
│  │  - Config (file-based)         │ │
│  └────────────────────────────────┘ │
│                                      │
│  ┌────────────────────────────────┐ │
│  │   Embedded Storage             │ │
│  │  - SQLite (events/alerts)      │ │
│  │  - In-memory TSDB              │ │
│  └────────────────────────────────┘ │
└──────────────────────────────────────┘
```

**Deployment:**

```bash
# Installation
curl -fsSL https://get.llm-sentinel.io | sh

# Configuration
cat > /etc/sentinel/config.yaml <<EOF
mode: standalone
ingestion:
  grpc:
    port: 9090
  http:
    port: 8080
storage:
  path: /var/sentinel/data
detectors:
  configFile: /etc/sentinel/detectors.yaml
alerts:
  configFile: /etc/sentinel/alerts.yaml
EOF

# Run
sentinel --config /etc/sentinel/config.yaml
```

**Docker Deployment:**

```dockerfile
FROM alpine:3.18
COPY sentinel /usr/local/bin/
COPY config.yaml /etc/sentinel/
EXPOSE 8080 9090
VOLUME ["/var/sentinel/data"]
CMD ["sentinel", "--config", "/etc/sentinel/config.yaml"]
```

```yaml
# docker-compose.yml
version: '3.8'
services:
  sentinel:
    image: llm-sentinel:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./config:/etc/sentinel
      - sentinel-data:/var/sentinel/data
    environment:
      - LOG_LEVEL=info
    restart: unless-stopped

volumes:
  sentinel-data:
```

**Use Cases:**
- Development and testing
- Small deployments (< 1000 events/s)
- Edge deployments
- Proof of concept

**Pros:**
- Simple deployment and operation
- No external dependencies
- Low operational overhead
- Fast startup time
- Easy troubleshooting

**Cons:**
- Limited scalability (vertical only)
- Single point of failure
- No horizontal scaling
- Resource contention between components
- Limited high availability options

**Resource Requirements:**
- CPU: 2-4 cores
- Memory: 4-8 GB
- Disk: 50+ GB SSD
- Network: 1 Gbps

**Scaling Limits:**
- Max throughput: ~5,000 events/s
- Max concurrent alerts: ~1,000
- Max detectors: ~100

### 2. Microservice Architecture Deployment

**Overview:** Distributed deployment with separate services for horizontal scalability and resilience.

```
┌────────────────────────────────────────────────────────────────┐
│                    Load Balancer Layer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                    │
│  │  gRPC LB │  │ HTTP LB  │  │ Kafka    │                    │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘                    │
└───────┼─────────────┼─────────────┼────────────────────────────┘
        │             │             │
┌───────┼─────────────┼─────────────┼────────────────────────────┐
│       │             │             │                            │
│       ▼             ▼             ▼                            │
│  ┌────────────────────────────────────┐                       │
│  │   Ingestion Service (Cluster)      │                       │
│  │  ┌──────┐ ┌──────┐ ┌──────┐       │                       │
│  │  │ Pod1 │ │ Pod2 │ │ PodN │       │                       │
│  │  └──────┘ └──────┘ └──────┘       │                       │
│  └────────────────┬───────────────────┘                       │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────┐                       │
│  │      Message Queue (Kafka)         │                       │
│  └────────────────┬───────────────────┘                       │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────┐                       │
│  │   Detection Service (Cluster)      │                       │
│  │  ┌──────┐ ┌──────┐ ┌──────┐       │                       │
│  │  │ Pod1 │ │ Pod2 │ │ PodN │       │                       │
│  │  └──────┘ └──────┘ └──────┘       │                       │
│  └────────────────┬───────────────────┘                       │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────┐                       │
│  │      Message Queue (Kafka)         │                       │
│  └────────────────┬───────────────────┘                       │
│                   │                                           │
│                   ▼                                           │
│  ┌────────────────────────────────────┐                       │
│  │    Alert Service (Cluster)         │                       │
│  │  ┌──────┐ ┌──────┐ ┌──────┐       │                       │
│  │  │ Pod1 │ │ Pod2 │ │ PodN │       │                       │
│  │  └──────┘ └──────┘ └──────┘       │                       │
│  └────────────────────────────────────┘                       │
│                                                               │
│  ┌────────────────────────────────────┐                       │
│  │     API Gateway Service            │                       │
│  │  ┌──────┐ ┌──────┐                │                       │
│  │  │ Pod1 │ │ Pod2 │                │                       │
│  │  └──────┘ └──────┘                │                       │
│  └────────────────────────────────────┘                       │
│                                                               │
│  ┌────────────────────────────────────┐                       │
│  │   Configuration Service (etcd)     │                       │
│  │  ┌──────┐ ┌──────┐ ┌──────┐       │                       │
│  │  │Node1 │ │Node2 │ │Node3 │       │                       │
│  │  └──────┘ └──────┘ └──────┘       │                       │
│  └────────────────────────────────────┘                       │
└────────────────────────────────────────────────────────────────┘
        │                    │                    │
        ▼                    ▼                    ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│  TimescaleDB │  │Elasticsearch │  │ PostgreSQL   │
│   (Metrics)  │  │   (Events)   │  │   (Alerts)   │
└──────────────┘  └──────────────┘  └──────────────┘
```

**Kubernetes Deployment:**

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: sentinel

---
# ingestion-service.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sentinel-ingestion
  namespace: sentinel
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sentinel-ingestion
  template:
    metadata:
      labels:
        app: sentinel-ingestion
    spec:
      containers:
      - name: ingestion
        image: sentinel-ingestion:v1.0.0
        ports:
        - containerPort: 9090
          name: grpc
        - containerPort: 8080
          name: http
        - containerPort: 9102
          name: metrics
        env:
        - name: COMPONENT
          value: "ingestion"
        - name: KAFKA_BROKERS
          value: "kafka:9092"
        - name: CONFIG_ETCD_ENDPOINTS
          value: "etcd:2379"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          grpc:
            port: 9090
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          grpc:
            port: 9090
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: sentinel-ingestion
  namespace: sentinel
spec:
  selector:
    app: sentinel-ingestion
  ports:
  - name: grpc
    port: 9090
    targetPort: 9090
  - name: http
    port: 8080
    targetPort: 8080
  type: LoadBalancer

---
# detection-service.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sentinel-detection
  namespace: sentinel
spec:
  replicas: 5
  selector:
    matchLabels:
      app: sentinel-detection
  template:
    metadata:
      labels:
        app: sentinel-detection
    spec:
      containers:
      - name: detection
        image: sentinel-detection:v1.0.0
        ports:
        - containerPort: 9102
          name: metrics
        env:
        - name: COMPONENT
          value: "detection"
        - name: KAFKA_BROKERS
          value: "kafka:9092"
        - name: CONFIG_ETCD_ENDPOINTS
          value: "etcd:2379"
        - name: ML_MODEL_PATH
          value: "/models"
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        volumeMounts:
        - name: models
          mountPath: /models
          readOnly: true
      volumes:
      - name: models
        persistentVolumeClaim:
          claimName: ml-models-pvc

---
# alert-service.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sentinel-alert
  namespace: sentinel
spec:
  replicas: 3
  selector:
    matchLabels:
      app: sentinel-alert
  template:
    metadata:
      labels:
        app: sentinel-alert
    spec:
      containers:
      - name: alert
        image: sentinel-alert:v1.0.0
        ports:
        - containerPort: 9102
          name: metrics
        env:
        - name: COMPONENT
          value: "alert"
        - name: KAFKA_BROKERS
          value: "kafka:9092"
        - name: CONFIG_ETCD_ENDPOINTS
          value: "etcd:2379"
        - name: POSTGRES_HOST
          value: "postgresql"
        - name: POSTGRES_DB
          value: "sentinel"
        envFrom:
        - secretRef:
            name: sentinel-secrets
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"

---
# api-gateway.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sentinel-api
  namespace: sentinel
spec:
  replicas: 2
  selector:
    matchLabels:
      app: sentinel-api
  template:
    metadata:
      labels:
        app: sentinel-api
    spec:
      containers:
      - name: api
        image: sentinel-api:v1.0.0
        ports:
        - containerPort: 8443
          name: https
        - containerPort: 9443
          name: grpc
        - containerPort: 9102
          name: metrics
        env:
        - name: COMPONENT
          value: "api"
        - name: CONFIG_ETCD_ENDPOINTS
          value: "etcd:2379"
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"

---
apiVersion: v1
kind: Service
metadata:
  name: sentinel-api
  namespace: sentinel
spec:
  selector:
    app: sentinel-api
  ports:
  - name: https
    port: 8443
    targetPort: 8443
  - name: grpc
    port: 9443
    targetPort: 9443
  type: LoadBalancer

---
# HorizontalPodAutoscaler for detection service
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sentinel-detection-hpa
  namespace: sentinel
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sentinel-detection
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: kafka_consumer_lag
      target:
        type: AverageValue
        averageValue: "1000"
```

**Helm Chart Structure:**

```
sentinel-chart/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── ingestion/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   └── hpa.yaml
│   ├── detection/
│   │   ├── deployment.yaml
│   │   └── hpa.yaml
│   ├── alert/
│   │   ├── deployment.yaml
│   │   └── service.yaml
│   ├── api/
│   │   ├── deployment.yaml
│   │   ├── service.yaml
│   │   └── ingress.yaml
│   ├── configmap.yaml
│   ├── secrets.yaml
│   └── servicemonitor.yaml
└── values/
    ├── production.yaml
    ├── staging.yaml
    └── development.yaml
```

**Use Cases:**
- Production deployments
- High-scale environments (> 10,000 events/s)
- Multi-tenant platforms
- Mission-critical systems

**Pros:**
- Horizontal scalability
- Independent component scaling
- High availability
- Fault isolation
- Rolling updates
- Resource optimization per component

**Cons:**
- Complex deployment and operations
- Higher infrastructure costs
- Network overhead between services
- Requires orchestration platform
- More moving parts to monitor

**Resource Requirements (Production):**
- Ingestion: 3 pods × (2 CPU, 4 GB RAM)
- Detection: 5 pods × (4 CPU, 8 GB RAM)
- Alert: 3 pods × (1 CPU, 2 GB RAM)
- API: 2 pods × (1 CPU, 2 GB RAM)
- Total: 34 CPU cores, 84 GB RAM

**Scaling Characteristics:**
- Max throughput: 100K+ events/s
- Horizontal scaling: Auto-scaling based on load
- Replication: 3+ replicas per service
- Partition tolerance: Kafka-based event distribution

### 3. Sidecar Pattern Deployment

**Overview:** Deploy Sentinel as a sidecar container alongside LLM services for low-latency monitoring.

```
┌─────────────────────────────────────────────────────────┐
│                Application Pod                          │
│                                                         │
│  ┌──────────────────┐       ┌──────────────────┐      │
│  │   LLM Service    │       │  Sentinel Sidecar│      │
│  │   Container      │       │   Container      │      │
│  │                  │       │                  │      │
│  │  - Model Server  │◄─────▶│  - Ingestion     │      │
│  │  - API Endpoint  │ local │  - Detection     │      │
│  │  - Business      │ IPC   │  - Local Cache   │      │
│  │    Logic         │       │                  │      │
│  └────────┬─────────┘       └────────┬─────────┘      │
│           │                          │                 │
└───────────┼──────────────────────────┼─────────────────┘
            │                          │
            ▼                          ▼
     External Traffic          Central Sentinel
                               (Aggregation Layer)
```

**Kubernetes Pod Configuration:**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-chatbot
  namespace: applications
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-chatbot
  template:
    metadata:
      labels:
        app: llm-chatbot
      annotations:
        sentinel.io/inject: "true"
        sentinel.io/mode: "sidecar"
    spec:
      containers:
      # Main application container
      - name: chatbot
        image: llm-chatbot:v1.0.0
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: SENTINEL_ENDPOINT
          value: "localhost:9090"  # Sidecar local endpoint
        - name: TELEMETRY_MODE
          value: "push"
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"

      # Sentinel sidecar container
      - name: sentinel-sidecar
        image: sentinel-sidecar:v1.0.0
        ports:
        - containerPort: 9090
          name: grpc
        - containerPort: 8081
          name: health
        env:
        - name: MODE
          value: "sidecar"
        - name: APPLICATION_ID
          value: "llm-chatbot"
        - name: CENTRAL_AGGREGATOR
          value: "sentinel-central.sentinel.svc.cluster.local:9090"
        - name: LOCAL_CACHE_SIZE
          value: "1000"
        - name: BATCH_SIZE
          value: "100"
        - name: FLUSH_INTERVAL
          value: "1s"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8081
          initialDelaySeconds: 10
          periodSeconds: 10

      # Shared volume for local communication
      volumes:
      - name: shared-memory
        emptyDir:
          medium: Memory
          sizeLimit: 100Mi
```

**Service Mesh Integration (Istio):**

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: llm-chatbot
  namespace: applications
spec:
  hosts:
  - llm-chatbot.applications.svc.cluster.local
  http:
  - route:
    - destination:
        host: llm-chatbot.applications.svc.cluster.local
        port:
          number: 8080
    # Sentinel sidecar automatically intercepts telemetry

---
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: sentinel-telemetry
  namespace: applications
spec:
  host: sentinel-central.sentinel.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
      http:
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10
```

**Sidecar Configuration:**

```yaml
# sentinel-sidecar.yaml
mode: sidecar
application:
  id: ${APPLICATION_ID}
  namespace: ${POD_NAMESPACE}

ingestion:
  local:
    grpc:
      port: 9090
      unix_socket: /var/run/sentinel/sentinel.sock

  buffer:
    type: ring
    size: 1000
    overflow: drop  # Drop on overflow in sidecar mode

  batching:
    enabled: true
    size: 100
    timeout: 1s

detection:
  local:
    # Run lightweight detectors locally
    enabled: true
    detectors:
      - type: threshold
        low_latency: true
      - type: pattern
        fast_match: true

  remote:
    # Delegate complex detection to central
    enabled: true
    endpoint: ${CENTRAL_AGGREGATOR}
    async: true

forwarding:
  central:
    endpoint: ${CENTRAL_AGGREGATOR}
    protocol: grpc
    tls:
      enabled: true
    retry:
      enabled: true
      maxAttempts: 3
      backoff: exponential

  compression:
    enabled: true
    algorithm: gzip

cache:
  type: memory
  size: ${LOCAL_CACHE_SIZE}
  ttl: 5m

health:
  port: 8081
  endpoint: /health
```

**Use Cases:**
- Low-latency telemetry collection
- Service mesh environments
- Per-application isolation
- Edge deployments
- Real-time detection requirements

**Pros:**
- Minimal network latency (local IPC)
- Co-located with application
- Automatic scaling with application
- Isolated failure domain
- Service mesh integration
- Real-time telemetry capture

**Cons:**
- Resource overhead per pod
- Complex coordination
- Limited detector capabilities (resource constrained)
- Increased pod size
- Harder to update separately

**Resource Requirements (Per Pod):**
- Sidecar CPU: 250-500m
- Sidecar Memory: 512 MB - 1 GB
- Storage: Shared with main container
- Network: Negligible (local IPC)

**Performance Characteristics:**
- Telemetry latency: < 1ms (local)
- Forwarding latency: 5-10ms
- Local detection: < 5ms
- Overhead: ~10-15% CPU, 5-10% memory

---

## Integration Patterns

### 1. Observatory → Sentinel Integration

**Pattern:** Push-based telemetry streaming

```
┌──────────────────┐
│  LLM-Observatory │
│                  │
│  ┌────────────┐  │      gRPC Stream
│  │ Telemetry  │──┼──────────────────────┐
│  │  Exporter  │  │                      │
│  └────────────┘  │                      │
└──────────────────┘                      │
                                          ▼
                               ┌──────────────────┐
                               │  LLM-Sentinel    │
                               │                  │
                               │ ┌──────────────┐ │
                               │ │  Ingestion   │ │
                               │ │   Service    │ │
                               │ └──────────────┘ │
                               └──────────────────┘
```

**gRPC Service Definition:**

```protobuf
// telemetry.proto
syntax = "proto3";

package llm.sentinel.v1;

service TelemetryIngestion {
  // Streaming telemetry ingestion
  rpc StreamTelemetry(stream TelemetryEvent) returns (StreamResponse);

  // Batch telemetry ingestion
  rpc BatchTelemetry(BatchTelemetryRequest) returns (BatchTelemetryResponse);

  // Health check
  rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}

message TelemetryEvent {
  string event_id = 1;
  google.protobuf.Timestamp timestamp = 2;
  EventSource source = 3;
  EventType type = 4;
  string application_id = 5;
  string tenant_id = 6;
  oneof payload {
    RequestData request = 10;
    ResponseData response = 11;
    MetricData metric = 12;
  }
  map<string, string> metadata = 20;
}

message StreamResponse {
  bool accepted = 1;
  string message = 2;
}
```

**Observatory Configuration:**

```yaml
# observatory-config.yaml
exporters:
  sentinel:
    enabled: true
    endpoint: sentinel.example.com:9090
    protocol: grpc
    tls:
      enabled: true
      ca_cert: /etc/certs/ca.crt

    batching:
      enabled: true
      size: 100
      timeout: 1s

    retry:
      enabled: true
      max_attempts: 3
      backoff: exponential

    compression: gzip

    filters:
      # Only send relevant events
      include:
        event_types: ["llm.request", "llm.response", "llm.error"]
      exclude:
        metadata:
          internal: "true"
```

### 2. Sentinel → Shield Integration

**Pattern:** Action triggering for threat response

```
┌──────────────────┐
│  LLM-Sentinel    │
│                  │
│ ┌──────────────┐ │   Action Request (gRPC)
│ │   Alert      │─┼────────────────────────┐
│ │  Manager     │ │                        │
│ └──────────────┘ │                        │
└──────────────────┘                        │
                                            ▼
                                 ┌──────────────────┐
                                 │   LLM-Shield     │
                                 │                  │
                                 │ ┌──────────────┐ │
                                 │ │   Action     │ │
                                 │ │  Enforcer    │ │
                                 │ └──────────────┘ │
                                 └──────────────────┘
```

**gRPC Service Definition:**

```protobuf
// shield-action.proto
syntax = "proto3";

package llm.shield.v1;

service ShieldAction {
  rpc EnforceAction(ActionRequest) returns (ActionResponse);
  rpc BulkEnforceActions(BulkActionRequest) returns (BulkActionResponse);
  rpc RevokeAction(RevokeRequest) returns (RevokeResponse);
}

message ActionRequest {
  string action_id = 1;
  ActionType type = 2;
  string target_entity = 3;
  string reason = 4;
  google.protobuf.Duration duration = 5;
  map<string, string> parameters = 10;
}

enum ActionType {
  BLOCK = 0;
  RATE_LIMIT = 1;
  THROTTLE = 2;
  QUARANTINE = 3;
  ALERT_ONLY = 4;
}

message ActionResponse {
  bool success = 1;
  string action_id = 2;
  string message = 3;
  google.protobuf.Timestamp enforced_at = 4;
}
```

**Sentinel Alert Action Configuration:**

```yaml
detectors:
  - id: prompt-injection-detector
    type: ml
    actions:
      - type: shield
        config:
          endpoint: shield.example.com:9090
          action: block
          target: request
          duration: 1h
          parameters:
            reason: "Prompt injection detected"
            severity: critical
```

### 3. Sentinel → Incident Manager Integration

**Pattern:** Automated incident creation and updates

```
┌──────────────────┐
│  LLM-Sentinel    │
│                  │
│ ┌──────────────┐ │   Webhook / REST API
│ │   Alert      │─┼────────────────────────┐
│ │  Manager     │ │                        │
│ └──────────────┘ │                        │
└──────────────────┘                        │
                                            ▼
                                 ┌──────────────────┐
                                 │LLM-Incident-Mgr  │
                                 │                  │
                                 │ ┌──────────────┐ │
                                 │ │  Incident    │ │
                                 │ │   Handler    │ │
                                 │ └──────────────┘ │
                                 └──────────────────┘
```

**REST API Integration:**

```yaml
# Sentinel configuration
integrations:
  incident_manager:
    enabled: true
    endpoint: https://incident-manager.example.com
    api_key: ${INCIDENT_MANAGER_API_KEY}

    auto_create:
      enabled: true
      conditions:
        - severity: critical
        - score: ">= 0.9"

    mapping:
      severity:
        critical: P1
        warning: P2
        info: P3

      fields:
        title: "{{ .anomaly.title }}"
        description: "{{ .anomaly.description }}"
        source: "llm-sentinel"
        tags:
          - "anomaly:{{ .anomaly.detector_id }}"
          - "application:{{ .anomaly.application_id }}"
```

**Webhook Payload:**

```json
POST /api/v1/incidents
Content-Type: application/json
Authorization: Bearer ${API_KEY}

{
  "source": "llm-sentinel",
  "incident_type": "anomaly_detected",
  "severity": "P1",
  "title": "Token rate spike detected in chatbot-prod",
  "description": "Token processing rate increased by 450% above baseline",
  "metadata": {
    "anomaly_id": "660e8400-e29b-41d4-a716-446655440001",
    "detector_id": "token-rate-spike",
    "application_id": "chatbot-prod",
    "score": 0.87,
    "timestamp": "2025-11-06T12:35:00.000Z"
  },
  "affected_entities": [
    {
      "type": "model",
      "id": "gpt-4",
      "name": "GPT-4 Turbo"
    }
  ],
  "remediation_actions": [
    "Investigate recent changes to chatbot-prod",
    "Review token usage patterns",
    "Check for potential abuse"
  ]
}
```

### 4. Sentinel → Governance Dashboard Integration

**Pattern:** Metrics and analytics export

```
┌──────────────────┐
│  LLM-Sentinel    │
│                  │
│ ┌──────────────┐ │   Query API (GraphQL/REST)
│ │   Storage    │◄┼────────────────────────┐
│ │    Layer     │ │                        │
│ └──────────────┘ │                        │
│                  │                        │
│ ┌──────────────┐ │   Metrics Push         │
│ │   Metrics    │─┼────────────────────────┤
│ │  Aggregator  │ │                        │
│ └──────────────┘ │                        │
└──────────────────┘                        │
                                            │
                                            ▼
                                 ┌──────────────────┐
                                 │   Governance     │
                                 │   Dashboard      │
                                 │                  │
                                 │ ┌──────────────┐ │
                                 │ │ Analytics &  │ │
                                 │ │   Reporting  │ │
                                 │ └──────────────┘ │
                                 └──────────────────┘
```

**GraphQL API Schema:**

```graphql
type Query {
  # Anomaly queries
  anomalies(
    filter: AnomalyFilter
    timeRange: TimeRange!
    limit: Int = 100
  ): [Anomaly!]!

  anomalyById(id: ID!): Anomaly

  # Metrics queries
  metrics(
    names: [String!]!
    dimensions: [Dimension!]
    timeRange: TimeRange!
    aggregation: Aggregation!
  ): [MetricSeries!]!

  # Alert queries
  alerts(
    filter: AlertFilter
    timeRange: TimeRange!
  ): [Alert!]!

  # Statistics
  statistics(
    timeRange: TimeRange!
    groupBy: [String!]
  ): Statistics!
}

type Anomaly {
  id: ID!
  timestamp: DateTime!
  detectorId: String!
  severity: Severity!
  score: Float!
  status: AnomalyStatus!
  title: String!
  description: String!
  affectedEntity: Entity!
  metrics: AnomalyMetrics!
  relatedAnomalies: [Anomaly!]!
}

type MetricSeries {
  name: String!
  dimensions: [Dimension!]!
  datapoints: [Datapoint!]!
}

type Datapoint {
  timestamp: DateTime!
  value: Float!
}
```

**Prometheus Metrics Export:**

```yaml
# Metrics exposed by Sentinel
sentinel_telemetry_events_total{application="chatbot-prod",type="request"}
sentinel_telemetry_events_total{application="chatbot-prod",type="response"}

sentinel_anomalies_detected_total{detector="token-rate-spike",severity="warning"}
sentinel_anomalies_detected_total{detector="prompt-injection",severity="critical"}

sentinel_alerts_sent_total{receiver="pagerduty-oncall",status="success"}
sentinel_alerts_sent_total{receiver="slack-incidents",status="failed"}

sentinel_detector_latency_seconds{detector="token-rate-spike",quantile="0.5"}
sentinel_detector_latency_seconds{detector="token-rate-spike",quantile="0.95"}
sentinel_detector_latency_seconds{detector="token-rate-spike",quantile="0.99"}

sentinel_ingestion_throughput{protocol="grpc"}
sentinel_ingestion_throughput{protocol="http"}
sentinel_ingestion_throughput{protocol="kafka"}
```

---

## Scalability and Fault Tolerance

### Horizontal Scaling Strategy

**Component-Level Scaling:**

```yaml
# Auto-scaling policies
components:
  ingestion:
    minReplicas: 3
    maxReplicas: 20
    metrics:
      - type: cpu
        target: 70%
      - type: custom
        metric: events_per_second
        target: 5000

  detection:
    minReplicas: 5
    maxReplicas: 50
    metrics:
      - type: cpu
        target: 80%
      - type: custom
        metric: kafka_consumer_lag
        target: 1000
      - type: custom
        metric: processing_latency_p99
        target: 100ms

  alert:
    minReplicas: 2
    maxReplicas: 10
    metrics:
      - type: cpu
        target: 60%
      - type: custom
        metric: alert_queue_depth
        target: 100
```

### Fault Tolerance Design

**1. Data Durability:**

```
Telemetry Event Flow with Durability:

Observatory
    │
    ▼
┌────────────┐
│ Ingestion  │──────┐
│  Service   │      │ Persist to Kafka
└────────────┘      │ (replication factor: 3)
                    │
                    ▼
            ┌───────────────┐
            │  Kafka Topic  │
            │ (3 replicas)  │
            └───────────────┘
                    │
        ┌───────────┼───────────┐
        ▼           ▼           ▼
   ┌─────────┐ ┌─────────┐ ┌─────────┐
   │Detection│ │Detection│ │Detection│
   │ Worker1 │ │ Worker2 │ │ WorkerN │
   └─────────┘ └─────────┘ └─────────┘

# Kafka configuration for durability
kafka:
  topics:
    llm-telemetry:
      partitions: 12
      replicationFactor: 3
      minInSyncReplicas: 2
      retentionMs: 604800000  # 7 days
```

**2. Circuit Breaker Pattern:**

```go
// Example: Circuit breaker for Shield integration
type ShieldClient struct {
    breaker *circuitbreaker.CircuitBreaker
    client  shield.ShieldClient
}

func (s *ShieldClient) EnforceAction(ctx context.Context, req *ActionRequest) error {
    return s.breaker.Execute(func() error {
        _, err := s.client.EnforceAction(ctx, req)
        return err
    })
}

// Circuit breaker configuration
breaker := circuitbreaker.New(
    circuitbreaker.WithFailureThreshold(5),
    circuitbreaker.WithSuccessThreshold(2),
    circuitbreaker.WithTimeout(30 * time.Second),
)
```

**3. Retry and Backoff:**

```yaml
# Retry configuration for critical paths
retry:
  alert_notification:
    enabled: true
    maxAttempts: 5
    initialBackoff: 1s
    maxBackoff: 60s
    backoffMultiplier: 2

  shield_action:
    enabled: true
    maxAttempts: 3
    initialBackoff: 500ms
    maxBackoff: 5s
    backoffMultiplier: 2
```

**4. Leader Election for Singleton Tasks:**

```yaml
# Leader election for maintenance tasks
ha:
  leaderElection:
    enabled: true
    backend: etcd
    leaseDuration: 15s
    renewDeadline: 10s
    retryPeriod: 2s

  singletonTasks:
    - name: config-sync
      schedule: "*/5 * * * *"
    - name: cleanup
      schedule: "0 2 * * *"
```

### Multi-Region Deployment

```
Region 1 (us-east-1)          Region 2 (us-west-2)          Region 3 (eu-west-1)
┌──────────────────┐          ┌──────────────────┐          ┌──────────────────┐
│  Sentinel        │          │  Sentinel        │          │  Sentinel        │
│  Cluster         │          │  Cluster         │          │  Cluster         │
│                  │          │                  │          │                  │
│  - Ingestion     │          │  - Ingestion     │          │  - Ingestion     │
│  - Detection     │          │  - Detection     │          │  - Detection     │
│  - Alerting      │          │  - Alerting      │          │  - Alerting      │
└────────┬─────────┘          └────────┬─────────┘          └────────┬─────────┘
         │                              │                              │
         └──────────────────────────────┼──────────────────────────────┘
                                        │
                                        ▼
                          ┌──────────────────────────┐
                          │  Global Configuration    │
                          │  (Multi-region etcd)     │
                          └──────────────────────────┘
                                        │
                          ┌─────────────┴─────────────┐
                          │  Global Analytics Store   │
                          │  (Cross-region S3/GCS)    │
                          └───────────────────────────┘
```

**Multi-Region Configuration:**

```yaml
multiRegion:
  enabled: true
  localRegion: us-east-1

  regions:
    - name: us-east-1
      priority: 1
      endpoint: sentinel-us-east-1.example.com
    - name: us-west-2
      priority: 2
      endpoint: sentinel-us-west-2.example.com
    - name: eu-west-1
      priority: 3
      endpoint: sentinel-eu-west-1.example.com

  dataLocality:
    enabled: true
    # Keep data in local region
    localProcessing: true

  failover:
    enabled: true
    healthCheckInterval: 30s
    failoverTimeout: 60s
```

---

## Summary

This architecture document provides a comprehensive blueprint for deploying LLM-Sentinel in production environments. The key design decisions prioritize:

1. **Flexibility**: Multiple deployment topologies to suit different scales and requirements
2. **Scalability**: Horizontal scaling with stateless services and event streaming
3. **Reliability**: Fault tolerance through replication, retries, and graceful degradation
4. **Performance**: Optimized pipelines with configurable latency/throughput trade-offs
5. **Integration**: Well-defined interfaces for the broader LLM governance ecosystem

The architecture supports deployment scenarios from simple standalone binaries to complex multi-region microservice deployments, making it suitable for organizations at any stage of LLM adoption.
