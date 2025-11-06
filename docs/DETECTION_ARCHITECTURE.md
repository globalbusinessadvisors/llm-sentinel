# LLM Anomaly Detection Architecture Reference

## System Architecture Diagrams

### 1. Overall Monitoring Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         LLM Application                              │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐        │
│  │ User API │───│ Prompt   │───│   LLM    │───│ Response │        │
│  │          │   │ Processing│   │ Inference│   │ Handler  │        │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘        │
│       │              │              │              │                │
│       └──────────────┴──────────────┴──────────────┘                │
│                            │                                         │
│                    ┌───────▼────────┐                               │
│                    │ Metrics Logger │                               │
│                    └───────┬────────┘                               │
└────────────────────────────┼─────────────────────────────────────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
    ┌─────────▼─────────┐       ┌──────────▼──────────┐
    │   Real-Time Path  │       │    Batch Path       │
    │   (Kafka/Flink)   │       │  (Data Warehouse)   │
    └─────────┬─────────┘       └──────────┬──────────┘
              │                             │
    ┌─────────▼─────────┐       ┌──────────▼──────────┐
    │ Fast Detection    │       │ Deep Analysis       │
    │ - Z-Score         │       │ - Isolation Forest  │
    │ - Thresholds      │       │ - LSTM Autoencoder  │
    │ - Simple Rules    │       │ - Drift Detection   │
    └─────────┬─────────┘       └──────────┬──────────┘
              │                             │
              └──────────────┬──────────────┘
                             │
                   ┌─────────▼─────────┐
                   │  Alert Manager    │
                   │  - Aggregation    │
                   │  - Deduplication  │
                   │  - Routing        │
                   └─────────┬─────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
    ┌─────────▼─────────┐       ┌──────────▼──────────┐
    │  PagerDuty/Slack  │       │    Dashboard        │
    │  (Critical)       │       │   (Monitoring)      │
    └───────────────────┘       └─────────────────────┘
```

---

### 2. Real-Time Streaming Architecture (Kafka + Flink)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        LLM Services                                  │
└───────┬──────────────┬──────────────┬──────────────┬────────────────┘
        │              │              │              │
        │ (metrics)    │ (metrics)    │ (metrics)    │ (metrics)
        │              │              │              │
        ▼              ▼              ▼              ▼
┌────────────────────────────────────────────────────────────┐
│                    Apache Kafka                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  Topic: llm-metrics-raw                              │ │
│  │  - Latency, tokens, cost, quality per request        │ │
│  │  - Partitioned by user_id                            │ │
│  │  - Retention: 7 days                                 │ │
│  └──────────────────────────────────────────────────────┘ │
└────────────────────────────┬───────────────────────────────┘
                             │
                             │ (stream)
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                  Apache Flink Cluster                        │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Source (Kafka Consumer)                   │ │
│  └───────────────────────┬────────────────────────────────┘ │
│                          │                                   │
│  ┌───────────────────────▼────────────────────────────────┐ │
│  │         Windowing (1-min tumbling windows)             │ │
│  │  - Aggregate metrics per window                        │ │
│  │  - Calculate rolling statistics                        │ │
│  └───────────────────────┬────────────────────────────────┘ │
│                          │                                   │
│  ┌───────────────────────▼────────────────────────────────┐ │
│  │          Anomaly Detection Functions                   │ │
│  │                                                         │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐ │ │
│  │  │  Z-Score    │  │ Percentile   │  │  RCF (AWS)   │ │ │
│  │  │  Detection  │  │  Thresholds  │  │  Detection   │ │ │
│  │  └─────────────┘  └──────────────┘  └──────────────┘ │ │
│  │                                                         │ │
│  └───────────────────────┬────────────────────────────────┘ │
│                          │                                   │
│  ┌───────────────────────▼────────────────────────────────┐ │
│  │        Sink (Kafka Producer + Time-Series DB)          │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└────────────┬─────────────────────────┬───────────────────────┘
             │                         │
             ▼                         ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│ Kafka Topic:            │   │  Time-Series DB         │
│ llm-metrics-anomalies   │   │  (InfluxDB/Prometheus)  │
│ - Anomaly events        │   │  - All metrics          │
│ - Scores and metadata   │   │  - 30-day retention     │
└─────────┬───────────────┘   └─────────┬───────────────┘
          │                             │
          ▼                             ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│  Lambda/Function        │   │  Grafana Dashboard      │
│  - Alert routing        │   │  - Real-time charts     │
│  - Notification logic   │   │  - Anomaly markers      │
└─────────────────────────┘   └─────────────────────────┘
```

