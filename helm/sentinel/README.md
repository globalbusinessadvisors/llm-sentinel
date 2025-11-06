# LLM-Sentinel Helm Chart

Enterprise-grade anomaly detection for LLM applications.

## Prerequisites

- Kubernetes 1.24+
- Helm 3.8+
- PV provisioner support (for persistence)
- Apache Kafka cluster
- InfluxDB v3
- RabbitMQ
- Redis (optional)

## Installing the Chart

To install the chart with the release name `sentinel`:

```bash
# Add required secrets
helm install sentinel ./helm/sentinel \
  --set secrets.influxdbToken="your-token" \
  --set secrets.rabbitmqPassword="your-password"

# Or use a values file
helm install sentinel ./helm/sentinel -f my-values.yaml
```

## Uninstalling the Chart

To uninstall/delete the `sentinel` deployment:

```bash
helm delete sentinel
```

## Configuration

The following table lists the configurable parameters of the Sentinel chart and their default values.

### Basic Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of Sentinel replicas | `3` |
| `image.repository` | Sentinel image repository | `ghcr.io/llm-devops/sentinel` |
| `image.tag` | Sentinel image tag | `0.1.0` |
| `image.pullPolicy` | Image pull policy | `IfNotPresent` |

### Autoscaling

| Parameter | Description | Default |
|-----------|-------------|---------|
| `autoscaling.enabled` | Enable horizontal pod autoscaler | `true` |
| `autoscaling.minReplicas` | Minimum number of replicas | `3` |
| `autoscaling.maxReplicas` | Maximum number of replicas | `10` |
| `autoscaling.targetCPUUtilizationPercentage` | Target CPU utilization | `70` |
| `autoscaling.targetMemoryUtilizationPercentage` | Target memory utilization | `80` |

### Resources

| Parameter | Description | Default |
|-----------|-------------|---------|
| `resources.limits.cpu` | CPU limit | `2000m` |
| `resources.limits.memory` | Memory limit | `2Gi` |
| `resources.requests.cpu` | CPU request | `500m` |
| `resources.requests.memory` | Memory request | `512Mi` |

### Persistence

| Parameter | Description | Default |
|-----------|-------------|---------|
| `persistence.enabled` | Enable persistent volume for baselines | `true` |
| `persistence.storageClassName` | Storage class name | `""` |
| `persistence.accessMode` | Access mode | `ReadWriteMany` |
| `persistence.size` | Volume size | `10Gi` |

### Detection Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.detection.enabledDetectors` | Enabled detection algorithms | `["zscore", "iqr", "mad", "cusum"]` |
| `config.detection.baseline.windowSize` | Baseline window size | `1000` |
| `config.detection.baseline.minSamples` | Minimum samples for baseline | `10` |
| `config.detection.zscore.threshold` | Z-score threshold | `3.0` |

### Storage Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.storage.influxdb.url` | InfluxDB URL | `http://influxdb.influxdb.svc.cluster.local:8086` |
| `config.storage.influxdb.org` | InfluxDB organization | `sentinel` |
| `config.storage.redis.enabled` | Enable Redis caching | `true` |
| `config.storage.redis.url` | Redis URL | `redis://redis-master.redis.svc.cluster.local:6379` |

### Secrets

| Parameter | Description | Default |
|-----------|-------------|---------|
| `secrets.influxdbToken` | InfluxDB authentication token | `""` |
| `secrets.rabbitmqUsername` | RabbitMQ username | `sentinel` |
| `secrets.rabbitmqPassword` | RabbitMQ password | `""` |
| `secrets.redisPassword` | Redis password | `""` |

### External Secrets

| Parameter | Description | Default |
|-----------|-------------|---------|
| `externalSecrets.enabled` | Use external-secrets operator | `false` |
| `externalSecrets.secretStore.name` | Secret store name | `aws-secrets-manager` |

### Monitoring

| Parameter | Description | Default |
|-----------|-------------|---------|
| `serviceMonitor.enabled` | Create Prometheus ServiceMonitor | `false` |
| `serviceMonitor.interval` | Scrape interval | `15s` |

### Security

| Parameter | Description | Default |
|-----------|-------------|---------|
| `podSecurityContext.runAsNonRoot` | Run as non-root user | `true` |
| `podSecurityContext.runAsUser` | User ID | `1000` |
| `securityContext.readOnlyRootFilesystem` | Read-only root filesystem | `true` |
| `networkPolicy.enabled` | Enable network policy | `false` |

## Examples

### Production Deployment

```yaml
# production-values.yaml
replicaCount: 5

autoscaling:
  enabled: true
  minReplicas: 5
  maxReplicas: 20

resources:
  limits:
    cpu: 4000m
    memory: 4Gi
  requests:
    cpu: 1000m
    memory: 1Gi

persistence:
  enabled: true
  storageClassName: fast-ssd
  size: 50Gi

externalSecrets:
  enabled: true
  secretStore:
    name: aws-secrets-manager
    kind: SecretStore

serviceMonitor:
  enabled: true
  interval: 10s

networkPolicy:
  enabled: true

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: sentinel.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: sentinel-tls
      hosts:
        - sentinel.example.com
```

```bash
helm install sentinel ./helm/sentinel -f production-values.yaml
```

### Development Deployment

```yaml
# dev-values.yaml
replicaCount: 1

autoscaling:
  enabled: false

resources:
  limits:
    cpu: 1000m
    memory: 1Gi
  requests:
    cpu: 250m
    memory: 256Mi

persistence:
  enabled: false

config:
  ingestion:
    kafka:
      brokers:
        - localhost:9092
  storage:
    influxdb:
      url: http://localhost:8086
    redis:
      enabled: false
  alerting:
    rabbitmq:
      url: amqp://localhost:5672
```

```bash
helm install sentinel ./helm/sentinel -f dev-values.yaml
```

## Upgrade

To upgrade an existing release:

```bash
helm upgrade sentinel ./helm/sentinel -f my-values.yaml
```

## Verify Installation

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=sentinel

# Check service
kubectl get svc -l app.kubernetes.io/name=sentinel

# View logs
kubectl logs -l app.kubernetes.io/name=sentinel -f

# Port forward and test
kubectl port-forward svc/sentinel 8080:8080
curl http://localhost:8080/health/live
```

## Troubleshooting

### Pods not starting

Check init containers:
```bash
kubectl describe pod <pod-name>
kubectl logs <pod-name> -c wait-for-kafka
kubectl logs <pod-name> -c wait-for-influxdb
```

### Missing secrets

Verify secrets are created:
```bash
kubectl get secret sentinel
kubectl describe secret sentinel
```

### PVC not binding

Check persistent volume claim:
```bash
kubectl get pvc
kubectl describe pvc sentinel
```

### High memory usage

Adjust baseline window size:
```yaml
config:
  detection:
    baseline:
      windowSize: 500  # Reduce from 1000
```

## More Information

- [LLM-Sentinel Documentation](https://github.com/llm-devops/llm-sentinel)
- [Configuration Reference](https://github.com/llm-devops/llm-sentinel/blob/main/docs/configuration.md)
- [Deployment Guide](https://github.com/llm-devops/llm-sentinel/blob/main/DEPLOYMENT.md)
