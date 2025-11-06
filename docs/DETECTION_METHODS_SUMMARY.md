# LLM Anomaly Detection Methods - Quick Reference

## Executive Summary

This document provides state-of-the-art anomaly detection techniques for LLM runtime monitoring based on 2024-2025 research and industry best practices. The comprehensive guide covers five critical detection categories with practical implementation guidance.

## Detection Categories Overview

### 1. Model Drift Detection
**What it detects:** Changes in input/output distributions and model behavior over time

**Key Statistics:**
- 75% of businesses observed AI performance declines without monitoring (2024)
- Models unchanged for 6+ months saw 35% error rate increase

**Top Methods:**
- **Population Stability Index (PSI)**: Symmetric KL divergence variant
  - Thresholds: <0.10 (OK), 0.10-0.25 (investigate), >0.25 (action required)
- **KL Divergence**: Information-theoretic drift measurement
- **Embedding Drift**: Semantic drift detection using vector spaces
- **Canary Prompts**: Unchanging test inputs to detect behavioral shifts

**Best For:** Daily/weekly batch analysis

---

### 2. Latency Anomalies
**What it detects:** Response time outliers, throughput degradation, queue issues

**Top Methods:**

**Statistical (Real-Time):**
- Z-Score: Fast, <1ms latency, >100K/sec throughput
- IQR: Robust to outliers, handles skewed distributions
- MAD: Best for highly skewed data

**Machine Learning:**
- Isolation Forest: 5-10ms latency, 1-5K/sec throughput
- LSTM Autoencoder: Complex patterns, 50-200ms (CPU), 5-20ms (GPU)

**Time-Series:**
- Prophet: Automatic seasonality detection, 1-10 sec training
- ARIMA: Traditional forecasting, good for smaller datasets

**Best For:** Real-time streaming (statistical) + daily trend analysis (ML/time-series)

---

### 3. Hallucination Detection
**What it detects:** Factual errors, inconsistencies, low-confidence outputs

**Research Breakthrough (2024):**
- **LLM-Check (NeurIPS 2024)**: 450x faster than multi-inference methods
  - Analyzes attention maps, activations, probabilities in single pass
  - Suitable for real-time production

**Top Methods:**
- **Self-Consistency**: Sample 3-5 responses, measure agreement (score >0.90 = factual)
- **Perplexity Monitoring**: High perplexity signals uncertainty/hallucination
- **RAG Verification**: Cross-reference claims with knowledge base
- **Semantic Entropy**: Cluster outputs to quantify uncertainty

**Industry Adoption:**
- Datadog: Real-time hallucination tracking and alerting
- Threshold: >5% hallucination rate triggers investigation

**Best For:** Real-time (LLM-Check) + batch validation (self-consistency)

---

### 4. Cost Anomalies
**What it detects:** Token usage spikes, expensive prompts, budget violations

**Impact:** Organizations identified cost anomalies representing 20-40% of LLM expenses (2024)

**Top Methods:**

**Token Usage Monitoring:**
- CUSUM (Cumulative Sum): Detects gradual and sudden changes
- Prophet: Predicts daily/monthly costs with seasonality
- Isolation Forest: Multi-dimensional resource consumption

**Cost Tracking (Platform Examples):**
- **Opik**: Trace-level cost monitoring, high-cost request filtering
- **Datadog CCM**: Granular token usage and cost insights
- **Alert Thresholds**:
  - 80% quota: Notify user
  - 90% quota: Reduce priority
  - 100% quota: Rate limit (429 status)

**Best For:** Real-time rate limiting + daily budget analysis

---

### 5. Quality Degradation
**What it detects:** Output coherence decline, task performance drift, user satisfaction drops

**Top Methods:**

**Output Coherence:**
- Semantic Consistency: Embedding similarity between segments (threshold: 0.75)
- Logical Flow: Entity/temporal/causal consistency checks

**Task Performance:**
- Success Rate Tracking: Alert on >10% degradation
- A/B Testing: Shadow evaluation against reference model

**User Feedback:**
- Explicit: Thumbs up/down, ratings, regeneration rate
- Implicit: Abandonment, edit time, engagement metrics

