# LLM-Sentinel Integration Examples

## Table of Contents

1. [Observatory Integration](#observatory-integration)
2. [Shield Integration](#shield-integration)
3. [Incident Manager Integration](#incident-manager-integration)
4. [Governance Dashboard Integration](#governance-dashboard-integration)
5. [Custom Integrations](#custom-integrations)
6. [End-to-End Scenarios](#end-to-end-scenarios)

---

## Observatory Integration

### gRPC Streaming Integration

**Observatory Client Implementation (Go):**

```go
package main

import (
    "context"
    "log"
    "time"

    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    sentinelpb "github.com/llm-sentinel/api/proto/v1"
)

type ObservatoryExporter struct {
    client sentinelpb.TelemetryIngestionClient
    stream sentinelpb.TelemetryIngestion_StreamTelemetryClient
}

func NewObservatoryExporter(sentinelEndpoint string) (*ObservatoryExporter, error) {
    conn, err := grpc.Dial(sentinelEndpoint,
        grpc.WithTransportCredentials(insecure.NewCredentials()),
        grpc.WithDefaultCallOptions(grpc.MaxCallRecvMsgSize(10*1024*1024)),
    )
    if err != nil {
        return nil, err
    }

    client := sentinelpb.NewTelemetryIngestionClient(conn)
    stream, err := client.StreamTelemetry(context.Background())
    if err != nil {
        return nil, err
    }

    return &ObservatoryExporter{
        client: client,
        stream: stream,
    }, nil
}

func (e *ObservatoryExporter) ExportEvent(event *TelemetryEvent) error {
    pbEvent := &sentinelpb.TelemetryEvent{
        EventId:       event.ID,
        Timestamp:     timestamppb.New(event.Timestamp),
        EventType:     event.Type,
        ApplicationId: event.ApplicationID,
        TenantId:      event.TenantID,
    }

    // Set payload based on type
    switch event.Type {
    case "llm.request":
        pbEvent.Payload = &sentinelpb.TelemetryEvent_Request{
            Request: &sentinelpb.RequestData{
                Model:        event.Data.Model,
                Prompt:       event.Data.Prompt,
                PromptTokens: int32(event.Data.PromptTokens),
            },
        }
    case "llm.response":
        pbEvent.Payload = &sentinelpb.TelemetryEvent_Response{
            Response: &sentinelpb.ResponseData{
                Status:           event.Data.Status,
                CompletionTokens: int32(event.Data.CompletionTokens),
                TotalTokens:      int32(event.Data.TotalTokens),
                LatencyMs:        event.Data.LatencyMs,
                Cost:             event.Data.Cost,
            },
        }
    }

    return e.stream.Send(pbEvent)
}

func (e *ObservatoryExporter) Close() error {
    _, err := e.stream.CloseAndRecv()
    return err
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
      client_cert: /etc/certs/client.crt
      client_key: /etc/certs/client.key

    batching:
      enabled: true
      size: 100
      timeout: 1s

    retry:
      enabled: true
      max_attempts: 3
      initial_backoff: 100ms
      max_backoff: 5s
      backoff_multiplier: 2

    compression: gzip

    # Filter what to send
    filters:
      include:
        event_types:
          - llm.request
          - llm.response
          - llm.error
        applications:
          - chatbot-prod
          - summarizer-prod

      exclude:
        metadata:
          internal: "true"
          test: "true"

    # Sampling (send only X% of events)
    sampling:
      enabled: true
      rate: 1.0  # 100% (no sampling)
      # For high-volume apps, use adaptive sampling
      adaptive:
        enabled: false
        target_rate: 10000  # events/sec
```

### HTTP REST Integration

**Observatory HTTP Client (Python):**

```python
import requests
import json
from typing import Dict, Any
from datetime import datetime
import uuid

class SentinelExporter:
    def __init__(self, sentinel_url: str, api_key: str):
        self.sentinel_url = sentinel_url.rstrip('/')
        self.api_key = api_key
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {api_key}'
        })

    def export_event(self, event_data: Dict[str, Any]) -> bool:
        """Export a single telemetry event"""
        endpoint = f"{self.sentinel_url}/api/v1/telemetry/events"

        event = {
            "event_id": str(uuid.uuid4()),
            "timestamp": datetime.utcnow().isoformat() + "Z",
            "source": {
                "service": "llm-observatory",
                "host": "obs-worker-01"
            },
            **event_data
        }

        try:
            response = self.session.post(endpoint, json=event, timeout=5)
            response.raise_for_status()
            return True
        except requests.exceptions.RequestException as e:
            print(f"Failed to export event: {e}")
            return False

    def export_batch(self, events: list) -> bool:
        """Export multiple events in batch"""
        endpoint = f"{self.sentinel_url}/api/v1/telemetry/batch"

        try:
            response = self.session.post(
                endpoint,
                json={"events": events},
                timeout=10
            )
            response.raise_for_status()
            return True
        except requests.exceptions.RequestException as e:
            print(f"Failed to export batch: {e}")
            return False

# Usage example
exporter = SentinelExporter(
    sentinel_url="https://sentinel.example.com",
    api_key="your-api-key"
)

# Export LLM request/response
exporter.export_event({
    "event_type": "llm.response",
    "application_id": "chatbot-prod",
    "tenant_id": "acme-corp",
    "data": {
        "status": "success",
        "completion_tokens": 75,
        "total_tokens": 225,
        "latency_ms": 1234.56,
        "cost": 0.045,
        "finish_reason": "stop"
    },
    "metadata": {
        "user_id": "user-123",
        "session_id": "sess-456"
    }
})
```

### Kafka Integration

**Observatory Kafka Producer:**

```java
// ObservatorySentinelProducer.java
import org.apache.kafka.clients.producer.*;
import org.apache.kafka.common.serialization.StringSerializer;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.util.Properties;

public class ObservatorySentinelProducer {
    private final KafkaProducer<String, String> producer;
    private final ObjectMapper objectMapper;
    private final String topic;

    public ObservatorySentinelProducer(String bootstrapServers, String topic) {
        this.topic = topic;
        this.objectMapper = new ObjectMapper();

        Properties props = new Properties();
        props.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, bootstrapServers);
        props.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
        props.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
        props.put(ProducerConfig.ACKS_CONFIG, "all");
        props.put(ProducerConfig.RETRIES_CONFIG, 3);
        props.put(ProducerConfig.COMPRESSION_TYPE_CONFIG, "gzip");
        props.put(ProducerConfig.BATCH_SIZE_CONFIG, 16384);
        props.put(ProducerConfig.LINGER_MS_CONFIG, 10);

        this.producer = new KafkaProducer<>(props);
    }

    public void sendTelemetryEvent(TelemetryEvent event) {
        try {
            String key = event.getApplicationId(); // Partition by application
            String value = objectMapper.writeValueAsString(event);

            ProducerRecord<String, String> record =
                new ProducerRecord<>(topic, key, value);

            producer.send(record, (metadata, exception) -> {
                if (exception != null) {
                    System.err.println("Failed to send event: " + exception);
                } else {
                    System.out.println("Event sent to partition " +
                        metadata.partition() + " offset " + metadata.offset());
                }
            });
        } catch (Exception e) {
            System.err.println("Failed to serialize event: " + e);
        }
    }

    public void close() {
        producer.flush();
        producer.close();
    }
}

// Usage
ObservatorySentinelProducer producer = new ObservatorySentinelProducer(
    "kafka-1:9092,kafka-2:9092",
    "llm-telemetry"
);

TelemetryEvent event = new TelemetryEvent()
    .setEventId(UUID.randomUUID().toString())
    .setTimestamp(Instant.now())
    .setEventType("llm.response")
    .setApplicationId("chatbot-prod")
    .setData(responseData);

producer.sendTelemetryEvent(event);
```

**Sentinel Kafka Consumer Configuration:**

```yaml
# sentinel-config.yaml
ingestion:
  kafka:
    enabled: true
    brokers:
      - kafka-1.example.com:9092
      - kafka-2.example.com:9092

    topics:
      - llm-telemetry
      - llm-metrics

    consumerGroup: sentinel-prod

    # Consumer tuning
    consumer:
      sessionTimeoutMs: 30000
      heartbeatIntervalMs: 3000
      maxPollRecords: 500
      maxPollIntervalMs: 300000
      autoOffsetReset: latest
      enableAutoCommit: false

    # Security
    sasl:
      enabled: true
      mechanism: SCRAM-SHA-512
      username: sentinel
      passwordSecret: kafka-password

    tls:
      enabled: true
      caFile: /etc/sentinel/certs/kafka-ca.crt

    # Performance
    compression: gzip
    fetchMinBytes: 1
    fetchMaxWaitMs: 500
```

---

## Shield Integration

### Real-time Action Enforcement

**Sentinel → Shield gRPC Client:**

```go
package shield

import (
    "context"
    "time"

    "google.golang.org/grpc"
    shieldpb "github.com/llm-shield/api/proto/v1"
)

type ShieldActionClient struct {
    client shieldpb.ShieldActionClient
}

func NewShieldActionClient(endpoint string) (*ShieldActionClient, error) {
    conn, err := grpc.Dial(endpoint, grpc.WithInsecure())
    if err != nil {
        return nil, err
    }

    return &ShieldActionClient{
        client: shieldpb.NewShieldActionClient(conn),
    }, nil
}

func (c *ShieldActionClient) BlockRequest(
    ctx context.Context,
    anomalyID string,
    targetEntity string,
    reason string,
    duration time.Duration,
) error {
    req := &shieldpb.ActionRequest{
        ActionId:     anomalyID,
        Type:         shieldpb.ActionType_BLOCK,
        TargetEntity: targetEntity,
        Reason:       reason,
        Duration:     durationpb.New(duration),
        Parameters: map[string]string{
            "source": "sentinel",
            "severity": "critical",
        },
    }

    resp, err := c.client.EnforceAction(ctx, req)
    if err != nil {
        return err
    }

    if !resp.Success {
        return fmt.Errorf("shield action failed: %s", resp.Message)
    }

    return nil
}

func (c *ShieldActionClient) RateLimit(
    ctx context.Context,
    anomalyID string,
    targetEntity string,
    limit int,
    window time.Duration,
) error {
    req := &shieldpb.ActionRequest{
        ActionId:     anomalyID,
        Type:         shieldpb.ActionType_RATE_LIMIT,
        TargetEntity: targetEntity,
        Reason:       "Anomalous behavior detected",
        Duration:     durationpb.New(window),
        Parameters: map[string]string{
            "limit": fmt.Sprintf("%d", limit),
            "window": window.String(),
        },
    }

    resp, err := c.client.EnforceAction(ctx, req)
    if err != nil {
        return err
    }

    if !resp.Success {
        return fmt.Errorf("shield rate limit failed: %s", resp.Message)
    }

    return nil
}
```

**Sentinel Alert Action Configuration:**

```yaml
# detector-with-shield-action.yaml
detectors:
  - id: prompt-injection-ml
    name: Prompt Injection Detector (ML)
    type: ml
    enabled: true
    config:
      model: prompt-injection-v2
      threshold: 0.85
      features: ["prompt_text", "embedding"]

    actions:
      # Block the request immediately
      - type: shield
        config:
          endpoint: shield.example.com:9090
          action: block
          target: request
          duration: 1h
          reason: "Prompt injection attack detected"
          parameters:
            block_type: immediate
            notify_user: true

      # Alert security team
      - type: alert
        config:
          severity: critical
          receivers: ["security-team", "pagerduty-security"]

  - id: abnormal-cost-spike
    name: Abnormal Cost Spike Detector
    type: statistical
    enabled: true
    config:
      metric: cost_per_minute
      strategy: zscore
      threshold: 4.0
      window: 5m

    actions:
      # Rate limit instead of block
      - type: shield
        config:
          endpoint: shield.example.com:9090
          action: rate_limit
          target: application
          duration: 10m
          parameters:
            limit: 100
            window: 1m

      # Alert operations
      - type: alert
        config:
          severity: warning
          receivers: ["ops-team"]
```

### Shield Webhook Callbacks

**Sentinel Webhook Endpoint for Shield Status:**

```go
// Receive status updates from Shield
func (s *SentinelServer) HandleShieldCallback(w http.ResponseWriter, r *http.Request) {
    var callback ShieldCallback
    if err := json.NewDecoder(r.Body).Decode(&callback); err != nil {
        http.Error(w, "Invalid request", http.StatusBadRequest)
        return
    }

    // Update anomaly with Shield action result
    err := s.anomalyStore.UpdateActionStatus(
        callback.AnomalyID,
        callback.ActionID,
        callback.Status,
        callback.Message,
    )
    if err != nil {
        http.Error(w, "Failed to update", http.StatusInternalServerError)
        return
    }

    // If action failed, trigger escalation
    if callback.Status == "failed" {
        s.alertManager.Escalate(callback.AnomalyID, "shield_action_failed")
    }

    w.WriteHeader(http.StatusOK)
}

type ShieldCallback struct {
    AnomalyID string `json:"anomaly_id"`
    ActionID  string `json:"action_id"`
    Status    string `json:"status"`
    Message   string `json:"message"`
    Timestamp string `json:"timestamp"`
}
```

---

## Incident Manager Integration

### Automated Incident Creation

**Sentinel Incident Manager Client (Node.js):**

```javascript
// incident-manager-client.js
const axios = require('axios');

class IncidentManagerClient {
  constructor(baseURL, apiKey) {
    this.client = axios.create({
      baseURL,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${apiKey}`
      },
      timeout: 10000
    });
  }

  async createIncident(anomaly) {
    const incident = {
      source: 'llm-sentinel',
      incident_type: 'anomaly_detected',
      severity: this.mapSeverity(anomaly.severity),
      title: anomaly.title,
      description: this.buildDescription(anomaly),
      metadata: {
        anomaly_id: anomaly.anomaly_id,
        detector_id: anomaly.detector_id,
        application_id: anomaly.application_id,
        score: anomaly.score,
        timestamp: anomaly.timestamp
      },
      affected_entities: [{
        type: anomaly.affected_entity.type,
        id: anomaly.affected_entity.id,
        name: anomaly.affected_entity.name
      }],
      remediation_actions: this.getRemediationSteps(anomaly)
    };

    try {
      const response = await this.client.post('/api/v1/incidents', incident);
      return response.data.incident_id;
    } catch (error) {
      console.error('Failed to create incident:', error);
      throw error;
    }
  }

  async updateIncident(incidentID, status, resolution) {
    try {
      await this.client.patch(`/api/v1/incidents/${incidentID}`, {
        status,
        resolution,
        resolved_at: new Date().toISOString()
      });
    } catch (error) {
      console.error('Failed to update incident:', error);
      throw error;
    }
  }

  async addComment(incidentID, comment) {
    try {
      await this.client.post(`/api/v1/incidents/${incidentID}/comments`, {
        author: 'llm-sentinel',
        text: comment,
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      console.error('Failed to add comment:', error);
      throw error;
    }
  }

  mapSeverity(sentinelSeverity) {
    const mapping = {
      'critical': 'P1',
      'warning': 'P2',
      'info': 'P3'
    };
    return mapping[sentinelSeverity] || 'P3';
  }

  buildDescription(anomaly) {
    return `
## Anomaly Details

**Detector:** ${anomaly.detector_id}
**Application:** ${anomaly.application_id}
**Confidence Score:** ${(anomaly.score * 100).toFixed(1)}%
**Detected At:** ${anomaly.timestamp}

## Description

${anomaly.description}

## Metrics

- **Baseline Value:** ${anomaly.metrics.baseline_value}
- **Observed Value:** ${anomaly.metrics.observed_value}
- **Deviation:** ${anomaly.metrics.deviation}σ
- **Threshold:** ${anomaly.metrics.threshold}σ

## Context

- **Window:** ${anomaly.context.window_start} to ${anomaly.context.window_end}
- **Sample Events:** ${anomaly.context.sample_events.join(', ')}

## Actions Taken

${anomaly.actions_taken.map(a => `- ${a.action}: ${a.status}`).join('\n')}
    `.trim();
  }

  getRemediationSteps(anomaly) {
    const steps = [
      `Review anomaly details in Sentinel: https://sentinel.example.com/anomalies/${anomaly.anomaly_id}`,
      `Investigate sample events for patterns`,
      `Check related anomalies for correlation`,
    ];

    // Add detector-specific steps
    if (anomaly.detector_id === 'prompt-injection-ml') {
      steps.push(
        'Review blocked requests for false positives',
        'Update ML model if needed',
        'Consider adjusting detection threshold'
      );
    } else if (anomaly.detector_id === 'token-rate-spike') {
      steps.push(
        'Check for legitimate traffic spikes (marketing campaigns, etc.)',
        'Review rate limiting policies',
        'Investigate potential abuse or bot activity'
      );
    }

    return steps;
  }
}

module.exports = IncidentManagerClient;
```

**Usage in Sentinel:**

```javascript
const IncidentManagerClient = require('./incident-manager-client');

const incidentClient = new IncidentManagerClient(
  'https://incident-manager.example.com',
  process.env.INCIDENT_MANAGER_API_KEY
);

// On critical anomaly detection
async function handleCriticalAnomaly(anomaly) {
  if (anomaly.severity === 'critical' && anomaly.score >= 0.9) {
    try {
      const incidentID = await incidentClient.createIncident(anomaly);
      console.log(`Incident created: ${incidentID}`);

      // Store incident ID with anomaly
      await anomalyStore.update(anomaly.anomaly_id, {
        incident_id: incidentID
      });
    } catch (error) {
      console.error('Failed to create incident:', error);
    }
  }
}

// On anomaly resolution
async function handleAnomalyResolution(anomaly, resolution) {
  if (anomaly.incident_id) {
    try {
      await incidentClient.updateIncident(
        anomaly.incident_id,
        'resolved',
        resolution
      );
    } catch (error) {
      console.error('Failed to resolve incident:', error);
    }
  }
}
```

---

## Governance Dashboard Integration

### GraphQL API Integration

**Sentinel GraphQL Schema:**

```graphql
# schema.graphql
type Query {
  """Get anomalies with filtering"""
  anomalies(
    filter: AnomalyFilter
    timeRange: TimeRange!
    pagination: Pagination
  ): AnomalyConnection!

  """Get specific anomaly by ID"""
  anomaly(id: ID!): Anomaly

  """Get metrics time series"""
  metrics(
    names: [String!]!
    dimensions: [DimensionInput!]
    timeRange: TimeRange!
    aggregation: Aggregation!
    granularity: Granularity!
  ): [MetricSeries!]!

  """Get statistics summary"""
  statistics(
    timeRange: TimeRange!
    groupBy: [String!]
  ): Statistics!

  """Get detector configuration and status"""
  detectors(
    filter: DetectorFilter
  ): [Detector!]!

  """Get alert history"""
  alerts(
    filter: AlertFilter
    timeRange: TimeRange!
  ): AlertConnection!
}

type Mutation {
  """Acknowledge an anomaly"""
  acknowledgeAnomaly(
    id: ID!
    acknowledgedBy: String!
    note: String
  ): Anomaly!

  """Update detector configuration"""
  updateDetector(
    id: ID!
    config: DetectorConfigInput!
  ): Detector!

  """Test a detector"""
  testDetector(
    id: ID!
    testData: JSON!
  ): DetectorTestResult!
}

type Subscription {
  """Subscribe to new anomalies"""
  anomalyDetected(
    filter: AnomalyFilter
  ): Anomaly!

  """Subscribe to metric updates"""
  metricsUpdated(
    names: [String!]!
  ): MetricUpdate!
}

type Anomaly {
  id: ID!
  timestamp: DateTime!
  detectorId: String!
  detectorType: DetectorType!
  severity: Severity!
  score: Float!
  status: AnomalyStatus!
  title: String!
  description: String!
  affectedEntity: Entity!
  metrics: AnomalyMetrics!
  context: AnomalyContext!
  actionsTaken: [ActionTaken!]!
  relatedAnomalies: [Anomaly!]!
  incidentId: String
  acknowledgedBy: String
  acknowledgedAt: DateTime
}

type MetricSeries {
  name: String!
  dimensions: [Dimension!]!
  datapoints: [Datapoint!]!
  unit: String
  aggregation: Aggregation!
}

type Statistics {
  totalAnomalies: Int!
  anomaliesByDetector: [DetectorStats!]!
  anomaliesBySeverity: [SeverityStats!]!
  anomaliesByApplication: [ApplicationStats!]!
  topAffectedEntities: [EntityStats!]!
  detectionRate: Float!
  averageScore: Float!
  mttr: Float  # Mean Time To Resolution
}

input AnomalyFilter {
  detectorIds: [String!]
  severities: [Severity!]
  statuses: [AnomalyStatus!]
  applicationIds: [String!]
  scoreMin: Float
  scoreMax: Float
}

input TimeRange {
  start: DateTime!
  end: DateTime!
}

input Pagination {
  limit: Int = 100
  offset: Int = 0
}

enum Severity {
  INFO
  WARNING
  CRITICAL
}

enum AnomalyStatus {
  ACTIVE
  ACKNOWLEDGED
  RESOLVED
  SUPPRESSED
}

enum DetectorType {
  STATISTICAL
  RULE
  ML
  BEHAVIORAL
}
```

**Dashboard Client Example (React):**

```typescript
// hooks/useSentinelData.ts
import { useQuery, useMutation, useSubscription } from '@apollo/client';
import { gql } from '@apollo/client';

const GET_ANOMALIES = gql`
  query GetAnomalies($filter: AnomalyFilter, $timeRange: TimeRange!, $pagination: Pagination) {
    anomalies(filter: $filter, timeRange: $timeRange, pagination: $pagination) {
      edges {
        node {
          id
          timestamp
          detectorId
          severity
          score
          status
          title
          description
          affectedEntity {
            type
            id
            name
          }
        }
      }
      pageInfo {
        hasNextPage
        endCursor
      }
      totalCount
    }
  }
`;

const GET_METRICS = gql`
  query GetMetrics(
    $names: [String!]!
    $timeRange: TimeRange!
    $aggregation: Aggregation!
    $granularity: Granularity!
  ) {
    metrics(
      names: $names
      timeRange: $timeRange
      aggregation: $aggregation
      granularity: $granularity
    ) {
      name
      datapoints {
        timestamp
        value
      }
      unit
    }
  }
`;

const ANOMALY_SUBSCRIPTION = gql`
  subscription OnAnomalyDetected($filter: AnomalyFilter) {
    anomalyDetected(filter: $filter) {
      id
      timestamp
      severity
      title
      score
    }
  }
`;

const ACKNOWLEDGE_ANOMALY = gql`
  mutation AcknowledgeAnomaly($id: ID!, $acknowledgedBy: String!, $note: String) {
    acknowledgeAnomaly(id: $id, acknowledgedBy: $acknowledgedBy, note: $note) {
      id
      status
      acknowledgedBy
      acknowledgedAt
    }
  }
`;

export function useSentinelAnomalies(filter, timeRange) {
  const { data, loading, error, fetchMore } = useQuery(GET_ANOMALIES, {
    variables: { filter, timeRange, pagination: { limit: 50 } },
    pollInterval: 30000, // Poll every 30 seconds
  });

  return {
    anomalies: data?.anomalies?.edges?.map(edge => edge.node) || [],
    totalCount: data?.anomalies?.totalCount || 0,
    loading,
    error,
    loadMore: () => {
      if (data?.anomalies?.pageInfo?.hasNextPage) {
        fetchMore({
          variables: {
            pagination: {
              limit: 50,
              offset: data.anomalies.edges.length,
            },
          },
        });
      }
    },
  };
}

export function useSentinelMetrics(metricNames, timeRange, aggregation, granularity) {
  const { data, loading, error } = useQuery(GET_METRICS, {
    variables: {
      names: metricNames,
      timeRange,
      aggregation,
      granularity,
    },
    pollInterval: 15000, // Poll every 15 seconds
  });

  return {
    metrics: data?.metrics || [],
    loading,
    error,
  };
}

export function useAnomalySubscription(filter) {
  const { data } = useSubscription(ANOMALY_SUBSCRIPTION, {
    variables: { filter },
  });

  return data?.anomalyDetected;
}

export function useAcknowledgeAnomaly() {
  const [acknowledgeAnomaly, { loading }] = useMutation(ACKNOWLEDGE_ANOMALY);

  return {
    acknowledgeAnomaly,
    loading,
  };
}
```

**Dashboard Component Example:**

```tsx
// components/AnomalyDashboard.tsx
import React, { useState } from 'react';
import {
  useSentinelAnomalies,
  useSentinelMetrics,
  useAnomalySubscription,
  useAcknowledgeAnomaly,
} from '../hooks/useSentinelData';

export function AnomalyDashboard() {
  const [timeRange, setTimeRange] = useState({
    start: new Date(Date.now() - 24 * 60 * 60 * 1000), // Last 24 hours
    end: new Date(),
  });

  const [filter, setFilter] = useState({
    severities: ['CRITICAL', 'WARNING'],
  });

  const { anomalies, totalCount, loading } = useSentinelAnomalies(filter, timeRange);

  const { metrics } = useSentinelMetrics(
    ['anomaly_rate', 'detection_latency'],
    timeRange,
    'AVG',
    '5m'
  );

  const newAnomaly = useAnomalySubscription(filter);
  const { acknowledgeAnomaly } = useAcknowledgeAnomaly();

  // Show notification for new anomaly
  React.useEffect(() => {
    if (newAnomaly) {
      showNotification(`New ${newAnomaly.severity} anomaly: ${newAnomaly.title}`);
    }
  }, [newAnomaly]);

  const handleAcknowledge = async (anomalyId: string) => {
    await acknowledgeAnomaly({
      variables: {
        id: anomalyId,
        acknowledgedBy: 'current-user@example.com',
      },
    });
  };

  return (
    <div className="anomaly-dashboard">
      <header>
        <h1>LLM Sentinel - Anomaly Dashboard</h1>
        <div className="metrics-summary">
          <MetricCard title="Total Anomalies" value={totalCount} />
          <MetricCard title="Critical" value={anomalies.filter(a => a.severity === 'CRITICAL').length} />
          <MetricCard title="Active" value={anomalies.filter(a => a.status === 'ACTIVE').length} />
        </div>
      </header>

      <section className="charts">
        <TimeSeriesChart
          title="Anomaly Detection Rate"
          data={metrics.find(m => m.name === 'anomaly_rate')}
        />
        <TimeSeriesChart
          title="Detection Latency (p99)"
          data={metrics.find(m => m.name === 'detection_latency')}
        />
      </section>

      <section className="anomaly-list">
        <h2>Recent Anomalies</h2>
        {loading ? (
          <Loading />
        ) : (
          <AnomalyTable
            anomalies={anomalies}
            onAcknowledge={handleAcknowledge}
          />
        )}
      </section>
    </div>
  );
}
```

### REST API Integration

**Dashboard REST Client (Python):**

```python
# sentinel_client.py
import requests
from typing import List, Dict, Optional
from datetime import datetime, timedelta

class SentinelClient:
    def __init__(self, base_url: str, api_key: str):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.session.headers.update({
            'Authorization': f'Bearer {api_key}',
            'Content-Type': 'application/json'
        })

    def get_anomalies(
        self,
        start_time: datetime,
        end_time: datetime,
        severity: Optional[List[str]] = None,
        status: Optional[List[str]] = None,
        limit: int = 100
    ) -> List[Dict]:
        """Fetch anomalies within time range"""
        params = {
            'start': start_time.isoformat(),
            'end': end_time.isoformat(),
            'limit': limit
        }

        if severity:
            params['severity'] = ','.join(severity)
        if status:
            params['status'] = ','.join(status)

        response = self.session.get(
            f'{self.base_url}/api/v1/anomalies',
            params=params
        )
        response.raise_for_status()
        return response.json()['anomalies']

    def get_metrics(
        self,
        metric_names: List[str],
        start_time: datetime,
        end_time: datetime,
        aggregation: str = 'avg',
        granularity: str = '5m'
    ) -> List[Dict]:
        """Fetch metrics time series"""
        payload = {
            'metrics': metric_names,
            'start': start_time.isoformat(),
            'end': end_time.isoformat(),
            'aggregation': aggregation,
            'granularity': granularity
        }

        response = self.session.post(
            f'{self.base_url}/api/v1/metrics/query',
            json=payload
        )
        response.raise_for_status()
        return response.json()['series']

    def get_statistics(
        self,
        start_time: datetime,
        end_time: datetime,
        group_by: Optional[List[str]] = None
    ) -> Dict:
        """Get statistics summary"""
        params = {
            'start': start_time.isoformat(),
            'end': end_time.isoformat()
        }

        if group_by:
            params['group_by'] = ','.join(group_by)

        response = self.session.get(
            f'{self.base_url}/api/v1/statistics',
            params=params
        )
        response.raise_for_status()
        return response.json()

    def acknowledge_anomaly(
        self,
        anomaly_id: str,
        acknowledged_by: str,
        note: Optional[str] = None
    ) -> Dict:
        """Acknowledge an anomaly"""
        payload = {
            'acknowledged_by': acknowledged_by,
            'note': note
        }

        response = self.session.post(
            f'{self.base_url}/api/v1/anomalies/{anomaly_id}/acknowledge',
            json=payload
        )
        response.raise_for_status()
        return response.json()

# Usage example
client = SentinelClient(
    base_url='https://sentinel.example.com',
    api_key='your-api-key'
)

# Get critical anomalies from last 24 hours
anomalies = client.get_anomalies(
    start_time=datetime.now() - timedelta(hours=24),
    end_time=datetime.now(),
    severity=['critical'],
    status=['active']
)

# Get anomaly detection rate metric
metrics = client.get_metrics(
    metric_names=['anomaly_rate', 'detection_latency_p99'],
    start_time=datetime.now() - timedelta(hours=24),
    end_time=datetime.now(),
    aggregation='avg',
    granularity='5m'
)

# Get statistics
stats = client.get_statistics(
    start_time=datetime.now() - timedelta(days=7),
    end_time=datetime.now(),
    group_by=['detector_id', 'severity']
)

print(f"Total anomalies: {stats['total_anomalies']}")
print(f"Average score: {stats['average_score']}")
```

---

## Custom Integrations

### Webhook Integration

**Custom Webhook Configuration:**

```yaml
# custom-webhook.yaml
integrations:
  webhooks:
    - name: custom-analytics
      enabled: true
      url: https://analytics.example.com/api/sentinel-events
      events:
        - anomaly.detected
        - anomaly.resolved
        - alert.triggered
      headers:
        Authorization: "Bearer ${ANALYTICS_API_KEY}"
        X-Source: "sentinel"
      retry:
        enabled: true
        max_attempts: 3
        backoff: exponential
      timeout: 10s

    - name: slack-custom
      enabled: true
      url: https://hooks.slack.com/services/YOUR/WEBHOOK/URL
      events:
        - anomaly.detected
      filter:
        severity: [critical]
      template: |
        {
          "text": "Critical Anomaly Detected",
          "blocks": [
            {
              "type": "header",
              "text": {
                "type": "plain_text",
                "text": "{{ .anomaly.title }}"
              }
            },
            {
              "type": "section",
              "fields": [
                {
                  "type": "mrkdwn",
                  "text": "*Severity:*\n{{ .anomaly.severity }}"
                },
                {
                  "type": "mrkdwn",
                  "text": "*Score:*\n{{ .anomaly.score }}"
                }
              ]
            },
            {
              "type": "section",
              "text": {
                "type": "mrkdwn",
                "text": "{{ .anomaly.description }}"
              }
            }
          ]
        }
```

### Custom Detector Plugin

**Python Detector Plugin:**

```python
# custom_detector.py
from sentinel.detector import BaseDetector, DetectorResult
from typing import Dict, Any, List

class CustomBehavioralDetector(BaseDetector):
    """
    Custom detector plugin for behavioral anomaly detection
    """

    def __init__(self, config: Dict[str, Any]):
        super().__init__(config)
        self.detector_id = config['detector_id']
        self.threshold = config.get('threshold', 0.8)
        self.window_size = config.get('window_size', 100)
        self.event_buffer = []

    def detect(self, event: Dict[str, Any]) -> List[DetectorResult]:
        """
        Analyze event for behavioral anomalies
        """
        self.event_buffer.append(event)

        # Keep only recent events
        if len(self.event_buffer) > self.window_size:
            self.event_buffer.pop(0)

        results = []

        # Custom detection logic
        if self._is_unusual_pattern():
            results.append(DetectorResult(
                anomaly_id=self.generate_id(),
                detector_id=self.detector_id,
                severity='warning',
                score=self._calculate_score(),
                title='Unusual behavioral pattern detected',
                description=self._build_description(),
                metrics=self._get_metrics(),
                affected_entity=self._get_affected_entity(event)
            ))

        return results

    def _is_unusual_pattern(self) -> bool:
        """Check for unusual patterns in event buffer"""
        if len(self.event_buffer) < self.window_size:
            return False

        # Example: Detect rapid model switching
        models = [e.get('data', {}).get('model') for e in self.event_buffer]
        unique_models = len(set(models))

        return unique_models > self.threshold * len(models)

    def _calculate_score(self) -> float:
        """Calculate anomaly confidence score"""
        # Custom scoring logic
        return 0.85

    def _build_description(self) -> str:
        """Build human-readable description"""
        return "User is rapidly switching between different models"

    def _get_metrics(self) -> Dict[str, Any]:
        """Get relevant metrics"""
        models = [e.get('data', {}).get('model') for e in self.event_buffer]
        return {
            'window_size': len(self.event_buffer),
            'unique_models': len(set(models)),
            'switch_rate': len(set(models)) / len(models)
        }

    def _get_affected_entity(self, event: Dict[str, Any]) -> Dict[str, str]:
        """Get affected entity info"""
        return {
            'type': 'user',
            'id': event.get('metadata', {}).get('user_id', 'unknown'),
            'name': event.get('metadata', {}).get('user_id', 'unknown')
        }

# Register plugin
def register():
    return {
        'detector_type': 'custom_behavioral',
        'detector_class': CustomBehavioralDetector,
        'config_schema': {
            'type': 'object',
            'properties': {
                'threshold': {'type': 'number', 'minimum': 0, 'maximum': 1},
                'window_size': {'type': 'integer', 'minimum': 10}
            }
        }
    }
```

**Load Custom Detector:**

```yaml
# detector-config.yaml
detectors:
  - id: custom-model-switching
    type: custom_behavioral
    enabled: true
    plugin: /etc/sentinel/plugins/custom_detector.py
    config:
      threshold: 0.5
      window_size: 100
    actions:
      - alert: unusual-behavior
        severity: warning
```

---

## End-to-End Scenarios

### Scenario 1: Prompt Injection Detection and Response

**Flow:**

```
1. Observatory captures LLM request
   ↓
2. Sentinel ingests telemetry
   ↓
3. ML detector identifies prompt injection
   ↓
4. Sentinel triggers multiple actions:
   - Blocks request via Shield
   - Creates incident in Incident Manager
   - Sends alert to security team
   - Updates Governance Dashboard
   ↓
5. Security team investigates and resolves
   ↓
6. Sentinel updates incident and metrics
```

**Configuration:**

```yaml
detectors:
  - id: prompt-injection-ml
    type: ml
    model: prompt-injection-v2
    threshold: 0.85
    actions:
      # Immediate blocking
      - type: shield
        config:
          action: block
          duration: 1h

      # Incident creation
      - type: incident
        config:
          auto_create: true
          severity: P1

      # Alert security team
      - type: alert
        config:
          receivers: [security-team, pagerduty-security]
          severity: critical

      # Update dashboard
      - type: webhook
        config:
          url: https://dashboard.example.com/api/events
```

### Scenario 2: Cost Anomaly Detection and Budget Control

**Flow:**

```
1. Observatory tracks LLM API costs
   ↓
2. Sentinel aggregates cost metrics
   ↓
3. Statistical detector identifies cost spike
   ↓
4. Sentinel rate-limits via Shield
   ↓
5. Alerts finance and engineering teams
   ↓
6. Dashboard shows cost trends
   ↓
7. Teams investigate and optimize
```

**Configuration:**

```yaml
detectors:
  - id: cost-spike-detector
    type: statistical
    config:
      metric: cost_per_hour
      strategy: zscore
      threshold: 3.0
      window: 1h
    actions:
      # Gradual rate limiting
      - type: shield
        config:
          action: rate_limit
          parameters:
            limit: 1000
            window: 1h

      # Alert teams
      - type: alert
        config:
          receivers: [finance-team, engineering-team]
          severity: warning

      # Dashboard notification
      - type: webhook
        config:
          url: https://dashboard.example.com/api/cost-alerts
```

---

This integration guide provides comprehensive examples for connecting LLM-Sentinel with all components of the LLM governance ecosystem. All code examples are production-ready templates that can be adapted to specific deployment requirements.