---

### 3. Batch Processing Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Data Lake (S3/GCS)                       │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Raw LLM Logs (JSON/Parquet)                           │ │
│  │  - Partitioned by date, model, endpoint                │ │
│  │  - Retention: 90 days                                  │ │
│  └────────────────────────────────────────────────────────┘ │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           │ (scheduled ETL)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│            ETL Pipeline (Airflow/Databricks)                 │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Daily Job (1 AM)                                      │ │
│  │  1. Extract yesterday's logs                           │ │
│  │  2. Transform: parse, aggregate, calculate features    │ │
│  │  3. Load to Data Warehouse                             │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│           Data Warehouse (Snowflake/BigQuery)                │
│                                                              │
│  ┌────────────────────┐  ┌────────────────────┐            │
│  │  Metrics Table     │  │  Aggregates Table  │            │
│  │  - Per request     │  │  - Daily/hourly    │            │
│  └────────────────────┘  └────────────────────┘            │
│                                                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           │ (scheduled jobs)
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              Batch Anomaly Detection Jobs                    │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Hourly Job                                            │ │
│  │  - Cost analysis (token usage trends)                  │ │
│  │  - High-level metrics aggregation                      │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Daily Job                                             │ │
│  │  - Drift detection (PSI, KL divergence)                │ │
│  │  - Quality analysis (coherence, feedback)              │ │
│  │  - Isolation Forest training and detection             │ │
│  │  - Generate daily report                               │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Weekly Job                                            │ │
│  │  - LSTM Autoencoder retraining                         │ │
│  │  - Threshold optimization                              │ │
│  │  - Model performance review                            │ │
│  │  - Generate weekly trend report                        │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
└──────────────────────────┬──────────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│  Results Table          │   │  Email Reports          │
│  - Anomalies detected   │   │  - Stakeholder summary  │
│  - Scores and metadata  │   │  - Actionable insights  │
└─────────┬───────────────┘   └─────────────────────────┘
          │
          ▼