**Safety Monitoring:**
- Toxicity Detection: Block outputs with score >0.7
- Content Safety: Monitor across multiple dimensions (violence, hate, etc.)

**Best For:** Hourly aggregates (coherence) + daily reports (trends)

---

## Real-Time vs Batch Processing

### Real-Time Streaming (Kafka + Flink)
**Use Cases:**
- Critical alerts (SLA violations, security)
- Latency spikes, cost rate limiting
- Toxic output blocking

**Architecture:**
```
LLM → Kafka → Flink (windowing + detection) → Alerts
```

**Algorithms:** Z-score, simple thresholds, Random Cut Forest
**Latency:** <100ms detection overhead
**Throughput:** 10K-100K events/sec

### Batch Processing
**Use Cases:**
- Drift detection, comprehensive quality analysis
- Model retraining, trend reports
- Deep ML models (LSTM, Isolation Forest full training)

**Schedule:**
- Hourly: Cost analysis, aggregate metrics
- Daily: Quality reports, drift detection
- Weekly: Model retraining, threshold tuning

**Algorithms:** Isolation Forest, LSTM, Prophet, PSI/KL divergence

### Recommended: Hybrid (Lambda Architecture)
- **Speed Layer**: Real-time critical detection
- **Batch Layer**: Comprehensive analysis and ML
- **Serving Layer**: Unified dashboard with merged insights

---

## Threshold Tuning Strategies

### Statistical Baselines
- **Z-Score**: 2σ (sensitive), 3σ (balanced), 4σ (conservative)
- **IQR**: 1.5x (standard), 2.0x (relaxed), 1.0x (strict)
- **Percentiles**: p95 (5% anomalies), p99 (1%), p99.9 (0.1%)

### ML Parameter Tuning
- **Isolation Forest contamination**: 0.01-0.02 (clean), 0.05-0.10 (noisy)
- **LSTM threshold**: mean + 2-3σ reconstruction error

### Business-Driven
- **Cost-Sensitive**: Minimize (FP × FP_cost) + (FN × FN_cost)
- **SLA-Based**: Warning at 80%, alert at 100%, critical at 120% of SLA

### Multi-Threshold Severity
- INFO: 1.5-2σ (log only)
- WARNING: 2-3σ (notify channel)
- ALERT: 3-4σ (page on-call)
- CRITICAL: >4σ (escalate)

### Continuous Optimization
- A/B testing thresholds over 7-14 days
- Human-in-the-loop feedback collection
- Weekly/monthly retuning based on labeled data

---

## False Positive Mitigation

### Top Strategies

**1. Persistence Filters**
- Require N consecutive violations (N=2-3 for latency, 5-10 for drift)
- Time-based: Anomaly must persist for T seconds

**2. Contextual Detection**
- Segment by: time-of-day, day-of-week, user-type, model version
- Separate baselines per context reduce seasonal false positives

**3. Ensemble Voting**
- Unanimous: All detectors agree → HIGH confidence
- Majority: 2+ detectors → MEDIUM confidence
- Single detector → LOW confidence (log only)

**4. Root Cause Correlation**
- True anomalies appear in multiple metrics
- Check dependency graph (API → LLM → DB)
- Correlated anomalies = higher confidence

**5. ML False Positive Classifier**
- Two-stage: Statistical detector → ML classifier
- Features: time-of-day, trends, correlations
- Filters known false positive patterns

**6. Alert Aggregation**
- Debouncing: Max 1 alert per 15 minutes
- Grouping: Batch similar anomalies in 5-min window
- Suppress low-severity during critical incidents

---

## Performance Implications

### Computational Complexity

| Method | Training | Inference | Latency | Throughput | Best For |
|--------|----------|-----------|---------|------------|----------|
| Z-Score | O(1) | O(1) | <1ms | >100K/sec | Real-time |
| IQR | O(n) | O(1) | <5ms | >50K/sec | Real-time |
| Isolation Forest | O(t×n log n) | O(t×log n) | 5-10ms | 1-5K/sec | Batch/moderate RT |
| One-Class SVM | O(n²-n³) | O(n_sv) | 10-50ms | 500-2K/sec | Batch |
| LSTM Autoencoder | O(epochs×n×seq×params) | O(seq×params) | 50-200ms CPU | 200-1K/sec | Batch |
| Prophet | O(n) | O(1) | 1-10s train | N/A | Batch daily/hourly |

