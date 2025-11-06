# LLM-Sentinel Documentation

Welcome to the LLM-Sentinel documentation. This directory contains comprehensive guides for understanding, deploying, and operating the LLM-Sentinel anomaly detection system.

## Documentation Structure

### Architecture Documentation

- **[ARCHITECTURE.md](../ARCHITECTURE.md)** - Complete system architecture
  - High-level component diagram
  - Core components deep-dive
  - Data schemas (JSON Schema with examples)
  - Processing pipeline architecture
  - Deployment topologies (Standalone, Microservice, Sidecar)
  - Integration patterns with ecosystem components
  - Scalability and fault tolerance design

### Deployment Guides

- **[deployment-guide.md](./deployment-guide.md)** - Production deployment guide
  - Quick start (5-minute demo)
  - Standalone binary deployment
  - Docker Compose deployment
  - Kubernetes deployment (with Helm)
  - Sidecar deployment patterns
  - Production checklist
  - Monitoring and observability setup
  - Troubleshooting guide

### Integration Documentation

- **[integration-examples.md](./integration-examples.md)** - Integration code examples
  - Observatory → Sentinel integration (gRPC, HTTP, Kafka)
  - Sentinel → Shield integration (action enforcement)
  - Sentinel → Incident Manager integration (webhooks, REST)
  - Sentinel → Governance Dashboard integration (GraphQL, REST)
  - Custom integrations (webhooks, plugins)
  - End-to-end scenarios

### Performance Documentation

- **[performance-benchmarks.md](./performance-benchmarks.md)** - Performance benchmarks and tuning
  - Performance targets (SLOs)
  - Benchmark methodology
  - Deployment-specific benchmarks (Standalone, Microservice, Sidecar)
  - Performance tuning guide
  - Capacity planning calculator
  - Stress testing scenarios
  - Monitoring and alerting

## Quick Links

### Getting Started

