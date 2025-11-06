# LLM-Sentinel Integration Quick Reference
## Fast Access Guide for Developers

---

## INTEGRATION CHECKLIST

### Pre-Integration Requirements

- [ ] OpenTelemetry SDK installed and configured
- [ ] Kafka/RabbitMQ message broker accessible
- [ ] API Gateway configured with service routes
- [ ] Service discovery mechanism in place (Kubernetes DNS / Consul)
- [ ] Secrets management configured (Vault / AWS Secrets Manager)
- [ ] Time-series database available (Prometheus / InfluxDB)

### LLM-Observatory Integration

- [ ] OTLP collector endpoint configured
- [ ] Kafka topic `llm.telemetry` created
- [ ] Consumer group `sentinel-anomaly` registered
- [ ] Telemetry schema validation implemented
- [ ] Test traces successfully received

### LLM-Incident-Manager Integration

- [ ] Alert queue `incidents.high-priority` created
- [ ] Alert schema documented and validated
- [ ] Webhook endpoints configured
- [ ] Severity-based routing rules defined
- [ ] Test alerts successfully delivered

### LLM-Governance-Dashboard Integration

- [ ] Metrics endpoint `/api/v1/metrics` exposed
- [ ] Prometheus scrape config updated
- [ ] Grafana dashboards imported
- [ ] WebSocket connection tested (if real-time updates needed)
- [ ] Historical query API tested

### LLM-Shield Integration

- [ ] Security events queue configured
- [ ] Event schema mapped to anomaly patterns
- [ ] Test security events processed
- [ ] Alert correlation logic implemented

---

## DATA SCHEMAS

### Telemetry Event Schema (from LLM-Observatory)

```json
{
  "timestamp": "2025-11-06T14:32:10.123Z",
  "trace_id": "a1b2c3d4e5f6",
  "span_id": "f6e5d4c3b2a1",
  "service_name": "llm-api-service",
  "operation": "llm.chat.completion",
  "duration_ms": 1250,
  "attributes": {
    "llm.model": "gpt-4-0125-preview",
    "llm.provider": "openai",
    "llm.request.type": "chat",
    "llm.usage.prompt_tokens": 150,
    "llm.usage.completion_tokens": 420,
    "llm.usage.total_tokens": 570,
    "http.status_code": 200,
    "user.id": "user-12345",
    "api.key_id": "key-abc123"
  },
  "events": [],
  "status": "OK"
}
```

### Anomaly Alert Schema (to LLM-Incident-Manager)

```json
{
  "alert_id": "anomaly-2025-11-06-001",
  "timestamp": "2025-11-06T14:33:00.000Z",
  "severity": "high",
  "anomaly_type": "latency_spike",
  "service": "llm-api-service",
  "metrics": {
    "current_value": 1250,
    "baseline_value": 380,
    "std_deviation": 4.2,
    "percentile_rank": 99.8
  },
  "context": {
    "time_window": "5m",
    "affected_requests": 47,
    "model_version": "gpt-4-0125-preview",
    "deployment_timestamp": "2025-11-06T10:00:00.000Z"
  },
  "analysis": {
    "root_cause_suggestions": [
      "Recent model version update (2h ago)",
      "Increased token usage pattern detected",
      "Possible upstream API degradation"
    ],
    "confidence_score": 0.87
  },
  "recommended_actions": [
    "Check OpenAI status page",
    "Review recent deployment changes",
    "Consider temporary rollback to previous model version"
  ]
}
```

### Security Event Schema (from LLM-Shield)

```json
{
  "event_id": "sec-2025-11-06-042",
  "timestamp": "2025-11-06T14:32:05.000Z",
  "event_type": "jailbreak_attempt",
  "severity": "high",
  "source": {
    "user_id": "user-malicious-001",
    "api_key_id": "key-xyz789",
    "ip_address": "203.0.113.42"
  },
  "detection": {
    "method": "pattern_matching",
    "confidence": 0.95,
    "matched_patterns": ["ignore_previous_instructions", "you_are_now"]
  },
  "action_taken": "blocked",
  "metadata": {
    "user_agent": "curl/7.68.0",
    "endpoint": "/v1/chat/completions",
    "request_id": "req-abc123"
  }
}
```