### Latency Budget
- **Target**: Detection overhead <5% of request latency
- **Example**: 2000ms LLM request → <100ms detection
- **Optimization**: Async detection, sampling, tiered approaches

### Memory Footprint
- **Z-Score**: 24 bytes/metric (mean, variance, count)
- **t-digest**: ~1.6 KB/metric (quantile estimation)
- **Isolation Forest**: ~10 MB (100 trees)
- **LSTM**: 5-50 MB model + 128 KB/batch

### Scalability
- **Stateless**: Easy horizontal scaling (thresholds)
- **Stateful**: Partition by user_id, consistent routing
- **Data volume**: Aggregate to reduce 100K/sec → 278/sec (3600x)

### Cost Optimization
- **Real-time (Flink)**: ~$735/month (4 instances)
- **Batch (EMR)**: ~$76/month (1 hour/day)
- **Storage**: ~$16/month (compressed, downsampled)
- **Model training**: Weekly LSTM retraining ~$24/month

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- Basic statistical methods (Z-score, IQR)
- Logging and metrics infrastructure
- Simple threshold alerts
- Baseline establishment

### Phase 2: Real-Time Detection (Weeks 3-4)
- Kafka + Flink streaming pipeline
- Isolation Forest deployment
- Real-time alerting (PagerDuty, Slack)
- Initial threshold configuration

### Phase 3: Advanced Detection (Weeks 5-8)
- LSTM autoencoders
- Drift detection (PSI, KL divergence)
- Hallucination detection (LLM-Check)
- Quality monitoring (coherence, feedback)

### Phase 4: Optimization (Weeks 9-12)
- Threshold tuning from feedback
- False positive reduction
- Performance optimization
- Dashboards and reporting

### Phase 5: Continuous Improvement (Ongoing)
- Labeled data collection
- Model retraining (weekly/monthly)
- A/B testing
- Scale as needed

---

## Key Takeaways

1. **No Single Solution**: Combine statistical, ML, and time-series methods
2. **Hybrid Architecture**: Real-time + batch for best coverage and accuracy
3. **Context Matters**: Segment by user, time, model for better baselines
4. **Tune Continuously**: Use feedback loops and A/B testing
5. **Balance Trade-offs**: Accuracy vs. latency vs. cost vs. false positives
6. **Start Simple**: Statistical methods first, add complexity as needed
7. **Monitor the Monitors**: Track detection system health and performance

---

## Quick Algorithm Selection Guide

**Need real-time latency detection?**
→ Z-Score or IQR (sub-millisecond)

**High-dimensional anomalies (cost, latency, quality)?**
→ Isolation Forest (batch or moderate real-time)

**Detecting drift in distributions?**
→ PSI or KL Divergence (daily batch)

**Hallucination detection?**
→ LLM-Check (real-time) or Self-Consistency (batch)

**Complex temporal patterns?**
→ LSTM Autoencoder (batch with GPU)

**Forecasting costs or trends?**
→ Prophet (daily/hourly forecasts)

**Streaming data with concept drift?**
→ Kafka + Flink + Random Cut Forest

**User feedback and quality?**
→ Semantic Coherence + Feedback Aggregation (hourly/daily)

---

## Resources

### Platforms (2024-2025)
- Datadog LLM Observability
- Opik (Comet)
- WhyLabs
- New Relic AI Monitoring
- Dynatrace AI Observability

### Open Source
- scikit-learn (Isolation Forest, One-Class SVM)
- TensorFlow/PyTorch (LSTM)
- sentence-transformers (embeddings)
- evidently (drift detection)
- Apache Flink, Kafka (streaming)

### Research
- LLM-Check (NeurIPS 2024)
- Task Drift with Activations (arXiv 2024)
- LSTM + Isolation Forest hybrid methods

---

**Document Version:** 1.0
**Last Updated:** 2025-11-06
**Companion Document:** DETECTION_METHODS.md (full technical guide)