┌─────────────────────────┐
│  BI Dashboard           │
│  (Tableau/Looker)       │
│  - Trends, insights     │
└─────────────────────────┘
```

---

## Decision Trees

### Algorithm Selection Decision Tree

```
START: Need anomaly detection for LLM monitoring
│
├─ Q1: Is real-time detection required (<1 sec)?
│  │
│  ├─ YES: Real-time needed
│  │  │
│  │  ├─ Q2: Single metric or multi-dimensional?
│  │  │  │
│  │  │  ├─ Single metric (e.g., latency)
│  │  │  │  │
│  │  │  │  ├─ Q3: Data distribution?
│  │  │  │  │  ├─ Normal → Z-Score (fast, <1ms)
│  │  │  │  │  ├─ Skewed → IQR or MAD
│  │  │  │  │  └─ Unknown → IQR (safe default)
│  │  │  │
│  │  │  └─ Multi-dimensional (latency + cost + quality)
│  │  │     │
│  │  │     ├─ Throughput > 10K/sec → Z-Score per dimension
│  │  │     ├─ Throughput 1-10K/sec → Streaming RCF (AWS Kinesis)
│  │  │     └─ Throughput < 1K/sec → Isolation Forest
│  │  │
│  │  └─ Q4: Streaming infrastructure available?
│  │     ├─ YES (Kafka/Flink) → Flink + RCF + Z-Score
│  │     └─ NO → Simple threshold + Z-Score in app
│  │
│  └─ NO: Batch processing acceptable
│     │
│     ├─ Q5: What type of anomaly?
│     │  │
│     │  ├─ Drift detection
│     │  │  ├─ Input/output distribution → PSI or KL Divergence
│     │  │  ├─ Embedding drift → Cosine distance + MMD
│     │  │  └─ Concept drift → Canary prompts + activation analysis
│     │  │
│     │  ├─ Hallucination detection
│     │  │  ├─ Real-time needed? → LLM-Check (single-pass)
│     │  │  ├─ High accuracy needed? → Self-Consistency (multi-sample)
│     │  │  └─ RAG system? → Retrieval-based verification
│     │  │
│     │  ├─ Cost anomalies
│     │  │  ├─ Need forecasting? → Prophet (daily/monthly)
│     │  │  ├─ Change point detection? → CUSUM or Bayesian
│     │  │  └─ Multi-dimensional? → Isolation Forest
│     │  │
│     │  ├─ Quality degradation
│     │  │  ├─ Coherence → Semantic similarity (embeddings)
│     │  │  ├─ Task performance → Success rate tracking
│     │  │  └─ User feedback → Aggregation + trend analysis
│     │  │
│     │  └─ Complex patterns (temporal dependencies)
│     │     ├─ GPU available? → LSTM Autoencoder
│     │     ├─ Seasonal patterns? → Prophet or SARIMA
│     │     └─ General time-series? → ARIMA
│     │
│     └─ Q6: Data volume?
│        ├─ Small (<100K records/day) → Any algorithm
│        ├─ Medium (100K-10M/day) → Isolation Forest, Prophet
│        └─ Large (>10M/day) → Sampling + aggregation first
│
RESULT: Algorithm selected
```

---

### Threshold Tuning Decision Tree

```
START: Need to tune anomaly detection threshold
│
├─ Q1: Do you have labeled data (known anomalies)?
│  │
│  ├─ YES: Supervised approach
│  │  │
│  │  ├─ Q2: What is priority?
│  │  │  │
│  │  │  ├─ Minimize false positives (precision)
│  │  │  │  → Set threshold at p99.5 or high Z-score (4σ)
│  │  │  │  → Test on validation set
│  │  │  │  → If precision < 80%, increase threshold
│  │  │  │
│  │  │  ├─ Minimize false negatives (recall)
│  │  │  │  → Set threshold at p95 or low Z-score (2σ)
│  │  │  │  → Test on validation set
│  │  │  │  → If recall < 80%, decrease threshold
│  │  │  │
│  │  │  └─ Balance both (F1 score)
│  │  │     → Test range of thresholds
│  │  │     → Calculate F1 = 2×(precision×recall)/(precision+recall)
│  │  │     → Select threshold maximizing F1
│  │  │
│  │  └─ Q3: Can you estimate costs?
│  │     │
│  │     ├─ YES: Cost-sensitive tuning
│  │     │  │
│  │     │  └─ Minimize: (FP × FP_cost) + (FN × FN_cost)
│  │     │     Example: FP_cost=$5, FN_cost=$5000
│  │     │     → Test thresholds, calculate total cost
│  │     │     → Select minimum cost threshold
│  │     │
│  │     └─ NO: Use F1 score approach
│  │
│  └─ NO: Unsupervised approach
│     │
│     ├─ Q4: Statistical or ML-based detection?
│     │  │
│     │  ├─ Statistical (Z-score, IQR, MAD)
│     │  │  │
│     │  │  ├─ Q5: System criticality?
│     │  │  │  ├─ Critical (safety, security) → 2σ (high sensitivity)
│     │  │  │  ├─ Standard → 3σ (balanced)
│     │  │  │  └─ Low-noise → 4σ (low false positives)
│     │  │  │
│     │  │  └─ Q6: Initial deployment
│     │  │     → Start with 3σ
│     │  │     → Monitor for 7-14 days
│     │  │     → Calculate false positive rate
│     │  │     → If FP_rate > target: increase threshold
│     │  │     → If FP_rate < target/2: decrease threshold
│     │  │     → Iterate until converged
│     │  │
│     │  └─ ML-based (Isolation Forest, Autoencoder)
│     │     │
│     │     ├─ Isolation Forest contamination
│     │     │  ├─ Well-functioning system → 0.01-0.02 (1-2%)
│     │     │  ├─ Noisy system → 0.05-0.10 (5-10%)
│     │     │  └─ Test multiple values, select based on review
│     │     │
│     │     └─ Autoencoder reconstruction threshold
│     │        ├─ Method 1: mean + kσ (k=2,3,4)
│     │        ├─ Method 2: Percentile (p95, p99, p99.5)
│     │        └─ Method 3: Manual review of top anomalies
│     │
│     └─ Q7: Business context available?
│        │
│        ├─ YES: SLA-based thresholds
│        │  │
│        │  └─ Example: SLA = 95% requests < 500ms
│        │     → Primary threshold: 500ms (SLA boundary)
│        │     → Warning: 400ms (80% of SLA)
│        │     → Critical: 600ms (120% of SLA)
│        │
│        └─ NO: Use percentile-based
│           ├─ High-traffic system → p95 (5% anomalies)
│           ├─ Standard system → p99 (1% anomalies)
│           └─ Critical system → p99.9 (0.1% anomalies)
│
├─ CONTINUOUS TUNING
│  │
│  ├─ Feedback loop
│  │  ├─ Collect labels: true positive vs false positive
│  │  ├─ Store in database with context
│  │  ├─ Weekly/monthly review
│  │  ├─ Retrain or adjust thresholds
│  │  └─ Track performance over time
│  │
│  ├─ A/B testing
│  │  ├─ Split traffic: current vs candidate threshold
│  │  ├─ Run 7-14 days
│  │  ├─ Measure: precision, recall, alert count
│  │  ├─ Statistical significance test
│  │  └─ Adopt better threshold
│  │
│  └─ Automated optimization
│     ├─ Bayesian optimization of threshold
│     ├─ Objective: maximize F1 or minimize cost
│     └─ Run periodically (monthly)
│
RESULT: Optimal threshold selected and continuously improved
```

---

### False Positive Mitigation Decision Tree

```
START: Experiencing high false positive rate
│
├─ Q1: What is current FP rate?
│  ├─ >50% → CRITICAL: Major tuning needed
│  ├─ 20-50% → HIGH: Significant improvement needed
│  ├─ 10-20% → MODERATE: Some tuning beneficial
│  └─ <10% → LOW: Minor adjustments
│
├─ DIAGNOSIS: Identify FP patterns
│  │
│  ├─ Analyze recent false positives
│  │  ├─ Time of day correlation? → Contextual issue
│  │  ├─ Specific users/endpoints? → Segmentation needed
│  │  ├─ Transient spikes? → Persistence filtering
│  │  ├─ Single metric only? → Correlation needed
│  │  └─ Random/unpredictable? → Threshold too sensitive
│  │
│  └─ Review alert history
│     ├─ Calculate FP rate per alert type
│     ├─ Identify high-FP alert sources
│     └─ Prioritize fixes by impact
│
├─ MITIGATION STRATEGIES (apply in order)
│  │
│  ├─ Strategy 1: Threshold Adjustment
│  │  ├─ If statistical: Increase σ multiplier (3→4)
│  │  ├─ If percentile: Raise percentile (p95→p99)
│  │  ├─ If ML: Increase contamination or reconstruction threshold
│  │  └─ Expected FP reduction: 30-50%
│  │
│  ├─ Strategy 2: Persistence Filtering
│  │  ├─ Require N consecutive violations
│  │  │  ├─ Fast metrics (latency): N=2-3
│  │  │  └─ Slow metrics (drift): N=5-10
│  │  ├─ Or time-based: anomaly persists T seconds
│  │  │  ├─ Latency: T=1-2 minutes
│  │  │  └─ Cost/quality: T=5-10 minutes
│  │  └─ Expected FP reduction: 40-60%
│  │
│  ├─ Strategy 3: Contextual Segmentation
│  │  ├─ Segment by:
│  │  │  ├─ Time: hour-of-day, day-of-week
│  │  │  ├─ User: type, tier, region
│  │  │  ├─ Model: version, type
│  │  │  └─ Query: complexity, category
│  │  ├─ Calculate separate baselines per segment
│  │  ├─ Apply segment-specific thresholds
│  │  └─ Expected FP reduction: 50-70%
│  │
│  ├─ Strategy 4: Multi-Metric Correlation
│  │  ├─ Require anomalies in multiple related metrics
│  │  │  Example: latency AND (cost OR quality)
│  │  ├─ Build dependency graph
│  │  ├─ Check correlated metrics for confirmation
│  │  └─ Expected FP reduction: 60-80%
│  │
│  ├─ Strategy 5: Ensemble Voting
│  │  ├─ Run multiple detectors
│  │  ├─ Voting rules:
│  │  │  ├─ Unanimous (all agree) → HIGH confidence, low FP
│  │  │  ├─ Majority (2+) → MEDIUM confidence
│  │  │  └─ Any (1+) → LOW confidence, log only
│  │  └─ Expected FP reduction: 70-85%
│  │
│  ├─ Strategy 6: ML False Positive Classifier
│  │  ├─ Collect labeled FPs and TPs
│  │  ├─ Train binary classifier (Random Forest, XGBoost)
│  │  ├─ Features: time, trends, correlations, historical patterns
│  │  ├─ Two-stage: statistical detector → ML filter
│  │  └─ Expected FP reduction: 80-90%
│  │
│  └─ Strategy 7: Alert Aggregation
│     ├─ Debouncing: max 1 alert per 15 min
│     ├─ Grouping: batch similar anomalies
│     ├─ Suppression: low-severity during critical incidents
│     └─ Expected FP reduction: 50-70% (perceived)
│
├─ MONITORING & ITERATION
│  │
│  ├─ Track metrics
│  │  ├─ False positive rate (weekly)
│  │  ├─ Precision and recall
│  │  ├─ Alert volume
│  │  └─ Time to resolution
│  │
│  ├─ Feedback collection
│  │  ├─ Engineers label each alert: TP/FP/Unclear
│  │  ├─ Store labels with context
│  │  └─ Identify new FP patterns
│  │
│  └─ Continuous improvement
│     ├─ Weekly: Review FP patterns, quick fixes
│     ├─ Monthly: Retrain models, major tuning
│     └─ Quarterly: Architecture review, new methods
│
RESULT: Reduced false positives while maintaining detection coverage
```

---

## Deployment Patterns

### Pattern 1: Minimal Viable Monitoring

**Suitable For:** MVP, small teams, limited resources

```
┌─────────────────────────────────────────┐
│         LLM Application                 │
│  ┌────────────────────────────────────┐ │
│  │  Instrumentation (simple logging)  │ │
│  └──────────────┬─────────────────────┘ │
└─────────────────┼───────────────────────┘
                  │
                  ▼
         ┌────────────────────┐
         │  Application Logs  │
         │  (CloudWatch/      │
         │   Stackdriver)     │
         └────────┬───────────┘
                  │
                  ▼
         ┌────────────────────┐
         │  Simple Alerts     │
         │  - Latency > 5s    │
         │  - Error rate > 5% │
         │  - Cost > $100/day │
         └────────┬───────────┘
                  │
                  ▼
         ┌────────────────────┐
         │  Email/Slack       │
         └────────────────────┘