---

## API ENDPOINTS

### LLM-Sentinel Exposed APIs

#### 1. Real-Time Anomaly Detection
```http
POST /api/v1/detect
Content-Type: application/json
Authorization: Bearer <token>

{
  "metrics": {
    "latency_ms": 2300,
    "token_count": 5000,
    "error_count": 3
  },
  "context": {
    "service": "llm-api",
    "model": "gpt-4",
    "timestamp": "2025-11-06T14:32:10Z"
  }
}

Response 200 OK:
{
  "is_anomaly": true,
  "anomaly_score": 0.92,
  "anomaly_type": "latency_spike",
  "explanation": "Latency 4.5 std deviations above 7-day baseline (500ms)",
  "severity": "high"
}
```

#### 2. Query Historical Anomalies
```http
GET /api/v1/anomalies?start=2025-11-01T00:00:00Z&end=2025-11-06T23:59:59Z&severity=high&limit=50
Authorization: Bearer <token>

Response 200 OK:
{
  "total": 127,
  "page": 1,
  "page_size": 50,
  "anomalies": [
    {
      "alert_id": "anomaly-2025-11-06-001",
      "timestamp": "2025-11-06T14:33:00Z",
      "severity": "high",
      "anomaly_type": "latency_spike",
      "service": "llm-api-service",
      "status": "resolved"
    }
    // ... more anomalies
  ]
}
```

#### 3. Health Check
```http
GET /api/v1/health

Response 200 OK:
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "last_telemetry_received": "2025-11-06T14:33:45Z",
  "processing_lag_seconds": 2.3,
  "anomaly_detection_rate": 0.02
}
```

#### 4. Metrics (Prometheus Format)
```http
GET /api/v1/metrics

Response 200 OK (text/plain):
# HELP sentinel_anomalies_detected_total Total anomalies detected
# TYPE sentinel_anomalies_detected_total counter
sentinel_anomalies_detected_total{severity="critical"} 12
sentinel_anomalies_detected_total{severity="high"} 47
sentinel_anomalies_detected_total{severity="medium"} 203
sentinel_anomalies_detected_total{severity="low"} 891

# HELP sentinel_detection_latency_seconds Time to detect anomaly
# TYPE sentinel_detection_latency_seconds histogram
sentinel_detection_latency_seconds_bucket{le="0.1"} 8234
sentinel_detection_latency_seconds_bucket{le="0.5"} 9012
sentinel_detection_latency_seconds_bucket{le="1.0"} 9234
sentinel_detection_latency_seconds_sum 4523.2
sentinel_detection_latency_seconds_count 9345

# HELP sentinel_false_positive_rate False positive rate
# TYPE sentinel_false_positive_rate gauge
sentinel_false_positive_rate 0.05
```

#### 5. Feedback Submission
```http
POST /api/v1/feedback
Content-Type: application/json
Authorization: Bearer <token>

{
  "alert_id": "anomaly-2025-11-06-001",
  "feedback": "false_positive",
  "reason": "Planned load testing, not an actual anomaly",
  "submitted_by": "engineer@company.com"
}

Response 201 Created:
{
  "message": "Feedback recorded successfully",
  "feedback_id": "fb-20251106-001"
}
```

---

## KAFKA TOPICS

### Topic: `llm.telemetry` (Input)
- **Producers**: LLM-Observatory
- **Consumers**: LLM-Sentinel (consumer group: `sentinel-anomaly`)
- **Message Format**: JSON (Telemetry Event Schema)
- **Retention**: 7 days
- **Partitions**: 6 (partition by service name)
- **Replication Factor**: 3

### Topic: `llm.telemetry.edge` (Input)
- **Producers**: LLM-Edge-Agent (aggregated)
- **Consumers**: LLM-Sentinel (consumer group: `sentinel-edge`)
- **Message Format**: JSON (Edge Telemetry Schema)
- **Retention**: 3 days
- **Partitions**: 3 (partition by region)

