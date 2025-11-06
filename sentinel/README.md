# sentinel

Main binary for LLM-Sentinel anomaly detection system.

## Overview

The sentinel binary orchestrates all system components:

- Kafka ingestion pipeline
- Multi-algorithm detection engine
- InfluxDB storage and caching
- RabbitMQ alerting
- REST API server

## Installation

```bash
cargo install sentinel
```

Or build from source:

```bash
git clone https://github.com/globalbusinessadvisors/llm-sentinel
cd llm-sentinel
cargo build --release
```

## Usage

```bash
# Start with default configuration
sentinel --config sentinel.yaml

# Specify custom config
sentinel --config /etc/sentinel/config.yaml

# Show version
sentinel --version
```

## Configuration

Create a `sentinel.yaml` file:

```yaml
ingestion:
  kafka:
    brokers: ["localhost:9092"]
    topic: "llm.telemetry"

detection:
  enabled_detectors: ["zscore", "iqr"]

storage:
  influxdb:
    url: "http://localhost:8086"

api:
  bind_addr: "0.0.0.0:8080"
```

## Docker

```bash
docker run -v $(pwd)/sentinel.yaml:/etc/sentinel/sentinel.yaml \
  ghcr.io/globalbusinessadvisors/llm-sentinel:latest
```

## License

Apache-2.0