Components:
- Cloud logging (CloudWatch, Stackdriver, Application Insights)
- Simple threshold alerts (built-in)
- Email/Slack notifications

Cost: ~$50-100/month
Effort: 1-2 days setup
Pros: Simple, low cost, quick to deploy
Cons: Limited insights, high false positives, no ML
```

---

### Pattern 2: Standard Production Monitoring

**Suitable For:** Production systems, medium teams, growing scale

```
┌─────────────────────────────────────────────────────────┐
│              LLM Application                            │
│  ┌────────────────────────────────────────────────────┐ │
│  │  OpenTelemetry Instrumentation                     │ │
│  │  - Traces, metrics, logs                           │ │
│  └───────────────────┬────────────────────────────────┘ │
└──────────────────────┼──────────────────────────────────┘
                       │
        ┌──────────────┴──────────────┐
        │                             │
        ▼                             ▼
┌───────────────────┐       ┌───────────────────┐
│  Metrics Backend  │       │  Logging Backend  │
│  (Prometheus/     │       │  (Elasticsearch/  │
│   Datadog)        │       │   Loki)           │
└────────┬──────────┘       └─────────┬─────────┘
         │                            │
         ▼                            ▼
┌───────────────────┐       ┌───────────────────┐
│  Grafana          │       │  Kibana/          │
│  - Dashboards     │       │  Log Explorer     │
│  - Visualizations │       │  - Search logs    │
└────────┬──────────┘       └───────────────────┘
         │
         ▼