### Topic: `anomaly.alerts` (Output)
- **Producers**: LLM-Sentinel
- **Consumers**: LLM-Incident-Manager, LLM-Governance-Dashboard
- **Message Format**: JSON (Anomaly Alert Schema)
- **Retention**: 30 days
- **Partitions**: 4 (partition by severity)

### Topic: `security.events` (Input)
- **Producers**: LLM-Shield
- **Consumers**: LLM-Sentinel (consumer group: `sentinel-security`)
- **Message Format**: JSON (Security Event Schema)
- **Retention**: 90 days (compliance requirement)
- **Partitions**: 2

---

## RABBITMQ QUEUES

### Queue: `incidents.critical`
- **Type**: Priority queue
- **Producers**: LLM-Sentinel
- **Consumers**: LLM-Incident-Manager
- **Message TTL**: None (persist until consumed)
- **Max Priority**: 10
- **Dead Letter Exchange**: `incidents.dlx`

### Queue: `incidents.high-priority`
- **Type**: Durable queue
- **Producers**: LLM-Sentinel
- **Consumers**: LLM-Incident-Manager
- **Message TTL**: 1 hour
- **Max Length**: 10000 messages

### Queue: `security.events`
- **Type**: Durable queue
- **Producers**: LLM-Shield
- **Consumers**: LLM-Sentinel
- **Message TTL**: 24 hours
- **Max Length**: 50000 messages

---

## PROMETHEUS METRICS

### Key Metrics to Monitor

```promql
# Anomaly detection rate over time
rate(sentinel_anomalies_detected_total[5m])

# High-severity anomalies per hour
increase(sentinel_anomalies_detected_total{severity="high"}[1h])

# Detection latency (95th percentile)
histogram_quantile(0.95, rate(sentinel_detection_latency_seconds_bucket[5m]))

# False positive trend
sentinel_false_positive_rate

# Processing lag (should be < 5 seconds)
sentinel_kafka_consumer_lag_seconds

# Service availability (should be 1.0)
up{job="llm-sentinel"}
```

---

## GRAFANA DASHBOARD QUERIES

### Panel 1: Anomalies Over Time (Graph)
```promql
sum by (severity) (
  increase(sentinel_anomalies_detected_total[1h])
)
```

### Panel 2: Current Anomaly Rate (Stat)
```promql
sum(rate(sentinel_anomalies_detected_total[5m])) * 60
```
**Unit**: anomalies per minute

### Panel 3: Detection Performance (Heatmap)
```promql
sum(rate(sentinel_detection_latency_seconds_bucket[5m])) by (le)
```

### Panel 4: Service Health (Status)
```promql
up{job="llm-sentinel"}
```

### Panel 5: Top Anomaly Types (Pie Chart)
```promql
sum by (anomaly_type) (
  increase(sentinel_anomalies_detected_total[24h])
)
```

---

## CONFIGURATION FILES

### Kubernetes Deployment (llm-sentinel.yaml)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-sentinel
  namespace: llm-platform
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-sentinel
  template:
    metadata:
      labels:
        app: llm-sentinel
    spec:
      containers:
      - name: sentinel
        image: llm-platform/llm-sentinel:1.0.0
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: KAFKA_BROKERS
          value: "kafka-1:9092,kafka-2:9092,kafka-3:9092"
        - name: KAFKA_TOPIC_TELEMETRY
          value: "llm.telemetry"
        - name: RABBITMQ_HOST
          value: "rabbitmq-service"
        - name: RABBITMQ_PORT
          value: "5672"
        - name: PROMETHEUS_PUSHGATEWAY
          value: "http://prometheus-pushgateway:9091"
        - name: LOG_LEVEL
          value: "INFO"
        resources:
          requests:
            cpu: "500m"
            memory: "1Gi"
          limits:
            cpu: "2000m"
            memory: "4Gi"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: llm-sentinel-service
  namespace: llm-platform
spec:
  selector:
    app: llm-sentinel
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

### Prometheus Scrape Config

```yaml
scrape_configs:
  - job_name: 'llm-sentinel'
    scrape_interval: 15s
    static_configs:
      - targets:
          - 'llm-sentinel-service:9090'
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
        replacement: 'llm-sentinel'
```

