# LLM Anomaly Detection Methods

## Table of Contents
- [1. Model Drift Detection](#1-model-drift-detection)
- [2. Latency Anomalies](#2-latency-anomalies)
- [3. Hallucination Detection](#3-hallucination-detection)
- [4. Cost Anomalies](#4-cost-anomalies)
- [5. Quality Degradation](#5-quality-degradation)
- [6. Real-Time vs Batch Processing](#6-real-time-vs-batch-processing)
- [7. Threshold Tuning Strategies](#7-threshold-tuning-strategies)
- [8. False Positive Mitigation](#8-false-positive-mitigation)
- [9. Performance Implications](#9-performance-implications)

---

## 1. Model Drift Detection

Model drift encompasses both data drift (changes in input distribution) and concept drift (changes in the relationship between inputs and outputs). Research shows that without monitoring, models left unchanged for 6+ months saw error rates jump 35% on new data, with 75% of businesses in 2024 observing AI performance declines over time.

### 1.1 Input Distribution Drift

#### **Population Stability Index (PSI)**
PSI is a symmetric variant of KL divergence specifically designed for model monitoring.

**Algorithm:**
```
PSI = Σ (% current - % baseline) × ln(% current / % baseline)
```

**Interpretation Thresholds:**
- PSI < 0.10: Little change, no action required
- 0.10 < PSI < 0.25: Moderate change, investigation recommended
- PSI > 0.25: Significant change, action required

**Use Case:** Detecting shifts in user behavior, vocabulary, or input structure by comparing current input patterns with training baseline.

**Implementation Considerations:**
- Calculate PSI for each feature independently
- Use sliding windows for continuous monitoring (e.g., hourly, daily)
- Store baseline distributions from training or initial production data
- Works well for categorical and binned continuous variables

#### **Kullback-Leibler (KL) Divergence**
KL divergence quantifies the information lost when applying past training distributions to current traffic.

**Algorithm:**
```
KL(P || Q) = Σ P(x) × log(P(x) / Q(x))
```

**Characteristics:**
- Non-symmetric metric (KL(P||Q) ≠ KL(Q||P))
- Range: 0 to infinity (0 = identical distributions)
- More sensitive to tail differences than PSI

**When to Use:**
- Continuous probability distributions
- When direction of drift matters (reference vs current)
- Fine-grained drift detection in high-dimensional spaces

**Practical Thresholds:**
- KL < 0.1: Minimal drift
- 0.1 < KL < 0.5: Moderate drift
- KL > 0.5: Significant drift

#### **Statistical Tests**

**Kolmogorov-Smirnov (KS) Test:**
- Tests if two samples come from the same distribution
- Works for continuous univariate data
- Threshold: p-value < 0.05 indicates significant drift

**Chi-Square Test:**
- Tests independence for categorical features
- Compares observed vs expected frequencies
- Threshold: p-value < 0.05 indicates significant drift

**Jensen-Shannon Divergence:**
- Symmetric version of KL divergence
- Range: 0 to 1 (normalized)
- More stable for distribution comparison

### 1.2 Output Distribution Drift

#### **Embedding-Based Drift Detection**
Embedding distributions encode subtle semantic cues that reveal when a model starts generating off-pattern responses.

**Methods:**

**1. Embedding Space Monitoring:**
```
Algorithm:
1. Generate embeddings for outputs using sentence transformers
2. Calculate distribution metrics in embedding space
3. Compare with baseline using cosine similarity or Euclidean distance
4. Track drift using multivariate methods
```

**2. Dimensionality Reduction + Statistical Tests:**
```
1. Apply PCA/t-SNE to reduce embedding dimensions
2. Apply KS test or MMD (Maximum Mean Discrepancy) on reduced space
3. Track cluster centroids over time
```

**Detection Methods for Embedding Drift:**

1. **Model-based Reconstruction Error:**
   - Train autoencoder on baseline embeddings
   - Monitor reconstruction error on new embeddings
   - Threshold: 2-3 standard deviations above baseline

2. **Domain Classifier (Discriminative Approach):**
   - Train binary classifier to distinguish baseline vs current embeddings
   - AUC > 0.75 indicates significant drift

3. **Distance-based Methods:**
   - Calculate centroid distance between baseline and current batches
   - Use Wasserstein distance for distribution comparison
   - Monitor embedding space density changes

### 1.3 Concept Drift Detection

#### **Task Drift via Activation Analysis**
A cutting-edge 2024 approach looks inside the LLM's internal activation patterns.

**Algorithm:**
```
1. Extract hidden layer activations during inference
2. Track activation pattern distributions over time
3. Use PCA on activations to identify drift dimensions
4. Monitor reconstruction error of activation patterns
```

**Benefits:**
- Detects drift before it manifests in outputs
- Model-intrinsic approach (no external data needed)
- Can identify specific layers where drift occurs

#### **Canary Prompts**
Use unchanging test inputs to surface behavioral drift.

**Implementation:**
```
1. Maintain set of 50-100 representative prompts
2. Run canary prompts daily/hourly
3. Track response variations using:
   - Semantic similarity scores
   - Perplexity changes
   - Output length variations
   - Sentiment/tone shifts
```

**Anomaly Detection:**
- Baseline: First 30 days of responses
- Alert if similarity score drops below 0.85
- Alert if perplexity increases >20%

### 1.4 Performance Metrics for Drift

**Perplexity Monitoring:**
```
Perplexity = exp(-1/N × Σ log P(xi))
```
- High perplexity signals drift or increased hallucinations
- Track perplexity per prompt category
- Threshold: >15% increase from baseline

**Semantic Coherence:**
```
1. Calculate sentence embeddings for output segments
2. Measure cosine similarity between adjacent segments
3. Track coherence score: average similarity
4. Alert if coherence drops >10%
```

---

## 2. Latency Anomalies

Latency monitoring is critical for detecting infrastructure issues, model degradation, and resource contention. Modern platforms report 50% reduction in MTTR with ML-based latency anomaly detection.

### 2.1 Response Time Outliers

#### **Statistical Methods**

**Z-Score Detection:**
```
z = (x - μ) / σ
where:
  x = current latency
  μ = mean latency (rolling window)
  σ = standard deviation

Anomaly if: |z| > 3
```

**Advantages:**
- Simple, fast computation
- Works well for normally distributed latencies
- Low memory overhead

**Limitations:**
- Assumes normal distribution
- Sensitive to outliers in baseline
- Fixed threshold may not suit all patterns

**Interquartile Range (IQR):**
```
IQR = Q3 - Q1
Lower bound = Q1 - 1.5 × IQR
Upper bound = Q3 + 1.5 × IQR

Anomaly if: latency < lower bound OR latency > upper bound
```

**Advantages:**
- Robust to outliers
- Works with skewed distributions
- No distribution assumptions

**Median Absolute Deviation (MAD):**
```
MAD = median(|xi - median(x)|)
Modified Z-score = 0.6745 × (x - median) / MAD

Anomaly if: |Modified Z-score| > 3.5
```

**Best for:**
- Highly skewed latency distributions
- Systems with occasional extreme outliers
- More robust than standard Z-score

#### **Machine Learning Methods**

**Isolation Forest:**
```
Algorithm:
1. Build ensemble of isolation trees
2. Randomly select features and split points
3. Measure path length to isolate each point
4. Anomaly score = average path length (shorter = more anomalous)

Parameters:
- n_estimators: 100-200 trees
- contamination: 0.01-0.05 (expected anomaly rate)
- max_samples: 256-512 for large datasets
```

**Advantages:**
- Handles multi-dimensional latency features (p50, p95, p99)
- No need for normal distribution
- Fast training and inference
- Detects global and local outliers

**Use Case:**
```
Features for latency anomaly detection:
- Request latency (ms)
- Token generation rate (tokens/sec)
- Time-to-first-token (TTFT)
- Queue wait time
- Upstream service latency
- Request complexity (token count, context length)
```

**One-Class SVM:**
```
Algorithm:
1. Learn decision boundary around normal latency patterns
2. Use RBF kernel for non-linear patterns
3. ν parameter controls anomaly proportion

Parameters:
- kernel: 'rbf'
- nu: 0.01-0.05
- gamma: 'scale' or 'auto'
```

**Best for:**
- Complex latency patterns
- Multi-modal distributions
- When training data is mostly normal

**Performance:** Slower than Isolation Forest for large datasets

#### **Time-Series Methods**

**ARIMA (AutoRegressive Integrated Moving Average):**
```
Model: ARIMA(p, d, q)
where:
  p = autoregressive order
  d = differencing order
  q = moving average order

Steps:
1. Fit ARIMA on historical latency
2. Generate predictions with confidence intervals
3. Flag as anomaly if actual > upper confidence bound
```

**Use Case:**
- Latency with strong temporal patterns
- Detecting gradual degradation
- Predicting future latency issues

**Prophet (Facebook):**
```
Components:
- Trend: piecewise linear or logistic
- Seasonality: Fourier series (daily, weekly patterns)
- Holidays/Events: user-defined events

Anomaly Detection:
1. Fit Prophet model on 30-90 days of data
2. Generate forecast with uncertainty intervals
3. Flag points outside 95% prediction interval
```

**Advantages:**
- Handles missing data and outliers
- Automatic seasonality detection
- Interpretable components
- Works well with irregular time series

**LSTM Autoencoders:**
```
Architecture:
Encoder: LSTM layers (128 → 64 → 32 units)
Latent: Bottleneck representation
Decoder: LSTM layers (32 → 64 → 128 units)

Training:
1. Train on normal latency sequences
2. Input: sliding window of latency (e.g., last 100 requests)
3. Learn to reconstruct normal patterns

Detection:
reconstruction_error = MSE(actual, reconstructed)
threshold = mean_error + 3 × std_error
anomaly if: reconstruction_error > threshold
```

**Best for:**
- Complex temporal dependencies
- Multivariate latency patterns
- Learning seasonal and cyclical patterns
- Real-time streaming data

**Hybrid Approach (LSTM + Isolation Forest):**
```
1. Use Isolation Forest to label historical data
2. Train LSTM autoencoder on normal sequences
3. Combine reconstruction error + isolation score
4. Dynamic threshold adjustment based on patterns
```

### 2.2 Throughput Degradation

**Metrics to Monitor:**
- Requests per second (RPS)
- Tokens per second (TPS)
- Successful completions per minute
- Error rate

**Detection Method:**
```
Moving Average with Control Limits:

1. Calculate 7-day moving average of throughput
2. Calculate standard deviation over same period
3. Set control limits: μ ± 2σ
4. Alert if throughput drops below lower control limit for >5 minutes

Adaptive approach:
- Update baselines weekly
- Separate metrics by time-of-day
- Account for known traffic patterns
```

### 2.3 Queue Depth Anomalies

**Monitoring Approach:**
```
Queue Metrics:
- Current queue size
- Queue wait time (p50, p95, p99)
- Queue growth rate
- Queue clear time

Anomaly Conditions:
1. Queue size > 2x normal peak
2. Wait time > 5x median
3. Sustained growth rate > 10 requests/sec
4. Queue clear time > 2x normal
```

**Predictive Alerting:**
```
Use exponential smoothing:
S(t) = α × X(t) + (1-α) × S(t-1)

where:
  α = 0.2-0.3 (smoothing factor)
  X(t) = current queue size
  S(t) = smoothed estimate

Alert if: S(t) trajectory indicates queue will exceed capacity in <5 min
```

### 2.4 Token Generation Rate Changes

**Metric:**
```
Token Generation Rate = output_tokens / generation_time (tokens/sec)
```

**Detection:**
```
Baseline: Calculate p50, p95 over 7 days
Segment by:
- Model version
- Prompt length category
- Output length range

Alert if:
- p50 drops >20% from baseline
- p95 drops >30% from baseline
- Sustained decrease for >10 minutes
```

---

## 3. Hallucination Detection

Hallucination detection is critical for production LLM systems. Research in 2024 has produced methods achieving up to 450x speedup while maintaining accuracy.

### 3.1 LLM-Check Method (NeurIPS 2024)

**Approach:** Analyze internal model representations in a single forward pass.

**Algorithm:**
```
Components:
1. Attention Map Analysis
   - Extract attention weights from all layers
   - Identify low-attention tokens (hallucination candidates)
   - Threshold: attention weight < 0.1

2. Hidden Activation Patterns
   - Monitor activation magnitudes in middle layers
   - Hallucinations show distinct activation signatures
   - Use dimensionality reduction to identify patterns

3. Output Probability Distribution
   - Track token-level probabilities
   - Low probability tokens = potential hallucination
   - Entropy of distribution indicates uncertainty

Hallucination Score:
score = w1×attention_score + w2×activation_score + w3×probability_score
where w1, w2, w3 are learned weights

Threshold: score > 0.7 indicates hallucination
```

**Performance:**
- Up to 450x faster than multi-inference methods
- Suitable for real-time applications
- No additional inference overhead

### 3.2 Self-Consistency Checking

**Multi-Sampling Approach:**
```
Algorithm:
1. Generate N responses (N=3-5) for same prompt with temperature 0.7-0.9
2. Calculate pairwise semantic similarity using embeddings
3. Aggregate consistency score

Consistency Score = average(cosine_similarity(response_i, response_j))
                    for all pairs i,j

Interpretation:
- Score > 0.90: High consistency, likely factual
- 0.70 < Score < 0.90: Moderate consistency
- Score < 0.70: Low consistency, likely hallucination
```

**Cost Optimization:**
- Use smaller model for consistency checks
- Sample only 2-3 responses for non-critical queries
- Cache frequent queries

### 3.3 Retrieval-Based Verification

**RAG-Enhanced Validation:**
```
Algorithm:
1. Extract factual claims from LLM output
2. Retrieve relevant documents from knowledge base
3. Calculate entailment score between claim and evidence

Entailment Score:
- Use cross-encoder model (e.g., DeBERTa)
- Score range: 0-1
- Threshold: >0.75 for factual support

Hallucination Detection:
if claim_score < threshold:
    flag as potential hallucination
    request evidence citation
```

### 3.4 Perplexity-Based Detection

**Method:**
```
Calculate perplexity for generated output:

PPL = exp(-1/N × Σ log P(token_i | context))

Baseline: Establish normal perplexity range from validated outputs
Anomaly Detection:
- High perplexity (>2x baseline): Model uncertain, possible hallucination
- Extremely low perplexity (<0.5x baseline): Memorization, check for training data leakage

Track per-sentence perplexity:
Sentences with PPL > threshold → hallucination candidates
```

### 3.5 Factuality Scoring

**Multi-Dimension Approach:**
```
Factuality Score = weighted_average(
    attribution_score,    # Can claims be attributed to sources?
    consistency_score,    # Internal logical consistency?
    uncertainty_score,    # Model confidence in statements?
    retrieval_score       # Match with ground truth data?
)

Weights:
- Attribution: 0.35
- Consistency: 0.25
- Uncertainty: 0.20
- Retrieval: 0.20

Threshold: Score < 0.65 → flag for review
```

### 3.6 Uncertainty Quantification

**Semantic Entropy:**
```
1. Generate multiple outputs with sampling
2. Cluster semantically similar outputs
3. Calculate entropy over cluster distribution

High entropy = high uncertainty = potential hallucination

Entropy = -Σ p(cluster_i) × log(p(cluster_i))

Threshold: Entropy > 1.5 → high uncertainty
```

### 3.7 Real-Time Production Monitoring

**Datadog LLM Observability Approach:**
```
Metrics Dashboard:
1. Total hallucinations count (trend over time)
2. Hallucination rate by prompt category
3. Hallucination severity distribution
4. Time-to-detection metrics

Alerting Rules:
- Hallucination rate > 5% over 1 hour
- Critical hallucinations (factuality < 0.3) detected
- Sudden spike: >3x normal rate in 15 minutes
```

---

## 4. Cost Anomalies

Cost monitoring is essential for preventing budget overruns. Organizations in 2024 reported identifying cost anomalies representing 20-40% of total LLM expenses through proper monitoring.

### 4.1 Token Usage Spikes

#### **Statistical Detection**

**Change Point Detection:**
```
Algorithm: CUSUM (Cumulative Sum)

S(t) = max(0, S(t-1) + (X(t) - μ - k))

where:
  X(t) = current token usage
  μ = baseline mean
  k = slack parameter (0.5σ)

Alert if: S(t) > h (threshold, typically 4-5σ)
```

**Advantages:**
- Detects gradual and sudden changes
- Low false positive rate
- Adapts to slowly changing baseline

**Bayesian Change Point Detection:**
```
1. Model token usage as piecewise stationary process
2. Calculate posterior probability of change point at each time
3. Alert if P(change point) > 0.8
```

**Best for:**
- Detecting when usage patterns fundamentally change
- Identifying new high-cost user segments

#### **Time-Series Forecasting**

**Prophet for Cost Prediction:**
```
Model Components:
1. Trend: Long-term cost trajectory
2. Weekly seasonality: Weekday vs weekend patterns
3. Daily seasonality: Peak hours
4. Holiday effects: Known low-usage dates

Anomaly Detection:
forecast = prophet_model.predict(current_date)
upper_bound = forecast.yhat_upper
actual_cost = current_token_usage × cost_per_token

if actual_cost > upper_bound:
    severity = (actual_cost - upper_bound) / upper_bound
    if severity > 0.5:
        CRITICAL_ALERT
    elif severity > 0.2:
        WARNING_ALERT
```

### 4.2 API Call Pattern Anomalies

#### **Frequency Analysis**

**Time-Series Clustering:**
```
Algorithm:
1. Extract hourly API call counts
2. Create feature vectors: [calls_h0, calls_h1, ..., calls_h23]
3. Cluster using K-means or DBSCAN
4. Identify normal vs anomalous patterns

Normal Patterns:
- Business hours spike (cluster 1)
- Batch processing pattern (cluster 2)
- Steady baseline (cluster 3)

Anomalies:
- Points far from all cluster centroids
- New patterns not seen in training
```

**Sequential Pattern Mining:**
```
Detect unusual call sequences:

1. Track API call sequences: [endpoint1, endpoint2, endpoint3, ...]
2. Build Markov chain of normal transitions
3. Calculate transition probability for new sequences

Anomaly Score = -log P(sequence | normal patterns)

Alert if: score > threshold (e.g., 3)
```

#### **Expensive Prompt Detection**

**Cost per Request Monitoring:**
```
Metrics:
1. Input tokens per request
2. Output tokens per request
3. Total cost per request

Cost = (input_tokens × input_price) + (output_tokens × output_price)

Detection:
baseline_p95 = 95th percentile cost over 7 days
current_cost = request cost

if current_cost > 3 × baseline_p95:
    flag_expensive_request()
    log_prompt_pattern()

Trace Analysis:
- Identify prompt templates generating high costs
- User/application causing cost spikes
- Inefficient prompting patterns
```

**Opik Cost Tracking Approach:**
```
Trace-Level Metrics:
- Input token count
- Output token count
- Cost per trace
- Duration
- Quality checks triggered

Dashboard Views:
1. Cost by project
2. Cost by user
3. Cost by model
4. Cost trend over time
5. High-cost trace explorer

Filters for Investigation:
- cost > $0.50
- duration > 30s
- model = "gpt-4"
- date_range = last_24h
```

### 4.3 Resource Consumption Outliers

**Multi-Dimensional Anomaly Detection:**
```
Features:
- Token usage
- Request count
- Compute time
- Memory usage
- API bandwidth

Algorithm: Isolation Forest
1. Train on historical resource metrics
2. Contamination = 0.03 (3% expected anomalies)
3. Anomaly score per request

High anomaly score indicates:
- Unusual resource combination
- Inefficient processing
- Potential abuse or attack
```

### 4.4 Budget Threshold Violations

**Predictive Alerting:**
```
Daily Budget Monitoring:

current_spend = sum(costs_today)
burn_rate = current_spend / hours_elapsed_today
projected_daily_spend = burn_rate × 24

monthly_budget = $50,000
daily_budget = monthly_budget / 30

Alerts:
1. if projected_daily_spend > 1.2 × daily_budget:
       WARNING: "On track to exceed daily budget by 20%"

2. if current_spend > daily_budget and time < 18:00:
       CRITICAL: "Daily budget exceeded with 6+ hours remaining"

3. monthly_burn_rate = sum(costs_this_month) / days_elapsed
   projected_monthly = monthly_burn_rate × 30
   if projected_monthly > monthly_budget:
       WARNING: "Projected to exceed monthly budget"
```

**Rate Limiting by Cost:**
```
Adaptive Rate Limiting:

1. Set cost-based quotas:
   - Per user: $100/day
   - Per application: $500/day
   - Global: $5000/day

2. Track running totals with sliding windows

3. When approaching limit:
   - 80% quota: Notify user
   - 90% quota: Reduce priority, add latency
   - 100% quota: Reject new requests (with 429 status)

4. Reset quotas at defined intervals (hourly/daily)
```

### 4.5 Model-Specific Cost Analysis

**Cost Breakdown by Model:**
```
Metrics per Model:
- Average cost per request
- Total requests
- Total cost
- Cost efficiency (task_success_rate / avg_cost)

Optimization Opportunities:
1. Identify expensive models with low success rates
2. Find cheaper model alternatives for specific tasks
3. Route simple queries to smaller models

Example Decision Tree:
if task_complexity < 0.3:
    use gpt-3.5-turbo  # $0.002/1K tokens
elif task_complexity < 0.7:
    use gpt-4-turbo    # $0.01/1K tokens
else:
    use gpt-4          # $0.03/1K tokens
```

---

## 5. Quality Degradation

Quality degradation can manifest as gradual decline that might go unnoticed until user satisfaction plummets. Continuous monitoring enables detection before user impact.

### 5.1 Output Coherence Metrics

#### **Semantic Consistency**

**Embedding-Based Coherence:**
```
Algorithm:
1. Split output into sentences/paragraphs
2. Generate embeddings for each segment (e.g., sentence-transformers)
3. Calculate pairwise cosine similarity between adjacent segments
4. Aggregate coherence score

Coherence Score = average(similarity(segment_i, segment_{i+1}))

Baseline: Establish from validated high-quality outputs
Alert if: coherence < 0.75 (configurable threshold)
```

**Implementation:**
```python
from sentence_transformers import SentenceTransformer

model = SentenceTransformer('all-mpnet-base-v2')

def calculate_coherence(text):
    sentences = split_into_sentences(text)
    embeddings = model.encode(sentences)

    similarities = []
    for i in range(len(embeddings) - 1):
        sim = cosine_similarity(embeddings[i], embeddings[i+1])
        similarities.append(sim)

    return np.mean(similarities)

threshold = 0.75
if calculate_coherence(llm_output) < threshold:
    flag_low_coherence()
```

#### **Logical Flow Analysis**

**Discourse Coherence:**
```
Metrics:
1. Entity consistency: Track entity references across text
2. Temporal consistency: Verify time references are logical
3. Causal consistency: Check cause-effect relationships

Detection:
- Entity re-introduction without context: score penalty
- Contradictory temporal statements: flag inconsistency
- Reversed causality: flag logical error
```

### 5.2 Task Performance Drift

#### **Success Rate Monitoring**

**Task Completion Metrics:**
```
Per Task Category:
- Completion rate
- Accuracy (if ground truth available)
- User satisfaction score
- Retry rate
- Error rate

Drift Detection:
baseline_success_rate = historical_average(last_30_days)
current_success_rate = moving_average(last_24_hours)

degradation = (baseline_success_rate - current_success_rate) / baseline_success_rate

Alerts:
- degradation > 0.10: WARNING (10% decrease)
- degradation > 0.25: CRITICAL (25% decrease)
```

#### **Comparative A/B Analysis**

**Shadow Evaluation:**
```
1. Maintain reference model or curated test set
2. Periodically run production model on reference inputs
3. Compare outputs with expected results

Metrics:
- BLEU score (for text generation)
- ROUGE score (for summarization)
- Exact match (for Q&A)
- Semantic similarity

Degradation Detection:
if current_score < baseline_score - 0.05:
    investigate_quality_degradation()
```

### 5.3 User Feedback Patterns

#### **Explicit Feedback Monitoring**

**Feedback Metrics:**
```
Sources:
- Thumbs up/down ratings
- Star ratings (1-5)
- Regeneration requests
- Edit/modification rate
- Abandonment rate

Aggregation:
satisfaction_score = (positive_feedback - negative_feedback) / total_feedback

Trend Analysis:
7_day_avg = rolling_average(satisfaction_score, window=7)
30_day_avg = rolling_average(satisfaction_score, window=30)

if 7_day_avg < 30_day_avg - 0.15:
    ALERT: "User satisfaction declining"
```

#### **Implicit Feedback Signals**

**Behavioral Analysis:**
```
Negative Signals:
- Session abandonment: User exits without completion
- Multiple regenerations: User unsatisfied with outputs
- Long edit time: Output requires substantial modification
- Low engagement: Short time spent with output

Positive Signals:
- Content sharing: User shares output
- Follow-up queries: User continues conversation
- Copy/paste action: User uses output
- High dwell time: User thoroughly reviews output

Composite Quality Score:
quality_score = w1×explicit_feedback + w2×implicit_signals + w3×task_completion
```

### 5.4 Relevance and Accuracy Scoring

#### **Automated Evaluation**

**LLM-as-Judge:**
```
Method:
1. Use evaluation LLM to score outputs
2. Provide scoring rubric and criteria
3. Generate scores (0-10 or 1-5)

Prompt Template:
"Evaluate the following response for:
1. Relevance to the question
2. Factual accuracy
3. Completeness
4. Clarity

Question: {question}
Response: {llm_output}

Provide scores 1-5 for each criterion and explain."

Aggregate: overall_score = average(relevance, accuracy, completeness, clarity)

Track over time:
if overall_score < 3.5:
    flag_low_quality()
```

#### **Retrieval-Augmented Verification**

**Context Relevance:**
```
For RAG Systems:

1. Measure answer-context relevance:
   relevance = cosine_similarity(answer_embedding, context_embedding)

2. Measure answer-question relevance:
   relevance = cosine_similarity(answer_embedding, question_embedding)

3. Combine scores:
   final_score = 0.6 × context_relevance + 0.4 × question_relevance

Threshold: score < 0.70 indicates poor relevance
```

### 5.5 Toxicity and Safety Monitoring

**Content Safety Checks:**
```
Dimensions:
- Toxicity
- Profanity
- Personal attacks
- Identity hate
- Sexual content
- Violence

Detection Methods:
1. Use pre-trained classifiers (e.g., Perspective API, Detoxify)
2. Score on 0-1 scale
3. Set thresholds per dimension

Guardrails:
if toxicity_score > 0.7:
    block_output()
    regenerate_with_safety_prompt()

if toxicity_score > 0.4:
    flag_for_review()
    log_incident()
```

### 5.6 Consistency Across Versions

**Model Version Monitoring:**
```
When model is updated (e.g., GPT-4 alias auto-updates):

1. Run regression test suite:
   - 100-500 diverse test prompts
   - Known good outputs as reference

2. Compare metrics:
   - Output similarity to reference
   - Task success rate
   - Latency changes
   - Cost changes

3. Alert if:
   similarity < 0.85  # Significant output change
   success_rate_delta < -0.10  # 10% drop in success
   latency_increase > 0.30  # 30% slower
   cost_increase > 0.20  # 20% more expensive
```

---

## 6. Real-Time vs Batch Processing

The choice between real-time and batch processing depends on latency requirements, data volume, and resource constraints.

### 6.1 Real-Time Streaming Analytics

**Architecture: Kafka + Flink**

**Data Flow:**
```
LLM Inference → Kafka Topic (raw events)
                    ↓
           Apache Flink Processing
                    ↓
    Anomaly Detection Algorithms
                    ↓
        Kafka Topic (anomalies)
                    ↓
    Alert Handler + Monitoring Dashboard
```

**Flink Implementation:**
```
Components:
1. Stream Ingestion:
   - Consume from Kafka topic
   - Parse event data (latency, tokens, cost, quality metrics)

2. Windowing:
   - Tumbling windows: Fixed-size, non-overlapping (e.g., 1 minute)
   - Sliding windows: Overlapping (e.g., 5 min window, 1 min slide)
   - Session windows: Based on activity gaps

3. State Management:
   - Maintain running statistics (mean, std, quantiles)
   - Store baseline distributions
   - Track user/session state

4. Anomaly Detection:
   - Apply detection algorithms in real-time
   - Calculate anomaly scores
   - Trigger alerts

5. Output:
   - Write anomalies to Kafka topic
   - Update metrics in time-series DB
   - Trigger Lambda for notifications
```

**Example Flink Job:**
```java
DataStream<Event> events = env.addSource(new FlinkKafkaConsumer<>(...));

DataStream<Anomaly> anomalies = events
    .keyBy(event -> event.getUserId())
    .window(TumblingEventTimeWindows.of(Time.minutes(1)))
    .apply(new AnomalyDetectionFunction())
    .filter(result -> result.isAnomaly());

anomalies.addSink(new FlinkKafkaProducer<>(...));
```

**Algorithms for Real-Time:**

**1. Online Learning with Random Cut Forest (RCF):**
```
AWS Kinesis Analytics RCF:
- Incrementally updated with each data point
- No retraining required
- Anomaly score in real-time
- Handles concept drift automatically

Parameters:
- shingleSize: 10-50 (sequence length)
- numberOfTrees: 50-100
- sampleSize: 256
```

**2. Streaming Z-Score:**
```
Welford's online algorithm for variance:

mean_n = mean_{n-1} + (x_n - mean_{n-1}) / n
M2_n = M2_{n-1} + (x_n - mean_{n-1}) × (x_n - mean_n)
variance_n = M2_n / (n - 1)

z_score = (x_n - mean_n) / sqrt(variance_n)
```

**3. Streaming Quantiles (t-digest):**
```
Approximate percentiles with bounded memory:
- Maintains compressed distribution
- Updates incrementally
- Queries p50, p95, p99 in O(1)

Use for:
- Real-time latency percentile monitoring
- Dynamic threshold calculation
```

**Performance Characteristics:**
- Latency: <100ms for detection
- Throughput: 10K-100K events/sec per Flink task
- State size: 10-100 MB per key
- Scalability: Horizontal (add Flink workers)

### 6.2 Batch Analytics

**Architecture: Scheduled Jobs**

**Data Flow:**
```
LLM Logs → Data Lake (S3/GCS)
              ↓
    ETL Pipeline (hourly/daily)
              ↓
     Data Warehouse (Snowflake/BigQuery)
              ↓
    Batch Anomaly Detection
              ↓
    Reports + Alerts
```

**Processing Schedule:**
- Hourly: Cost analysis, high-level metrics
- Daily: Detailed quality analysis, drift detection
- Weekly: Model performance reports, trend analysis

**Algorithms for Batch:**

**1. Isolation Forest (Batch):**
```python
from sklearn.ensemble import IsolationForest

# Load daily batch of metrics
df = load_metrics(date='2025-11-06')
features = ['latency', 'token_count', 'cost', 'quality_score']

clf = IsolationForest(
    n_estimators=100,
    contamination=0.05,
    random_state=42
)
clf.fit(df[features])

anomalies = clf.predict(df[features])
anomaly_scores = clf.score_samples(df[features])

# Flag top anomalies for investigation
```

**2. ARIMA/Prophet:**
```python
from prophet import Prophet

# Aggregate metrics to daily
df_daily = aggregate_daily(df)

model = Prophet(
    seasonality_mode='multiplicative',
    yearly_seasonality=True,
    weekly_seasonality=True,
    daily_seasonality=False
)
model.fit(df_daily)

# Forecast next 7 days
future = model.make_future_dataframe(periods=7)
forecast = model.predict(future)

# Detect anomalies (actual outside prediction interval)
anomalies = df_daily[
    (df_daily['y'] < forecast['yhat_lower']) |
    (df_daily['y'] > forecast['yhat_upper'])
]
```

**3. LSTM Autoencoder (Batch Training):**
```python
from tensorflow.keras.models import Sequential
from tensorflow.keras.layers import LSTM, Dense, RepeatVector

# Train weekly on normal data
sequence_length = 100
n_features = 5

model = Sequential([
    LSTM(128, activation='relu', input_shape=(sequence_length, n_features)),
    RepeatVector(sequence_length),
    LSTM(128, activation='relu', return_sequences=True),
    LSTM(64, activation='relu', return_sequences=True),
    Dense(n_features)
])

model.compile(optimizer='adam', loss='mse')
model.fit(normal_sequences, normal_sequences, epochs=50, batch_size=32)

# Detect anomalies in new batch
reconstructions = model.predict(new_sequences)
mse = np.mean(np.square(new_sequences - reconstructions), axis=(1, 2))
threshold = np.percentile(mse, 95)
anomalies = new_sequences[mse > threshold]
```

**Advantages of Batch:**
- Complex algorithms (deep learning, ensemble methods)
- Full historical context for training
- Computationally intensive analysis
- Comprehensive reports and visualizations

**Disadvantages:**
- Delayed detection (hours to days)
- Cannot prevent issues in real-time
- Higher latency for alerts

### 6.3 Hybrid Approach (Recommended)

**Lambda Architecture:**
```
Real-Time Layer (Speed):
- Fast, simple anomaly detection
- Critical alerts (SLA violations, security issues)
- Streaming analytics with Kafka/Flink
- Algorithms: Z-score, simple thresholds, online RCF

Batch Layer (Accuracy):
- Complex ML models
- Comprehensive analysis
- Drift detection, trend analysis
- Algorithms: Isolation Forest, LSTM, Prophet

Serving Layer:
- Merge real-time + batch results
- Unified anomaly dashboard
- Context-aware alerting
```

**Example Workflow:**
```
1. Real-Time (Flink):
   - Detect latency spikes (Z-score > 3) → immediate alert
   - Flag high-cost requests → rate limiting
   - Identify toxic outputs → block generation

2. Batch (Daily):
   - Analyze drift over 24 hours → trend reports
   - Retrain models on new data → update baselines
   - Generate quality scorecards → stakeholder reports

3. Integration:
   - Real-time alerts reference batch context
   - Batch analysis validates real-time detections
   - Reduce false positives through cross-validation
```

---

## 7. Threshold Tuning Strategies

Threshold tuning balances sensitivity (detecting real anomalies) with specificity (avoiding false positives). The optimal threshold depends on business context and cost of errors.

### 7.1 Statistical Approach

#### **Standard Deviation Multipliers**

**Z-Score Thresholds:**
```
Common Values:
- 2σ: ~95% coverage, higher sensitivity, more false positives
- 3σ: ~99.7% coverage, balanced approach (recommended starting point)
- 4σ: ~99.99% coverage, lower sensitivity, fewer false positives

Selection Criteria:
- Critical systems (safety, security): 2σ
- General monitoring: 3σ
- Low-noise metrics: 4σ
```

**Adaptive Thresholds:**
```
Algorithm:
1. Calculate baseline statistics over training period (30-90 days)
2. Set initial threshold: μ + k×σ
3. Monitor false positive rate over validation period (7-14 days)
4. Adjust k based on feedback:

   if false_positive_rate > target_rate:
       k = k + 0.5  # Increase threshold (less sensitive)
   elif false_positive_rate < target_rate × 0.5:
       k = k - 0.5  # Decrease threshold (more sensitive)

5. Iterate until converged
```

#### **Percentile-Based Thresholds**

**IQR Multipliers:**
```
Standard: Upper bound = Q3 + 1.5×IQR
Relaxed:  Upper bound = Q3 + 2.0×IQR (fewer alerts)
Strict:   Upper bound = Q3 + 1.0×IQR (more alerts)

Tuning Process:
1. Calculate IQR on training data
2. Set multiplier = 1.5 (default)
3. Measure precision and recall on validation set
4. Adjust multiplier to achieve target metrics
```

**Direct Percentile Thresholds:**
```
Approach: Use high percentiles as thresholds
- p95: Flag top 5% as anomalies
- p99: Flag top 1% as anomalies
- p99.9: Flag top 0.1% as anomalies

Selection:
- p95: High-traffic systems (more alerts acceptable)
- p99: Standard systems
- p99.9: Critical systems (only severe anomalies)
```

### 7.2 Machine Learning Approach

#### **Contamination Parameter Tuning**

**Isolation Forest & One-Class SVM:**
```
contamination = expected proportion of anomalies

Tuning Strategy:
1. Initial estimate based on domain knowledge:
   - Well-functioning system: 0.01-0.02 (1-2%)
   - Noisy system: 0.05-0.10 (5-10%)

2. Validation approach:
   a. Label a sample of data (manual review)
   b. Train models with different contamination values
   c. Evaluate precision/recall
   d. Select contamination maximizing F1 score

3. Example results:
   contamination=0.01: Precision=0.92, Recall=0.45, F1=0.60
   contamination=0.05: Precision=0.78, Recall=0.72, F1=0.75 ← Best
   contamination=0.10: Precision=0.65, Recall=0.85, F1=0.74
```

#### **Reconstruction Error Thresholds (Autoencoders)**

**Threshold Selection:**
```
Method 1: Statistical
threshold = mean_reconstruction_error + k × std_reconstruction_error
where k ∈ [2, 3, 4]

Method 2: Percentile
threshold = percentile(reconstruction_errors, p)
where p ∈ [95, 99, 99.5]

Method 3: Precision-Recall Trade-off
1. Generate predictions for range of thresholds
2. Calculate precision and recall for each
3. Plot precision-recall curve
4. Select threshold at desired operating point
   - High precision: threshold at p99.5
   - Balanced: threshold at F1 maximum
   - High recall: threshold at p95
```

**Dynamic Threshold Updates:**
```
Sliding Window Approach:
1. Maintain rolling window of reconstruction errors (e.g., 10,000 samples)
2. Recalculate threshold daily/weekly
3. Smooth threshold changes to avoid instability

threshold(t) = 0.8 × threshold(t-1) + 0.2 × new_threshold

Benefits:
- Adapts to concept drift
- Maintains consistent false positive rate
- Handles seasonality
```

### 7.3 Business Context-Driven Tuning

#### **Cost-Sensitive Thresholds**

**Framework:**
```
Total Cost = (FP_count × FP_cost) + (FN_count × FN_cost)

where:
  FP_cost = cost of investigating false positive
  FN_cost = cost of missing true anomaly

Example:
- FP_cost = $5 (engineer time)
- FN_cost = $5000 (service outage)

Optimal threshold minimizes total cost.

Algorithm:
1. For each threshold candidate:
   - Calculate FP_count and FN_count on validation set
   - Calculate total_cost
2. Select threshold with minimum total_cost
```

#### **SLA-Based Thresholds**

**Latency Example:**
```
SLA: 95% of requests < 500ms

Threshold Setting:
- Primary threshold: 500ms (SLA boundary)
- Warning threshold: 400ms (80% of SLA)
- Critical threshold: 600ms (20% over SLA)

Alert Levels:
- Latency > 400ms for 5 min: Warning
- Latency > 500ms for 1 min: SLA violation (alert on-call)
- Latency > 600ms: Critical (page immediately)
```

### 7.4 Multi-Threshold Strategies

**Severity Levels:**
```
Define multiple thresholds for graduated response:

INFO:     1.5σ < x < 2σ   (log for analysis)
WARNING:  2σ < x < 3σ     (notify team channel)
ALERT:    3σ < x < 4σ     (page on-call)
CRITICAL: x > 4σ           (escalate to senior engineer)

Benefits:
- Contextual responses
- Reduces alert fatigue
- Prioritizes investigations
```

**Composite Scoring:**
```
Combine multiple signals with different thresholds:

anomaly_score = w1×latency_z_score + w2×cost_z_score + w3×quality_z_score

Thresholds:
- score > 2: Monitor
- score > 5: Warning
- score > 10: Alert

Weights based on importance:
w1=0.4 (latency), w2=0.3 (cost), w3=0.3 (quality)
```

### 7.5 Continuous Threshold Optimization

#### **A/B Testing Thresholds**

**Methodology:**
```
1. Split traffic 50/50:
   - Group A: current threshold
   - Group B: candidate threshold

2. Run for 7-14 days

3. Measure:
   - Alert count
   - Precision (% true positives)
   - Recall (% anomalies caught)
   - Time to detect
   - False positive rate

4. Statistical significance test (t-test)

5. Adopt better threshold
```

#### **Feedback Loop Integration**

**Human-in-the-Loop:**
```
Process:
1. Anomaly detected and flagged
2. Engineer investigates and labels:
   - True positive
   - False positive
   - Unclear

3. Store labeled examples in database

4. Weekly/monthly retuning:
   a. Retrieve labeled data
   b. Retrain models or recalculate thresholds
   c. Evaluate on holdout set
   d. Deploy updated thresholds

5. Track threshold evolution over time
```

#### **Automated Tuning with Bayesian Optimization**

**Approach:**
```python
from skopt import gp_minimize
from skopt.space import Real

def objective(params):
    threshold = params[0]

    # Apply threshold to validation set
    predictions = apply_threshold(validation_set, threshold)

    # Calculate F1 score (or custom metric)
    precision = true_positives / (true_positives + false_positives)
    recall = true_positives / (true_positives + false_negatives)
    f1 = 2 × precision × recall / (precision + recall)

    return -f1  # Minimize negative F1 (maximize F1)

space = [Real(0.01, 0.99, name='threshold')]

result = gp_minimize(
    objective,
    space,
    n_calls=50,
    random_state=42
)

optimal_threshold = result.x[0]
```

---

## 8. False Positive Mitigation

False positives are the primary cause of alert fatigue. Effective mitigation requires multiple strategies.

### 8.1 Statistical Techniques

#### **Moving Averages and Smoothing**

**Exponential Smoothing:**
```
Smoothed value: S(t) = α×X(t) + (1-α)×S(t-1)

where:
  α = smoothing factor (0.1-0.3 for noise reduction)
  X(t) = raw observation
  S(t) = smoothed value

Apply threshold to S(t) instead of X(t)
```

**Benefits:**
- Reduces noise-induced spikes
- Smooths transient fluctuations
- Preserves true anomalies (sustained deviations)

**Moving Median:**
```
For highly noisy data:
median_value = median(X[t-k:t])

Anomaly if: |X(t) - median_value| > threshold

More robust than moving average to outliers
```

#### **Persistence Filters**

**Consecutive Violations:**
```
Rule: Trigger alert only if threshold violated N consecutive times

Example:
threshold = 3σ
persistence = 3

violations = 0
for each observation:
    if observation > threshold:
        violations += 1
        if violations >= persistence:
            ALERT()
    else:
        violations = 0  # Reset counter
```

**Recommended Values:**
- Fast metrics (latency): persistence=2-3
- Slow metrics (drift): persistence=5-10

**Time-Based Persistence:**
```
Rule: Anomaly must persist for T seconds

Example:
threshold = 500ms
persistence_time = 5 minutes

if metric > threshold:
    start_timer()
    if elapsed_time > persistence_time:
        ALERT()
else:
    reset_timer()
```

### 8.2 Contextual Anomaly Detection

#### **Segmentation**

**Stratify by Context:**
```
Instead of global threshold, use context-specific thresholds:

Segments:
- Time of day: [midnight-6am, 6am-noon, noon-6pm, 6pm-midnight]
- Day of week: [weekday, weekend]
- User type: [free, premium, enterprise]
- Model version: [gpt-3.5, gpt-4, gpt-4-turbo]
- Query type: [summarization, QA, generation, translation]

For each segment:
  calculate separate baselines
  apply separate thresholds

Example:
  weekend_threshold = μ_weekend + 3×σ_weekend
  weekday_threshold = μ_weekday + 3×σ_weekday
```

**Benefits:**
- Reduces false positives from expected variations
- Captures anomalies relative to normal context
- Better handles seasonality and patterns

#### **Multi-Dimensional Context**

**Conditional Anomalies:**
```
Example: Latency anomaly detection

Context features:
- Input token count
- Output token count
- Model type
- Time of day

Model:
Train regression model: latency ~ f(context_features)

Anomaly detection:
predicted_latency = model.predict(context)
residual = actual_latency - predicted_latency
z_score = residual / std_residual

Alert if: |z_score| > 3

This accounts for expected latency variations based on context.
```

### 8.3 Ensemble Methods

#### **Voting Mechanisms**

**Multiple Detector Agreement:**
```
Detectors:
1. Z-score (threshold=3)
2. IQR (multiplier=1.5)
3. Isolation Forest (contamination=0.05)

Voting:
- Unanimous: All 3 flag anomaly → ALERT
- Majority: 2+ flag anomaly → WARNING
- Any: 1+ flag anomaly → LOG

Reduces false positives while maintaining sensitivity
```

**Weighted Voting:**
```
anomaly_score = w1×detector1_score + w2×detector2_score + w3×detector3_score

Weights based on historical precision:
- High precision detector: higher weight
- Noisy detector: lower weight

Threshold: anomaly_score > 0.7
```

### 8.4 Root Cause Correlation

#### **Multi-Metric Correlation**

**Correlated Anomaly Detection:**
```
Principle: True anomalies often manifest in multiple metrics

Example:
if latency_anomaly AND (cost_anomaly OR quality_anomaly):
    HIGH_CONFIDENCE_ALERT()
elif latency_anomaly:
    LOW_CONFIDENCE_ALERT()

Correlation Rules:
- Latency spike + high cost → resource contention (true positive)
- Latency spike only → network jitter (possible false positive)
- Quality drop + hallucination increase → model degradation (true positive)
```

#### **Dependency Graph Analysis**

**Infrastructure Correlation:**
```
Build dependency graph:
LLM Service → API Gateway → Load Balancer → Database

Anomaly in LLM latency?
  Check:
  - API Gateway latency (correlation expected)
  - Database latency (correlation expected)
  - Network metrics (correlation expected)

If all show anomalies: Likely true positive (infrastructure issue)
If only LLM anomalous: Investigate model-specific issue or false positive
```

### 8.5 Machine Learning for FP Reduction

#### **Anomaly Classifier**

**Two-Stage Detection:**
```
Stage 1: Statistical detector (high recall, moderate precision)
  → Flags potential anomalies

Stage 2: ML classifier (high precision)
  → Filters false positives

Classifier Features:
- Anomaly score from stage 1
- Time of day, day of week
- Recent metric trends
- Correlated metric states
- Historical patterns

Training:
- Labeled data: true positives vs false positives
- Model: Random Forest, XGBoost
- Objective: Maximize precision while maintaining recall

Deployment:
if stage1_detector.is_anomaly(observation):
    if stage2_classifier.predict(features) == TRUE_POSITIVE:
        ALERT()
```

#### **Autoencoders for Normal Pattern Learning**

**Approach:**
```
1. Train autoencoder on normal data only
2. Learn compressed representation of normal patterns
3. Calculate reconstruction error for new data

Key Insight:
- Normal data: low reconstruction error
- True anomalies: high reconstruction error
- False positive patterns: moderate reconstruction error (seen during training)

Threshold Selection:
- Conservative: p99.5 of training reconstruction errors
- Reduces false positives from known edge cases
```

### 8.6 Alert Aggregation and Suppression

#### **Time-Based Aggregation**

**Debouncing:**
```
Rule: Send at most 1 alert per anomaly every T minutes

Example:
min_alert_interval = 15 minutes
last_alert_time = None

if anomaly_detected:
    if last_alert_time is None or (current_time - last_alert_time) > min_alert_interval:
        SEND_ALERT()
        last_alert_time = current_time
    else:
        LOG_SUPPRESSED_ALERT()
```

#### **Severity-Based Suppression**

**Suppress Low-Severity During High-Severity:**
```
Active Alerts:
- Critical latency alert (severity=CRITICAL)
- Minor cost anomaly (severity=WARNING)

Rule: Suppress WARNING alerts when CRITICAL alert active for same service

Rationale: Focus on critical issues first, reduce noise
```

#### **Grouped Alerting**

**Batch Similar Anomalies:**
```
Window: 5 minutes

Anomalies detected:
- Latency anomaly at 10:00:00
- Latency anomaly at 10:01:30
- Latency anomaly at 10:03:00

Alert: "3 latency anomalies detected between 10:00-10:05"

Instead of 3 separate alerts → 1 grouped alert
```

### 8.7 Feedback Mechanisms

#### **Alert Review Process**

**Workflow:**
```
1. Alert triggered → Sent to engineer
2. Engineer investigates
3. Engineer labels:
   - TRUE_POSITIVE: "Confirmed issue, fixed X"
   - FALSE_POSITIVE: "No issue found, noise"
   - UNCLEAR: "Needs more investigation"

4. Store label in database with context

5. Weekly review:
   - Analyze false positive patterns
   - Identify common FP causes
   - Adjust thresholds/rules
   - Retrain models
```

#### **Automated FP Learning**

**Pattern Recognition:**
```
Analyze labeled false positives:

Common FP patterns:
- Deployment events (latency spike during rollout)
- Scheduled batch jobs (expected load increase)
- Time zone changes (DST transitions)
- Holiday traffic patterns

Automated Suppression:
if deployment_in_progress:
    suppress_latency_alerts(duration=30_minutes)

if scheduled_batch_job:
    suppress_throughput_alerts(duration=job_duration)
```

---

## 9. Performance Implications

Anomaly detection systems must balance accuracy with operational overhead. Performance considerations include computational cost, latency, memory usage, and scalability.

### 9.1 Computational Complexity

#### **Algorithm Comparison**

**Statistical Methods:**
```
Z-Score:
- Time: O(1) per observation (with running statistics)
- Space: O(1) (store only mean, variance)
- Latency: <1ms
- Throughput: >100K observations/sec
- Best for: Real-time, high-throughput systems

IQR:
- Time: O(n) for calculation, O(1) with pre-computed quantiles
- Space: O(1) with t-digest approximation
- Latency: <5ms
- Throughput: >50K observations/sec
- Best for: Real-time with moderate throughput
```

**Machine Learning Methods:**
```
Isolation Forest:
- Training: O(t × ψ × n log ψ), where t=trees, ψ=sample size, n=samples
- Prediction: O(t × depth) ≈ O(t × log ψ)
- Time per prediction: ~5-10ms (100 trees, 256 sample size)
- Throughput: ~1-5K observations/sec
- Best for: Batch processing or moderate-throughput real-time

One-Class SVM:
- Training: O(n² to n³) depending on kernel
- Prediction: O(n_support_vectors)
- Time per prediction: ~10-50ms
- Throughput: ~500-2K observations/sec
- Best for: Batch processing, offline training

LSTM Autoencoder:
- Training: O(epochs × n × sequence_length × parameters)
- Prediction: O(sequence_length × parameters)
- Time per prediction: ~50-200ms (CPU), ~5-20ms (GPU)
- Throughput: ~200-1K observations/sec (CPU), ~2-10K (GPU)
- Best for: Batch processing with GPU, complex patterns
```

**Time-Series Methods:**
```
Prophet:
- Training: O(n × iterations) ≈ O(n)
- Prediction: O(1) for single point
- Training time: 1-10 seconds (1 year of daily data)
- Best for: Batch processing, daily/hourly aggregates

ARIMA:
- Training: O(n)
- Prediction: O(1)
- Training time: 1-30 seconds
- Best for: Batch processing, smaller datasets
```

### 9.2 Latency Impact

#### **Real-Time Detection Latency Budget**

**LLM Request Flow:**
```
Total latency = ingestion + processing + detection + alerting

Target: Detection overhead < 5% of request latency

Example:
- Average LLM request: 2000ms
- Acceptable detection overhead: <100ms

Breakdown:
- Metric extraction: 10ms
- Feature calculation: 20ms
- Anomaly detection: 50ms
- Alert dispatch: 20ms
Total: 100ms ✓
```

**Optimization Strategies:**

**1. Asynchronous Detection:**
```
Decouple detection from critical path:

LLM Request → Response to user (2000ms)
    ↓ (async)
Log metrics → Anomaly detection (100ms, non-blocking)
    ↓
Alert if anomaly
```

**2. Sampling:**
```
For high-throughput systems:

if request_count % sample_rate == 0:
    run_detection()

sample_rate = 10  # Check every 10th request

Reduces detection overhead by 90%
Trade-off: May miss individual anomalies, but catches patterns
```

**3. Tiered Detection:**
```
Fast Tier (all requests):
- Simple thresholds (latency > 5000ms)
- Z-score on key metrics
- Overhead: <5ms

Slow Tier (sampled or batch):
- ML models (Isolation Forest)
- Complex correlations
- Overhead: 50-100ms
```

### 9.3 Memory Footprint

#### **Stateful Detection Memory Usage**

**Running Statistics (Z-Score):**
```
Per metric:
- Mean: 8 bytes (float64)
- Variance: 8 bytes
- Count: 8 bytes
Total: 24 bytes/metric

1000 metrics: 24 KB
1M metrics (per-user): 24 MB
```

**Quantile Estimation (t-digest):**
```
Per metric:
- ~100 centroids
- Each: 16 bytes (value + weight)
Total: ~1.6 KB/metric

1000 metrics: 1.6 MB
1M metrics: 1.6 GB
```

**Isolation Forest Model:**
```
100 trees × 256 samples × 10 features
- Tree structure: ~100 KB/tree
Total: ~10 MB

Loaded in memory for inference.
```

**LSTM Autoencoder:**
```
Model size: 5-50 MB (depends on layers, units)
Batch memory: sequence_length × features × batch_size × 4 bytes

Example:
100 seq × 10 features × 32 batch × 4 bytes = 128 KB/batch
```

**Optimization:**

**1. State Pruning:**
```
Expire old state:
- Keep only active users (last 24 hours)
- Archive inactive state to disk/database

Memory savings: 50-90% for sparse user activity
```

**2. Approximate Data Structures:**
```
Use sketches for large-scale monitoring:
- HyperLogLog: Cardinality estimation
- Count-Min Sketch: Frequency estimation
- t-digest: Quantile estimation

Memory: O(log n) instead of O(n)
```

### 9.4 Scalability Considerations

#### **Horizontal Scaling**

**Stateless Detection:**
```
Easy to scale:
- No shared state
- Can run on any worker
- Load balance across instances

Example: Threshold-based detection
Deploy: 10 instances behind load balancer
Throughput: 10x single instance
```

**Stateful Detection:**
```
Requires coordination:
- Partition state by key (user_id, session_id)
- Route requests to consistent workers
- Replicate state for fault tolerance

Example: Per-user anomaly detection

Partitioning:
worker = hash(user_id) % num_workers
Route user requests to assigned worker

State replication:
- Primary worker maintains state
- Backup worker receives state updates
- Failover if primary crashes
```

#### **Data Volume Scaling**

**Aggregation:**
```
Reduce data volume before detection:

Raw data: 100K requests/sec, 10 metrics/request = 1M metrics/sec

Aggregation (1-minute windows):
- 1M metrics/sec → 16.7K metrics/min
- Detection on aggregates: 16.7K/min = 278/sec

Throughput reduction: 1M/sec → 278/sec (3600x)

Trade-off: Lose granularity, but catch broader patterns
```

**Hierarchical Detection:**
```
Level 1: Service-wide metrics (low cardinality)
  - Overall latency, total cost, aggregate quality
  - 10-100 metrics
  - Full detection (all algorithms)

Level 2: Per-endpoint metrics (medium cardinality)
  - Latency per endpoint
  - 100-1000 metrics
  - Moderate detection (statistical + simple ML)

Level 3: Per-user metrics (high cardinality)
  - User-specific patterns
  - 10K-1M metrics
  - Lightweight detection (simple thresholds)
```

### 9.5 Cost Optimization

#### **Infrastructure Costs**

**Compute:**
```
Real-time detection (Flink on AWS):
- 4 Task Managers × r5.xlarge (4 vCPU, 32 GB)
- $0.252/hour × 4 × 730 hours/month = $735/month

Batch detection (EMR/Databricks):
- Run 1 hour/day
- 10 instances × r5.xlarge
- $0.252/hour × 10 × 30 hours/month = $76/month

Hybrid:
- Real-time (critical): $735/month
- Batch (comprehensive): $76/month
Total: $811/month
```

**Storage:**
```
Metrics storage (time-series DB like InfluxDB):
- 100K metrics/sec × 60 sec × 60 min × 24 hours = 8.64B points/day
- 8 bytes/point × 8.64B = 69 GB/day
- 30-day retention: 2.07 TB
- S3 cost: $0.023/GB × 2070 GB = $48/month

Optimizations:
- Downsampling: Keep raw data for 7 days, aggregates for 30 days
- Compression: InfluxDB compression ~3:1
- Adjusted storage: 690 GB, $16/month
```

#### **Model Training Costs**

**Frequency Trade-offs:**
```
LSTM Autoencoder training:
- GPU instance (p3.2xlarge): $3.06/hour
- Training time: 2 hours
- Per training run: $6.12

Frequency options:
- Daily retraining: $6.12 × 30 = $184/month
- Weekly retraining: $6.12 × 4 = $24/month
- Monthly retraining: $6.12 × 1 = $6/month

Recommendation: Weekly (balances adaptation with cost)
```

**Incremental Learning:**
```
Alternative to full retraining:
- Online learning with Random Cut Forest
- Continuously updated model
- No periodic retraining cost

Trade-off: Less sophisticated than deep learning, but cost-effective
```

### 9.6 Monitoring the Monitors

**Meta-Monitoring:**
```
Track anomaly detection system health:

Metrics:
1. Detection latency (p50, p95, p99)
2. Alert volume (per hour/day)
3. False positive rate (from feedback)
4. Detection coverage (% requests monitored)
5. System resource usage (CPU, memory)
6. Model drift (detection accuracy over time)

Alerts:
- Detection latency > 500ms: SLOW_DETECTION
- Alert volume > 100/hour: ALERT_STORM
- False positive rate > 20%: TUNE_THRESHOLDS
- CPU > 80%: SCALE_UP
```

**Performance Budgets:**
```
Define and enforce SLOs for detection system:

SLO 1: P99 detection latency < 200ms
SLO 2: Alert precision > 70%
SLO 3: Alert recall > 80%
SLO 4: System uptime > 99.9%

Monitor compliance and trigger remediation if violated.
```

---

## Summary and Recommendations

### Quick Reference: Algorithm Selection

| Use Case | Real-Time | Batch | Algorithm | Complexity |
|----------|-----------|-------|-----------|------------|
| Latency spikes | ✓ | - | Z-Score, IQR | Low |
| Cost anomalies | ✓ | ✓ | Isolation Forest | Medium |
| Drift detection | - | ✓ | PSI, KL Divergence | Low |
| Hallucinations | ✓ | - | LLM-Check, Self-Consistency | Medium |
| Quality degradation | - | ✓ | Semantic Coherence, Prophet | Medium |
| Complex patterns | - | ✓ | LSTM Autoencoder | High |
| Multi-dimensional | ✓ | ✓ | Isolation Forest | Medium |
| Streaming data | ✓ | - | Flink + RCF | Medium |

### Implementation Roadmap

**Phase 1: Foundation (Weeks 1-2)**
- Implement basic statistical methods (Z-score, IQR)
- Set up logging and metrics infrastructure
- Deploy simple threshold-based alerts
- Establish baseline metrics

**Phase 2: Real-Time Detection (Weeks 3-4)**
- Deploy streaming pipeline (Kafka + Flink)
- Implement Isolation Forest for multi-dimensional anomalies
- Set up real-time alerting (PagerDuty, Slack)
- Configure initial thresholds

**Phase 3: Advanced Detection (Weeks 5-8)**
- Train LSTM autoencoders for complex patterns
- Implement drift detection (PSI, KL divergence)
- Deploy hallucination detection (LLM-Check, self-consistency)
- Set up quality monitoring (coherence, user feedback)

**Phase 4: Optimization (Weeks 9-12)**
- Tune thresholds based on feedback
- Reduce false positives (ensemble methods, correlation)
- Optimize performance (sampling, caching)
- Establish monitoring dashboards and reports

**Phase 5: Continuous Improvement (Ongoing)**
- Collect labeled data (true/false positives)
- Retrain models weekly/monthly
- A/B test threshold changes
- Monitor system performance and scale as needed

---

## References and Further Reading

### Research Papers
1. **LLM-Check** (NeurIPS 2024): "Investigating Detection of Hallucinations in Large Language Models"
2. **Task Drift Detection** (2024): "Catching LLM Task Drift with Activations" (arXiv:2406.00799)
3. **LSTM Autoencoder + Isolation Forest**: "An anomaly detection approach based on the combination of LSTM autoencoder and isolation forest for multivariate time series data"

### Industry Reports
1. Fiddler AI: "How to Monitor LLMOps Performance with Drift Monitoring" (2024)
2. Galileo: "7 Strategies To Solve LLM Reliability Challenges at Scale" (2024)
3. Datadog: "LLM Observability Best Practices" (2024)

### Tools and Platforms
1. **Datadog LLM Observability**: Hallucination detection, cost tracking
2. **Opik**: Cost and quality monitoring for LLM applications
3. **WhyLabs**: Real-time drift and performance monitoring
4. **Apache Flink**: Real-time streaming analytics
5. **Prophet**: Time-series forecasting (Facebook)

### Open Source Libraries
1. **scikit-learn**: Isolation Forest, One-Class SVM
2. **TensorFlow/PyTorch**: LSTM Autoencoders
3. **sentence-transformers**: Embedding-based similarity
4. **evidently**: Data drift detection library

---

**Document Version:** 1.0
**Last Updated:** 2025-11-06
**Author:** LLM-Sentinel Anomaly Detection Research Specialist