┌───────────────────┐
│  Alert Manager    │
│  - Z-score        │
│  - Thresholds     │
│  - Percentiles    │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│  PagerDuty/Opsgenie │
└───────────────────┘

Components:
- OpenTelemetry for instrumentation
- Prometheus + Grafana for metrics
- Elasticsearch/Loki for logs
- Alert Manager for detection
- PagerDuty for on-call

Cost: ~$500-1,500/month
Effort: 1-2 weeks setup
Pros: Standard tools, good visibility, scalable
Cons: Manual threshold tuning, limited ML
```

---

### Pattern 3: Enterprise AI Observability

**Suitable For:** Large-scale deployments, dedicated ML teams, high reliability

```
┌──────────────────────────────────────────────────────────────┐
│                  LLM Applications (multiple)                  │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │  Comprehensive Instrumentation                           │ │
│  │  - OpenTelemetry (traces, metrics, logs)                 │ │
│  │  - LLM-specific metrics (tokens, prompts, outputs)       │ │
│  │  - User feedback collection                              │ │
│  └────────────────────────┬─────────────────────────────────┘ │
└───────────────────────────┼───────────────────────────────────┘
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
        ▼                                       ▼
┌────────────────────┐              ┌────────────────────┐
│  Real-Time Layer   │              │   Batch Layer      │
│  (Kafka + Flink)   │              │   (Spark/Databricks)│
└────────┬───────────┘              └─────────┬──────────┘
         │                                    │
         │ ┌──────────────────────────────────┘
         │ │
         ▼ ▼
