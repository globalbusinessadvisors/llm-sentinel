# LLM-Sentinel Deployment Guide

This guide covers deploying LLM-Sentinel in various environments from local development to production Kubernetes.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Local Development](#local-development)
3. [Docker Deployment](#docker-deployment)
4. [Kubernetes Deployment](#kubernetes-deployment)
5. [Production Checklist](#production-checklist)
6. [Monitoring & Observability](#monitoring--observability)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Software

- **Rust 1.75+** (for building from source)
- **Docker 24.0+** and Docker Compose v2
- **Kubernetes 1.24+** (for K8s deployment)
- **kubectl 1.24+**
- **Helm 3.0+** (optional)

### Required Services

- **Apache Kafka 3.0+**: Message queue for telemetry
- **InfluxDB v3**: Time-series database
- **RabbitMQ 3.11+**: Alert message broker
- **Redis 7.0+**: Distributed cache (optional)

## Local Development

### 1. Build from Source

```bash
# Clone repository
git clone https://github.com/llm-devops/llm-sentinel.git
cd llm-sentinel

# Build release binary
cargo build --release

# Binary located at: target/release/sentinel
```

### 2. Start Dependencies

```bash
# Start all dependencies using Docker Compose
docker-compose up -d

# Verify services are running
docker-compose ps

# View logs
docker-compose logs -f
```

### 3. Configure Sentinel

```bash
# Copy example configuration
cp config/sentinel.yaml config/sentinel-local.yaml

# Edit configuration
vim config/sentinel-local.yaml
```

**Key configuration changes for local development:**

```yaml
ingestion:
  kafka:
    brokers: ["localhost:9093"]  # Use host port

storage:
  influxdb:
    url: "http://localhost:8086"
    token: "sentinel-token-123456789"  # From docker-compose

alerting:
  rabbitmq:
    url: "amqp://admin:adminpass@localhost:5672"

  redis:
    enabled: true
    url: "redis://:redispass@localhost:6379"
```

### 4. Run Sentinel

```bash
# Run with custom config
./target/release/sentinel --config config/sentinel-local.yaml

# Or with environment variables
export INFLUXDB_TOKEN=sentinel-token-123456789
export SENTINEL_LOG_LEVEL=debug
./target/release/sentinel --config config/sentinel-local.yaml
```

### 5. Verify Deployment

```bash
# Check health
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8080/metrics

# Query telemetry
curl "http://localhost:8080/api/v1/telemetry?hours=1"
```

### 6. Access UIs

- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Kafka UI**: http://localhost:8081
- **RabbitMQ Management**: http://localhost:15672 (admin/adminpass)
- **InfluxDB**: http://localhost:8086

## Docker Deployment

### 1. Build Docker Image

```bash
# Build using Dockerfile
docker build -t sentinel:latest .

# Build with specific tag
docker build -t sentinel:0.1.0 .

# Build for multiple platforms
docker buildx build --platform linux/amd64,linux/arm64 -t sentinel:0.1.0 .
```

### 2. Run Container

```bash
# Run with docker-compose (recommended)
docker-compose up -d sentinel

# Run standalone container
docker run -d \
  --name sentinel \
  -p 8080:8080 \
  -p 9090:9090 \
  -e SENTINEL_LOG_LEVEL=info \
  -e INFLUXDB_TOKEN=your-token \
  -v $(pwd)/config/sentinel.yaml:/etc/sentinel/sentinel.yaml \
  -v sentinel-baselines:/var/lib/sentinel/baselines \
  sentinel:latest
```

### 3. View Logs

```bash
# Follow logs
docker logs -f sentinel

# Last 100 lines
docker logs --tail 100 sentinel
```

### 4. Stop and Cleanup

```bash
# Stop services
docker-compose down

# Remove volumes (WARNING: deletes data)
docker-compose down -v

# Remove images
docker rmi sentinel:latest
```

## Kubernetes Deployment

### Prerequisites

- Running Kubernetes cluster (v1.24+)
- kubectl configured
- Cluster has:
  - Kafka (Strimzi operator recommended)
  - InfluxDB
  - RabbitMQ
  - Redis (optional)
  - Prometheus Operator (for ServiceMonitor)
  - Cert-manager (for TLS)

### 1. Install Prerequisites

```bash
# Install Kafka with Strimzi
kubectl create namespace kafka
kubectl create -f 'https://strimzi.io/install/latest?namespace=kafka' -n kafka
kubectl apply -f k8s/kafka-cluster.yaml -n kafka

# Install InfluxDB
helm repo add influxdata https://helm.influxdata.com/
helm install influxdb influxdata/influxdb2 -n influxdb --create-namespace

# Install RabbitMQ
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install rabbitmq bitnami/rabbitmq -n rabbitmq --create-namespace

# Install Redis
helm install redis bitnami/redis -n redis --create-namespace
```

### 2. Prepare Secrets

```bash
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Create secrets
kubectl create secret generic sentinel-secrets \
  --from-literal=influxdb-token="your-influxdb-token" \
  --from-literal=rabbitmq-username="sentinel" \
  --from-literal=rabbitmq-password="your-password" \
  --from-literal=redis-password="your-redis-pass" \
  --from-literal=webhook-secret="your-webhook-secret" \
  -n sentinel

# Or apply from file
kubectl apply -f k8s/secret.yaml
```

### 3. Deploy Sentinel

#### Option A: Using kubectl

```bash
# Apply all manifests
kubectl apply -f k8s/

# Or apply in order
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/serviceaccount.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml
kubectl apply -f k8s/pvc.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/hpa.yaml
kubectl apply -f k8s/pdb.yaml
kubectl apply -f k8s/networkpolicy.yaml
kubectl apply -f k8s/servicemonitor.yaml
kubectl apply -f k8s/ingress.yaml
```

#### Option B: Using Kustomize

```bash
# Apply with kustomize
kubectl apply -k k8s/

# Preview changes
kubectl diff -k k8s/
```

### 4. Verify Deployment

```bash
# Check pods
kubectl get pods -n sentinel

# Check deployment status
kubectl rollout status deployment/sentinel -n sentinel

# Check services
kubectl get svc -n sentinel

# View logs
kubectl logs -f deployment/sentinel -n sentinel

# Check events
kubectl get events -n sentinel --sort-by='.lastTimestamp'
```

### 5. Access Application

```bash
# Port forward for local access
kubectl port-forward svc/sentinel 8080:8080 -n sentinel

# Access via Ingress (after DNS setup)
curl https://sentinel.example.com/health

# Get ingress IP
kubectl get ingress sentinel -n sentinel
```

### 6. Scale Deployment

```bash
# Manual scaling
kubectl scale deployment/sentinel --replicas=5 -n sentinel

# Check HPA status
kubectl get hpa -n sentinel

# View HPA metrics
kubectl describe hpa sentinel -n sentinel
```

### 7. Update Deployment

```bash
# Update image
kubectl set image deployment/sentinel \
  sentinel=ghcr.io/llm-devops/sentinel:0.2.0 \
  -n sentinel

# Watch rollout
kubectl rollout status deployment/sentinel -n sentinel

# Rollback if needed
kubectl rollout undo deployment/sentinel -n sentinel

# Check rollout history
kubectl rollout history deployment/sentinel -n sentinel
```

## Production Checklist

### Security

- [ ] Use TLS/SSL for all external connections
- [ ] Enable network policies
- [ ] Use secrets management (Vault, AWS Secrets Manager)
- [ ] Run containers as non-root user
- [ ] Enable Pod Security Standards
- [ ] Use read-only root filesystem
- [ ] Scan images for vulnerabilities
- [ ] Enable RBAC with minimal permissions
- [ ] Use private container registries
- [ ] Rotate credentials regularly

### High Availability

- [ ] Run at least 3 replicas
- [ ] Configure Pod Disruption Budget (min 2 available)
- [ ] Use Pod anti-affinity rules
- [ ] Configure liveness and readiness probes
- [ ] Set appropriate resource limits
- [ ] Use topology spread constraints
- [ ] Configure automatic scaling (HPA)
- [ ] Set up multi-zone deployment

### Monitoring & Observability

- [ ] Install Prometheus and Grafana
- [ ] Configure ServiceMonitor
- [ ] Set up alerting rules
- [ ] Enable structured logging (JSON)
- [ ] Configure log aggregation (ELK, Loki)
- [ ] Set up distributed tracing
- [ ] Monitor resource usage
- [ ] Track SLIs and SLOs
- [ ] Configure dashboards

### Performance

- [ ] Tune resource requests/limits
- [ ] Enable connection pooling
- [ ] Configure batch sizes appropriately
- [ ] Use caching (Redis)
- [ ] Optimize baseline window sizes
- [ ] Monitor and tune GC settings
- [ ] Use appropriate storage classes
- [ ] Enable compression for storage

### Backup & Recovery

- [ ] Back up InfluxDB data regularly
- [ ] Back up baseline data
- [ ] Back up configuration
- [ ] Test restore procedures
- [ ] Document recovery procedures
- [ ] Set up DR environment

### Configuration

- [ ] Use environment-specific configs
- [ ] Externalize all secrets
- [ ] Configure proper log levels
- [ ] Set appropriate timeouts
- [ ] Configure retry policies
- [ ] Tune detection thresholds
- [ ] Set resource quotas
- [ ] Configure network policies

## Monitoring & Observability

### Prometheus Metrics

Key metrics to monitor:

```promql
# Ingestion rate
rate(sentinel_events_processed_total[5m])

# Anomaly detection rate
rate(sentinel_anomalies_detected_total[5m])

# Error rates
rate(sentinel_ingestion_errors_total[5m])
rate(sentinel_detection_errors_total[5m])
rate(sentinel_storage_errors_total[5m])

# Latencies
histogram_quantile(0.95, sentinel_detection_latency_seconds)

# Cache performance
sentinel_cache_hits_total / (sentinel_cache_hits_total + sentinel_cache_misses_total)

# Resource usage
process_resident_memory_bytes
process_cpu_seconds_total
```

### Grafana Dashboards

Import pre-built dashboards:

```bash
# Import from file
kubectl create configmap sentinel-dashboard \
  --from-file=dashboard.json \
  -n monitoring
```

### Alerting Rules

Example Prometheus alerts:

```yaml
groups:
  - name: sentinel
    rules:
      - alert: SentinelHighErrorRate
        expr: rate(sentinel_ingestion_errors_total[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate in Sentinel"

      - alert: SentinelDown
        expr: up{job="sentinel"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Sentinel is down"
```

## Troubleshooting

### Common Issues

#### 1. Pods Not Starting

```bash
# Check pod status
kubectl describe pod <pod-name> -n sentinel

# Common causes:
# - Image pull errors: Check image name and registry credentials
# - Resource constraints: Check node resources
# - Volume mount issues: Verify PVC exists
# - Init container failures: Check dependencies
```

#### 2. Connection Issues

```bash
# Test Kafka connectivity
kubectl run kafka-test --rm -it --image=confluentinc/cp-kafka:7.5.0 -- \
  kafka-broker-api-versions --bootstrap-server kafka:9092

# Test InfluxDB connectivity
kubectl run influx-test --rm -it --image=influxdb:2.7 -- \
  influx ping --host http://influxdb:8086

# Test RabbitMQ connectivity
kubectl run rabbit-test --rm -it --image=rabbitmq:3.12 -- \
  rabbitmq-diagnostics -n rabbitmq ping
```

#### 3. High Memory Usage

```bash
# Check memory usage
kubectl top pods -n sentinel

# Increase memory limits
kubectl set resources deployment/sentinel \
  --limits=memory=4Gi \
  -n sentinel

# Check for memory leaks in logs
kubectl logs deployment/sentinel -n sentinel | grep -i "memory\|oom"
```

#### 4. Slow Detection

```bash
# Check detection latency metrics
kubectl port-forward svc/sentinel 8080:8080 -n sentinel
curl localhost:8080/metrics | grep sentinel_detection_latency

# Possible solutions:
# - Reduce baseline window size
# - Disable slow detectors
# - Increase CPU limits
# - Scale horizontally
```

### Debug Mode

Enable debug logging:

```bash
# Update deployment
kubectl set env deployment/sentinel \
  SENTINEL_LOG_LEVEL=debug \
  RUST_BACKTRACE=1 \
  -n sentinel

# Follow logs
kubectl logs -f deployment/sentinel -n sentinel
```

### Support

- **Documentation**: [README.md](README.md)
- **Issues**: [GitHub Issues](https://github.com/llm-devops/llm-sentinel/issues)
- **Discussions**: [GitHub Discussions](https://github.com/llm-devops/llm-sentinel/discussions)

---

**Last Updated**: 2025-11-06
**Version**: 0.1.0
