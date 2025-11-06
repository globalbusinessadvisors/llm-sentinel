# LLM-Sentinel Deployment Guide

## Table of Contents

1. [Quick Start](#quick-start)
2. [Standalone Deployment](#standalone-deployment)
3. [Kubernetes Deployment](#kubernetes-deployment)
4. [Sidecar Deployment](#sidecar-deployment)
5. [Production Checklist](#production-checklist)
6. [Monitoring and Observability](#monitoring-and-observability)
7. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Prerequisites

- Go 1.21+ (for building from source)
- Docker 24.0+ (for containerized deployment)
- Kubernetes 1.27+ (for orchestrated deployment)

### 5-Minute Demo

```bash
# Pull the latest image
docker pull llm-sentinel:latest

# Run with minimal configuration
docker run -d \
  --name sentinel \
  -p 8080:8080 \
  -p 9090:9090 \
  -e MODE=standalone \
  -e LOG_LEVEL=info \
  llm-sentinel:latest

# Send test telemetry
curl -X POST http://localhost:8080/api/v1/telemetry/events \
  -H "Content-Type: application/json" \
  -d '{
    "event_id": "test-001",
    "timestamp": "2025-11-06T12:00:00Z",
    "event_type": "llm.response",
    "application_id": "test-app",
    "data": {
      "status": "success",
      "latency_ms": 1250,
      "total_tokens": 225,
      "cost": 0.045
    }
  }'

# Check status
curl http://localhost:8080/api/v1/health
```

---

## Standalone Deployment

### Binary Installation

**Linux/macOS:**

```bash
# Download latest release
VERSION=v1.0.0
curl -LO https://github.com/llm-sentinel/sentinel/releases/download/${VERSION}/sentinel-${VERSION}-linux-amd64.tar.gz

# Extract
tar -xzf sentinel-${VERSION}-linux-amd64.tar.gz

# Move to PATH
sudo mv sentinel /usr/local/bin/

# Verify installation
sentinel version
```

**Configuration:**

```bash
# Create configuration directory
sudo mkdir -p /etc/sentinel

# Create basic configuration
cat > /etc/sentinel/config.yaml <<EOF
mode: standalone

ingestion:
  grpc:
    port: 9090
  http:
    port: 8080

storage:
  path: /var/sentinel/data
  retention:
    raw: 7d
    aggregated: 90d

detectors:
  configFile: /etc/sentinel/detectors.yaml

alerts:
  configFile: /etc/sentinel/alerts.yaml

observability:
  metrics:
    enabled: true
    port: 9102
  logging:
    level: info
    format: json
EOF
```

**Detector Configuration:**

```bash
cat > /etc/sentinel/detectors.yaml <<EOF
detectors:
  - id: high-latency
    name: High Latency Detector
    type: threshold
    enabled: true
    config:
      metric: latency_ms
      condition: greater_than
      threshold: 5000
      window: 5m
    actions:
      - alert: high-latency-alert
        severity: warning

  - id: high-cost
    name: High Cost Detector
    type: threshold
    enabled: true
    config:
      metric: cost
      condition: greater_than
      threshold: 1.0
      window: 1m
    actions:
      - alert: high-cost-alert
        severity: critical

  - id: error-rate
    name: Error Rate Detector
    type: statistical
    enabled: true
    config:
      metric: error_rate
      strategy: zscore
      threshold: 3.0
      window: 10m
    actions:
      - alert: error-spike-alert
        severity: warning
EOF
```

**Alert Configuration:**

```bash
cat > /etc/sentinel/alerts.yaml <<EOF
routing:
  - match:
      severity: critical
    receivers:
      - email-ops
      - slack-incidents
    repeat_interval: 15m

  - match:
      severity: warning
    receivers:
      - slack-monitoring
    repeat_interval: 1h

receivers:
  - name: email-ops
    type: email
    config:
      smtp:
        host: smtp.gmail.com
        port: 587
        username: alerts@example.com
        password: \${SMTP_PASSWORD}
      from: sentinel@example.com
      to:
        - ops-team@example.com

  - name: slack-incidents
    type: slack
    config:
      webhookUrl: \${SLACK_WEBHOOK_URL}
      channel: "#incidents"

  - name: slack-monitoring
    type: slack
    config:
      webhookUrl: \${SLACK_WEBHOOK_URL}
      channel: "#monitoring"
EOF
```

**Systemd Service:**

```bash
# Create systemd service
sudo cat > /etc/systemd/system/sentinel.service <<EOF
[Unit]
Description=LLM Sentinel Service
After=network.target

[Service]
Type=simple
User=sentinel
Group=sentinel
Environment="SMTP_PASSWORD=your-smtp-password"
Environment="SLACK_WEBHOOK_URL=your-slack-webhook-url"
ExecStart=/usr/local/bin/sentinel --config /etc/sentinel/config.yaml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Create sentinel user
sudo useradd -r -s /bin/false sentinel

# Create data directory
sudo mkdir -p /var/sentinel/data
sudo chown -R sentinel:sentinel /var/sentinel

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable sentinel
sudo systemctl start sentinel

# Check status
sudo systemctl status sentinel
sudo journalctl -u sentinel -f
```

### Docker Compose Deployment

```yaml
# docker-compose.yml
version: '3.8'

services:
  sentinel:
    image: llm-sentinel:latest
    container_name: sentinel
    ports:
      - "8080:8080"   # HTTP API
      - "9090:9090"   # gRPC
      - "9102:9102"   # Metrics
    volumes:
      - ./config:/etc/sentinel:ro
      - sentinel-data:/var/sentinel/data
    environment:
      - MODE=standalone
      - LOG_LEVEL=info
      - SMTP_PASSWORD=${SMTP_PASSWORD}
      - SLACK_WEBHOOK_URL=${SLACK_WEBHOOK_URL}
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  # Optional: Prometheus for metrics collection
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped

  # Optional: Grafana for visualization
  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3000:3000"
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./grafana/datasources:/etc/grafana/provisioning/datasources:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
      - GF_INSTALL_PLUGINS=grafana-clock-panel
    restart: unless-stopped

volumes:
  sentinel-data:
  prometheus-data:
  grafana-data:

networks:
  default:
    name: sentinel-network
```

**Prometheus Configuration:**

```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'sentinel'
    static_configs:
      - targets: ['sentinel:9102']
        labels:
          service: 'sentinel'
          environment: 'production'
```

**Starting the Stack:**

```bash
# Create .env file
cat > .env <<EOF
SMTP_PASSWORD=your-smtp-password
SLACK_WEBHOOK_URL=your-slack-webhook
EOF

# Start services
docker-compose up -d

# View logs
docker-compose logs -f sentinel

# Stop services
docker-compose down
```

---

## Kubernetes Deployment

### Prerequisites

```bash
# Create namespace
kubectl create namespace sentinel

# Create secrets
kubectl create secret generic sentinel-secrets \
  --from-literal=smtp-password=your-smtp-password \
  --from-literal=slack-webhook=your-slack-webhook \
  -n sentinel
```

### ConfigMap for Configuration

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: sentinel-config
  namespace: sentinel
data:
  config.yaml: |
    mode: distributed

    ingestion:
      grpc:
        port: 9090
        tls:
          enabled: false
      http:
        port: 8080
      kafka:
        enabled: true
        brokers:
          - kafka.kafka.svc.cluster.local:9092
        topics:
          - llm-telemetry
        consumerGroup: sentinel-prod

    detection:
      workers: 8
      batchSize: 100

    storage:
      timeseries:
        type: prometheus
        url: http://prometheus.monitoring.svc.cluster.local:9090
      events:
        type: elasticsearch
        urls:
          - http://elasticsearch.logging.svc.cluster.local:9200

    observability:
      metrics:
        enabled: true
        port: 9102
      logging:
        level: info
        format: json

  detectors.yaml: |
    detectors:
      - id: high-latency
        type: threshold
        enabled: true
        config:
          metric: latency_ms
          condition: greater_than
          threshold: 5000
          window: 5m
        actions:
          - alert: high-latency-alert
            severity: warning

  alerts.yaml: |
    routing:
      - match:
          severity: critical
        receivers:
          - slack-incidents

    receivers:
      - name: slack-incidents
        type: slack
        config:
          webhookUrl: ${SLACK_WEBHOOK_URL}
          channel: "#incidents"
```

### Deployment Manifests

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: sentinel
  namespace: sentinel
  labels:
    app: sentinel
    component: all-in-one
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: sentinel
  template:
    metadata:
      labels:
        app: sentinel
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9102"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: sentinel
      containers:
      - name: sentinel
        image: llm-sentinel:v1.0.0
        imagePullPolicy: IfNotPresent
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: grpc
          containerPort: 9090
          protocol: TCP
        - name: metrics
          containerPort: 9102
          protocol: TCP
        env:
        - name: MODE
          value: "distributed"
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: SMTP_PASSWORD
          valueFrom:
            secretKeyRef:
              name: sentinel-secrets
              key: smtp-password
        - name: SLACK_WEBHOOK_URL
          valueFrom:
            secretKeyRef:
              name: sentinel-secrets
              key: slack-webhook
        volumeMounts:
        - name: config
          mountPath: /etc/sentinel
          readOnly: true
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
      volumes:
      - name: config
        configMap:
          name: sentinel-config

---
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: sentinel
  namespace: sentinel
  labels:
    app: sentinel
spec:
  type: LoadBalancer
  selector:
    app: sentinel
  ports:
  - name: http
    port: 8080
    targetPort: 8080
    protocol: TCP
  - name: grpc
    port: 9090
    targetPort: 9090
    protocol: TCP
  sessionAffinity: ClientIP

---
# service-account.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sentinel
  namespace: sentinel

---
# hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: sentinel-hpa
  namespace: sentinel
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: sentinel
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 100
        periodSeconds: 30

---
# pdb.yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: sentinel-pdb
  namespace: sentinel
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: sentinel
```

### Deploy to Kubernetes

```bash
# Apply all manifests
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml

# Wait for rollout
kubectl rollout status deployment/sentinel -n sentinel

# Check pods
kubectl get pods -n sentinel

# Check service
kubectl get svc sentinel -n sentinel

# Get external IP (LoadBalancer)
kubectl get svc sentinel -n sentinel -o jsonpath='{.status.loadBalancer.ingress[0].ip}'

# View logs
kubectl logs -f deployment/sentinel -n sentinel

# Test the service
SENTINEL_IP=$(kubectl get svc sentinel -n sentinel -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
curl http://${SENTINEL_IP}:8080/health
```

### Helm Deployment

```bash
# Add Sentinel Helm repository
helm repo add sentinel https://charts.llm-sentinel.io
helm repo update

# Install with default values
helm install sentinel sentinel/sentinel \
  --namespace sentinel \
  --create-namespace

# Install with custom values
cat > values.yaml <<EOF
replicaCount: 3

image:
  repository: llm-sentinel
  tag: v1.0.0
  pullPolicy: IfNotPresent

resources:
  requests:
    memory: 2Gi
    cpu: 1000m
  limits:
    memory: 4Gi
    cpu: 2000m

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70

ingestion:
  grpc:
    port: 9090
  http:
    port: 8080

storage:
  timeseries:
    type: prometheus
    url: http://prometheus:9090
  events:
    type: elasticsearch
    url: http://elasticsearch:9200

integrations:
  observatory:
    enabled: true
    endpoint: grpc://observatory:9090
  shield:
    enabled: true
    endpoint: grpc://shield:9090

secrets:
  smtp_password: your-smtp-password
  slack_webhook: your-slack-webhook
EOF

helm install sentinel sentinel/sentinel \
  --namespace sentinel \
  --create-namespace \
  --values values.yaml

# Upgrade
helm upgrade sentinel sentinel/sentinel \
  --namespace sentinel \
  --values values.yaml

# Uninstall
helm uninstall sentinel --namespace sentinel
```

---

## Sidecar Deployment

### Kubernetes Sidecar Injection

**Manual Injection:**

```yaml
# application-with-sidecar.yaml
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
    spec:
      containers:
      # Main application
      - name: chatbot
        image: llm-chatbot:v1.0.0
        ports:
        - containerPort: 8080
          name: http
        env:
        - name: SENTINEL_ENDPOINT
          value: "localhost:9090"
        resources:
          requests:
            memory: 4Gi
            cpu: 2000m

      # Sentinel sidecar
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
          valueFrom:
            fieldRef:
              fieldPath: metadata.labels['app']
        - name: CENTRAL_AGGREGATOR
          value: "sentinel.sentinel.svc.cluster.local:9090"
        resources:
          requests:
            memory: 512Mi
            cpu: 250m
          limits:
            memory: 1Gi
            cpu: 500m
        livenessProbe:
          httpGet:
            path: /health
            port: 8081
          initialDelaySeconds: 10
          periodSeconds: 10
```

**Automatic Injection with Mutating Webhook:**

```yaml
# Install Sentinel webhook
kubectl apply -f https://raw.githubusercontent.com/llm-sentinel/sentinel/main/deploy/webhook.yaml

# Label namespace for auto-injection
kubectl label namespace applications sentinel-injection=enabled

# Deploy application (sidecar injected automatically)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-chatbot
  namespace: applications
  annotations:
    sentinel.io/inject: "true"
    sentinel.io/sidecar-cpu: "250m"
    sentinel.io/sidecar-memory: "512Mi"
spec:
  # ... application spec
```

---

## Production Checklist

### Pre-Deployment

- [ ] **Infrastructure Requirements**
  - [ ] Kubernetes cluster sized appropriately
  - [ ] Storage provisioned (TimescaleDB, Elasticsearch, PostgreSQL)
  - [ ] Message queue deployed (Kafka/RabbitMQ)
  - [ ] Configuration backend setup (etcd/Consul)

- [ ] **Security**
  - [ ] TLS certificates generated
  - [ ] Secrets management configured (Vault, AWS Secrets Manager)
  - [ ] RBAC policies defined
  - [ ] Network policies configured
  - [ ] Service mesh installed (optional: Istio, Linkerd)

- [ ] **Configuration**
  - [ ] Detector configurations reviewed
  - [ ] Alert routing rules defined
  - [ ] Integration endpoints configured
  - [ ] Retention policies set

- [ ] **Monitoring**
  - [ ] Prometheus/metrics collection setup
  - [ ] Grafana dashboards imported
  - [ ] Log aggregation configured (ELK, Loki)
  - [ ] Distributed tracing enabled (Jaeger, Tempo)

### Post-Deployment

- [ ] **Validation**
  - [ ] Health checks passing
  - [ ] Telemetry ingestion working
  - [ ] Detectors running
  - [ ] Alerts triggering correctly
  - [ ] Integrations functional

- [ ] **Performance**
  - [ ] Latency SLAs met (p50, p95, p99)
  - [ ] Throughput targets achieved
  - [ ] Resource utilization optimal
  - [ ] Auto-scaling working

- [ ] **Operations**
  - [ ] Runbooks created
  - [ ] Oncall rotation defined
  - [ ] Backup/restore tested
  - [ ] Disaster recovery plan documented

---

## Monitoring and Observability

### Key Metrics to Monitor

```promql
# Throughput
rate(sentinel_telemetry_events_total[5m])

# Latency
histogram_quantile(0.99, rate(sentinel_ingestion_latency_seconds_bucket[5m]))
histogram_quantile(0.99, rate(sentinel_detection_latency_seconds_bucket[5m]))

# Error rate
rate(sentinel_ingestion_errors_total[5m])
rate(sentinel_detection_errors_total[5m])

# Anomaly detection
rate(sentinel_anomalies_detected_total[5m])

# Alert delivery
rate(sentinel_alerts_sent_total[5m])
sentinel_alert_failures_total

# Resource utilization
rate(container_cpu_usage_seconds_total{pod=~"sentinel.*"}[5m])
container_memory_working_set_bytes{pod=~"sentinel.*"}

# Queue depth
sentinel_kafka_consumer_lag
sentinel_event_buffer_size
```

### Grafana Dashboard

Import the official dashboard:

```bash
# Download dashboard JSON
curl -o sentinel-dashboard.json \
  https://grafana.com/api/dashboards/12345/revisions/1/download

# Import to Grafana
curl -X POST http://admin:admin@grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @sentinel-dashboard.json
```

---

## Troubleshooting

### Common Issues

**1. High Ingestion Latency**

```bash
# Check ingestion buffer
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli metrics get sentinel_event_buffer_size

# Check network latency
kubectl exec -it deployment/sentinel -n sentinel -- \
  ping observatory.default.svc.cluster.local

# Check resource limits
kubectl top pods -n sentinel
```

**2. Detection Not Working**

```bash
# Check detector configuration
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli detectors list

# Check detector logs
kubectl logs deployment/sentinel -n sentinel | grep "detector"

# Test detector manually
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli detectors test high-latency --event test-event.json
```

**3. Alerts Not Sending**

```bash
# Check alert manager status
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli alerts status

# Test notification channels
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli alerts test slack-incidents

# Check alert logs
kubectl logs deployment/sentinel -n sentinel | grep "alert"
```

**4. Storage Issues**

```bash
# Check storage connectivity
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli storage check

# Check retention policies
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli storage retention show

# Check disk space
kubectl exec -it deployment/sentinel -n sentinel -- df -h
```

### Debug Mode

```bash
# Enable debug logging
kubectl set env deployment/sentinel LOG_LEVEL=debug -n sentinel

# Get diagnostic bundle
kubectl exec -it deployment/sentinel -n sentinel -- \
  sentinel-cli diagnostics collect --output /tmp/diagnostics.tar.gz

# Copy locally
kubectl cp sentinel/sentinel-xxx:/tmp/diagnostics.tar.gz ./diagnostics.tar.gz
```

### Support

For additional help:
- Documentation: https://docs.llm-sentinel.io
- GitHub Issues: https://github.com/llm-sentinel/sentinel/issues
- Community Slack: https://llm-sentinel.slack.com
- Email: support@llm-sentinel.io