┌────────────────────────────────────────────────────────┐
│           AI Observability Platform                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  Datadog LLM │  │   WhyLabs    │  │   Arize AI   │ │
│  │ Observability│  │              │  │              │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                         │
│  Features:                                              │
│  - Drift detection (PSI, KL divergence)                │
│  - Hallucination detection (multiple methods)          │
│  - Cost monitoring and forecasting                     │
│  - Quality scoring (coherence, relevance)              │
│  - Embedding drift visualization                       │
│  - Model comparison and A/B testing                    │
└────────────────────┬───────────────────────────────────┘
                     │
     ┌───────────────┴───────────────┐
     │                               │
     ▼                               ▼
┌─────────────────┐         ┌─────────────────┐
│  Alert Routing  │         │  Analytics      │
│  - Intelligent  │         │  - BI Dashboard │
│  - Context-aware│         │  - Reports      │
│  - Multi-channel│         │  - Insights     │
└────────┬────────┘         └─────────────────┘
         │
         ▼
┌─────────────────┐
│  Incident Mgmt  │
│  (PagerDuty +   │
│   Slack + JIRA) │
└─────────────────┘

Additional:
┌─────────────────┐
│  ML Pipeline    │
│  - Weekly model │
│    retraining   │
│  - Threshold    │
│    optimization │
│  - A/B testing  │
└─────────────────┘