### Kafka Consumer Configuration

```python
# Python example using confluent-kafka
from confluent_kafka import Consumer, KafkaError

consumer_config = {
    'bootstrap.servers': 'kafka-1:9092,kafka-2:9092,kafka-3:9092',
    'group.id': 'sentinel-anomaly',
    'auto.offset.reset': 'earliest',
    'enable.auto.commit': False,
    'max.poll.interval.ms': 300000,
    'session.timeout.ms': 10000,
}

consumer = Consumer(consumer_config)
consumer.subscribe(['llm.telemetry', 'security.events'])
```

---

## TESTING

### Integration Test Checklist

#### Test 1: Telemetry Processing
```bash
# Send test telemetry event to Kafka
kafka-console-producer --broker-list kafka-1:9092 --topic llm.telemetry < test_telemetry.json

# Verify LLM-Sentinel received and processed
curl http://llm-sentinel-service/api/v1/health | jq .last_telemetry_received

# Check logs for processing
kubectl logs -n llm-platform deployment/llm-sentinel | grep "Processed telemetry"
```

#### Test 2: Anomaly Detection
```bash
# Inject anomalous data (high latency)
curl -X POST http://llm-sentinel-service/api/v1/detect \
  -H "Content-Type: application/json" \
  -d '{
    "metrics": {"latency_ms": 10000},
    "context": {"service": "test-service"}
  }'

# Expected response: is_anomaly=true
```

#### Test 3: Alert Delivery
```bash
# Verify alert published to RabbitMQ
rabbitmqadmin get queue=incidents.high-priority count=1

# Check LLM-Incident-Manager received alert
curl http://llm-incident-manager/api/v1/incidents?status=open
```

#### Test 4: Metrics Exposure
```bash
# Verify Prometheus metrics
curl http://llm-sentinel-service:9090/api/v1/metrics | grep sentinel_anomalies_detected_total

# Query Prometheus directly
curl -G http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=sentinel_anomalies_detected_total'
```

---

## TROUBLESHOOTING

### Issue 1: No Telemetry Received

**Symptoms**: `last_telemetry_received` field in health check is stale

**Diagnosis**:
```bash
# Check Kafka topic has data
kafka-console-consumer --bootstrap-server kafka-1:9092 \
  --topic llm.telemetry --from-beginning --max-messages 1

# Check consumer group lag
kafka-consumer-groups --bootstrap-server kafka-1:9092 \
  --describe --group sentinel-anomaly

# Check LLM-Sentinel logs
kubectl logs -n llm-platform deployment/llm-sentinel --tail=100
```

**Resolution**:
- Verify Kafka broker connectivity
- Check consumer group offset (may need reset)
- Ensure topic partition assignment correct

### Issue 2: High False Positive Rate

**Symptoms**: `sentinel_false_positive_rate` > 0.10

**Diagnosis**:
```bash
# Query recent false positives from feedback API
curl http://llm-sentinel-service/api/v1/anomalies?feedback=false_positive&limit=100

# Analyze common patterns
jq '.anomalies | group_by(.anomaly_type) | map({type: .[0].anomaly_type, count: length})'
```

**Resolution**:
- Adjust baseline calculation window (e.g., 7 days → 14 days)
- Tune detection thresholds (e.g., 3 std dev → 4 std dev)
- Implement contextual baselines (per service, per model)

### Issue 3: Alert Storm

**Symptoms**: Hundreds of alerts generated in short time

**Diagnosis**:
```bash
# Check recent alert volume
curl http://llm-sentinel-service/api/v1/metrics | grep sentinel_anomalies_detected_total

# Query Prometheus for spike
curl -G http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=rate(sentinel_anomalies_detected_total[1m])'
```

**Resolution**:
- Implement alert deduplication (group similar alerts)
- Add rate limiting on alert publishing
- Increase detection window to reduce noise
- Temporary: Raise severity threshold for alerts

### Issue 4: Processing Lag

**Symptoms**: `sentinel_kafka_consumer_lag_seconds` > 10