1. **First Time Users**: Start with the [Quick Start](./deployment-guide.md#quick-start) section
2. **Architecture Overview**: Read the [System Overview](../ARCHITECTURE.md#system-overview)
3. **Deploy to Production**: Follow the [Production Checklist](./deployment-guide.md#production-checklist)

### Common Tasks

- **Deploy Standalone**: [Standalone Deployment](./deployment-guide.md#standalone-deployment)
- **Deploy to Kubernetes**: [Kubernetes Deployment](./deployment-guide.md#kubernetes-deployment)
- **Integrate with Observatory**: [Observatory Integration](./integration-examples.md#observatory-integration)
- **Configure Detectors**: See data schemas in [ARCHITECTURE.md](../ARCHITECTURE.md#data-schemas)
- **Tune Performance**: [Performance Tuning](./performance-benchmarks.md#performance-tuning)

## Document Conventions

### Code Examples

All code examples are production-ready and follow these conventions:

- **Configuration files**: YAML format with comments
- **API examples**: Include authentication and error handling
- **Deployment manifests**: Kubernetes-native YAML
- **Integration code**: Multiple languages (Go, Python, JavaScript, Java)

### Placeholders

Replace these placeholders in examples with your values:

- `${VARIABLE}` - Environment variable
- `example.com` - Your domain
- `your-api-key` - Your actual API key
- `chatbot-prod` - Your application ID

### Version Compatibility

- Documentation version: v1.0.0
- Compatible with Sentinel: v1.x.x
- Kubernetes: 1.27+
- Docker: 24.0+

## Architecture at a Glance

```
┌─────────────────────────────────────────────────────────────────────┐
│                        LLM Ecosystem                                 │
│                                                                      │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐     │
│  │ LLM Services │─────▶│    LLM-      │─────▶│    LLM-      │     │
│  │              │      │ Observatory  │      │  Sentinel    │     │
│  └──────────────┘      └──────────────┘      └──────┬───────┘     │
│                                                      │              │
│                              ┌───────────────────────┼─────────┐   │
│                              │                       │         │   │
│                              ▼                       ▼         ▼   │
│                        ┌──────────┐          ┌──────────┐  ┌─────┐│
│                        │   LLM-   │          │   LLM-   │  │ Gov │││
│                        │  Shield  │          │ Incident │  │Dash │││
│                        │          │          │ Manager  │  │     │││
│                        └──────────┘          └──────────┘  └─────┘│
└─────────────────────────────────────────────────────────────────────┘
```

## Deployment Topologies Summary

| Topology | Use Case | Max Throughput | Complexity | Cost |
|----------|----------|----------------|------------|------|
| **Standalone** | Development, Small scale | 15K events/s | Low | $ |
| **Microservice** | Production, Enterprise | 100K+ events/s | High | $$$ |
| **Sidecar** | Low-latency, Service mesh | 2K events/s per pod | Medium | $$ |

## Data Flow

```
Observatory ─[telemetry]─▶ Ingestion ─[events]─▶ Detection ─[anomalies]─▶ Alert
                              │                       │                       │
                              ▼                       ▼                       ▼
                          Storage                 Storage                 Shield
                                                                      Incident Manager
                                                                     Governance Dashboard
```

## Core Components

1. **Telemetry Ingestion Service** - Receive and validate events (gRPC/HTTP/Kafka)
2. **Anomaly Detection Engine** - Real-time detection (Statistical/Rule/ML/Behavioral)
3. **Alert Manager** - Route and deliver notifications
4. **Configuration Service** - Dynamic configuration (etcd/Consul)
5. **Storage Layer** - Time-series, events, alerts (InfluxDB/Elasticsearch/PostgreSQL)
6. **API Gateway** - External API (REST/gRPC/GraphQL)

## Data Schemas

All data schemas are defined using JSON Schema with comprehensive examples:

- **[Telemetry Event Schema](../ARCHITECTURE.md#1-telemetry-event-schema)** - Standard event format from Observatory
- **[Anomaly Event Schema](../ARCHITECTURE.md#2-anomaly-event-schema)** - Detected anomaly format
- **[Alert Definition Schema](../ARCHITECTURE.md#3-alert-definition-schema)** - Alert routing rules
- **[Configuration Schema](../ARCHITECTURE.md#4-configuration-schema)** - Master configuration
- **[Metrics Aggregation Schema](../ARCHITECTURE.md#5-metrics-aggregation-schema)** - Aggregated metrics

## Integration Patterns

### Observatory → Sentinel

- **Push**: gRPC streaming, HTTP POST, Kafka producer
- **Protocol**: ProtoBuf, JSON
- **Batching**: Configurable batch size and timeout
- **Reliability**: Retry with exponential backoff

### Sentinel → Shield

- **Action Enforcement**: Block, rate limit, throttle, quarantine
- **Protocol**: gRPC
- **Latency**: < 50ms p99
- **Reliability**: Circuit breaker, retry

### Sentinel → Incident Manager

- **Auto-creation**: Critical anomalies → P1 incidents
- **Protocol**: REST/Webhook
- **Enrichment**: Full anomaly context, remediation steps

### Sentinel → Governance Dashboard

- **Query API**: GraphQL, REST
- **Real-time**: WebSocket subscriptions
- **Metrics**: Prometheus export

## Performance Targets

| Metric | Target |
|--------|--------|
| Ingestion Latency (p99) | < 10ms |
| Detection Latency (p99) | < 100ms |
| Alert Delivery (p99) | < 500ms |
| Throughput (Microservice) | 100K events/s |
| Availability | 99.9% |
| Data Loss Rate | < 0.01% |

## Capacity Planning

### Quick Calculator

```
CPU cores needed = (events_per_second ÷ 2000) × 1.5
Memory GB needed = (events_per_second ÷ 4000) × 1.5

Example for 50K events/s:
  CPU = (50,000 ÷ 2,000) × 1.5 = 37.5 ≈ 40 cores
  Memory = (50,000 ÷ 4,000) × 1.5 = 18.75 ≈ 20 GB
```

### Sizing Guide

| Target Load | Deployment Type | Resources | Monthly Cost (AWS) |
|-------------|----------------|-----------|-------------------|
| 1K events/s | Standalone | 2C, 4GB | $125 |
| 10K events/s | Standalone | 8C, 16GB | $250 |
| 50K events/s | Microservice | 50C, 100GB | $1,500 |
| 100K events/s | Microservice | 80C, 160GB | $3,000 |

## Support and Community

- **Documentation**: https://docs.llm-sentinel.io
- **GitHub Issues**: https://github.com/llm-sentinel/sentinel/issues
- **Community Slack**: https://llm-sentinel.slack.com
- **Email Support**: support@llm-sentinel.io
- **Enterprise Support**: enterprise@llm-sentinel.io

## Contributing to Documentation

We welcome contributions to improve documentation:

1. **Fix errors**: Submit PR with corrections
2. **Add examples**: Share your integration examples
3. **Improve clarity**: Suggest better explanations
4. **Add translations**: Help translate docs

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Documentation is licensed under [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/).

Code examples are licensed under [Apache License 2.0](../LICENSE).

---

**Last Updated**: 2025-11-06
**Documentation Version**: v1.0.0
**Sentinel Version**: v1.0.0