Components:
- Kafka + Flink for real-time streaming
- Spark/Databricks for batch processing
- AI observability platform (Datadog, WhyLabs, Arize)
- Data warehouse (Snowflake/BigQuery)
- ML model management
- Comprehensive alerting and analytics

Cost: ~$5,000-20,000/month
Effort: 4-8 weeks setup, ongoing team
Pros: Comprehensive insights, ML-powered, production-ready
Cons: High cost, complex setup, requires expertise
```

---

## Monitoring Metrics Dashboard Layout

### Recommended Dashboard Structure

```
┌────────────────────────────────────────────────────────────────┐
│                    LLM Monitoring Dashboard                     │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  KEY METRICS (Last 24 Hours)                            │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   │  │
│  │  │Requests │  │Avg      │  │Total    │  │Error    │   │  │
│  │  │125.3K   │  │Latency  │  │Cost     │  │Rate     │   │  │
│  │  │         │  │2.3s     │  │$1,247   │  │0.8%     │   │  │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘   │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  LATENCY MONITORING                                      │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │                                                     │ │  │
│  │  │  Latency (p50, p95, p99) - Last 24h               │ │  │
│  │  │  [Line graph with anomaly markers]                │ │  │
│  │  │                                                     │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  │  Current: p50=1.8s, p95=4.2s, p99=7.1s                │  │
│  │  Anomalies detected: 3 (marked in red)                │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  COST MONITORING                                         │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │  Cost by Model - Last 7 Days                       │ │  │
│  │  │  [Stacked area chart]                              │ │  │
│  │  │  - GPT-4: $8,234                                   │ │  │
│  │  │  - GPT-3.5: $2,456                                 │ │  │
│  │  │  - Claude: $1,890                                  │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  │  ┌───────────────┐  ┌───────────────┐                  │  │
│  │  │Monthly Budget │  │Projected      │                  │  │
│  │  │$50,000        │  │Spend: $48,200 │                  │  │
│  │  │96% used       │  │(Warning!)     │                  │  │
│  │  └───────────────┘  └───────────────┘                  │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  DRIFT DETECTION                                         │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │  Input Distribution (PSI) - Last 30 Days           │ │  │
│  │  │  [Time series of PSI scores]                       │ │  │
│  │  │  Current PSI: 0.18 (Moderate drift - investigate) │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │  Output Embedding Drift                            │ │  │
│  │  │  [Scatter plot of embedding space]                 │ │  │
│  │  │  Centroid shift: 0.12 (baseline vs current)       │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  QUALITY METRICS                                         │  │
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐           │  │
│  │  │Hallucin.  │  │Coherence  │  │User Sat.  │           │  │
│  │  │Rate       │  │Score      │  │Score      │           │  │
│  │  │2.3%       │  │0.87       │  │4.2/5.0    │           │  │
│  │  │(↑ 0.5%)   │  │(↓ 0.03)   │  │(↓ 0.2)    │           │  │
│  │  └───────────┘  └───────────┘  └───────────┘           │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │  Quality Trend - Last 7 Days                       │ │  │
│  │  │  [Multi-line graph]                                │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  ACTIVE ALERTS                                           │  │
│  │  ┌────────────────────────────────────────────────────┐ │  │
│  │  │  🔴 CRITICAL: Latency p99 > 10s (ongoing 15 min)  │ │  │
│  │  │  🟡 WARNING: Cost on track to exceed budget by 15%│ │  │
│  │  │  🟡 WARNING: Hallucination rate increased 0.5%    │ │  │
│  │  └────────────────────────────────────────────────────┘ │  │
│  └─────────────────────────────────────────────────────────┘  │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

---

**Document Version:** 1.0
**Last Updated:** 2025-11-06
**Related Documents:**
- DETECTION_METHODS.md (comprehensive technical guide)
- DETECTION_METHODS_SUMMARY.md (quick reference)