**Diagnosis**:
```bash
# Check consumer lag in Kafka
kafka-consumer-groups --bootstrap-server kafka-1:9092 \
  --describe --group sentinel-anomaly

# Check pod resource usage
kubectl top pods -n llm-platform | grep llm-sentinel
```

**Resolution**:
- Scale up replicas (increase Kubernetes deployment replicas)
- Optimize processing pipeline (batch processing, async operations)
- Increase partition count on Kafka topic
- Allocate more CPU/memory resources

---

## ENVIRONMENT VARIABLES

### Required Configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `KAFKA_BROKERS` | Kafka broker addresses | `kafka-1:9092,kafka-2:9092` |
| `KAFKA_TOPIC_TELEMETRY` | Telemetry input topic | `llm.telemetry` |
| `KAFKA_GROUP_ID` | Consumer group ID | `sentinel-anomaly` |
| `RABBITMQ_HOST` | RabbitMQ server | `rabbitmq-service` |
| `RABBITMQ_PORT` | RabbitMQ port | `5672` |
| `RABBITMQ_USER` | RabbitMQ username | `sentinel-user` |
| `RABBITMQ_PASSWORD` | RabbitMQ password (secret) | `<from-secrets>` |
| `PROMETHEUS_PUSHGATEWAY` | Prometheus pushgateway URL | `http://prometheus-pushgateway:9091` |
| `API_PORT` | API server port | `8080` |
| `METRICS_PORT` | Metrics server port | `9090` |
| `LOG_LEVEL` | Logging verbosity | `INFO` / `DEBUG` |

### Optional Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `DETECTION_WINDOW_SECONDS` | Time window for baseline | `300` (5 min) |
| `BASELINE_LOOKBACK_DAYS` | Historical data for baseline | `7` |
| `STD_DEV_THRESHOLD` | Std deviations for anomaly | `3.0` |
| `ALERT_COOLDOWN_SECONDS` | Min time between duplicate alerts | `300` (5 min) |
| `MAX_ALERT_RATE_PER_MINUTE` | Rate limit for alerts | `10` |
| `VECTOR_DB_URL` | Vector database for RAG detection | *(optional)* |
| `ENABLE_LLM_ANALYSIS` | Use LLM for root cause analysis | `false` |

---

## SECURITY CHECKLIST

- [ ] **mTLS enabled** for inter-service communication
- [ ] **API authentication** configured (JWT / API keys)
- [ ] **Secrets** stored in Vault / Secrets Manager (not environment variables)
- [ ] **Network policies** restrict traffic to authorized services
- [ ] **RBAC** configured for Kubernetes service account
- [ ] **TLS 1.3** enforced for external APIs
- [ ] **PII detection** enabled for logs and alerts
- [ ] **Audit logging** active for all API requests
- [ ] **Rate limiting** configured on API endpoints
- [ ] **Input validation** implemented for all API inputs

---

## QUICK COMMANDS

### Development
```bash
# Run locally with Docker
docker run -e KAFKA_BROKERS=localhost:9092 llm-sentinel:latest

# Tail logs
kubectl logs -f -n llm-platform deployment/llm-sentinel

# Port forward for local testing
kubectl port-forward -n llm-platform svc/llm-sentinel-service 8080:80
```

### Operations
```bash
# Scale deployment
kubectl scale deployment llm-sentinel -n llm-platform --replicas=5

# Rolling update
kubectl set image deployment/llm-sentinel sentinel=llm-sentinel:1.1.0 -n llm-platform

# Restart deployment
kubectl rollout restart deployment/llm-sentinel -n llm-platform

# Check rollout status
kubectl rollout status deployment/llm-sentinel -n llm-platform
```

### Debugging
```bash
# Exec into pod
kubectl exec -it -n llm-platform deployment/llm-sentinel -- /bin/bash

# Check Kafka connectivity
kubectl exec -it -n llm-platform deployment/llm-sentinel -- \
  kafka-console-consumer --bootstrap-server kafka-1:9092 --topic llm.telemetry --max-messages 1

# Test API endpoint
curl http://localhost:8080/api/v1/health | jq
```

---

**Document Version**: 1.0
**Last Updated**: 2025-11-06
**Maintained By**: Platform Engineering Team
