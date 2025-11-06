# LLM-Sentinel: Technical Alternatives & Trade-offs Analysis

## Overview

This document provides detailed analysis of alternative technology choices and their trade-offs for the LLM-Sentinel project. Use this in conjunction with TECHNICAL_STACK_RESEARCH.md when evaluating architectural decisions.

---

## 1. Async Runtime Trade-offs

### Tokio vs async-std vs smol

| Feature | Tokio | async-std | smol |
|---------|-------|-----------|------|
| **Ecosystem** | 20,768+ crates | Smaller | Growing |
| **Default Scheduler** | Multi-threaded work-stealing | Single/Multi configurable | Lightweight |
| **Runtime Size** | Larger | Medium | Smallest |
| **API Style** | Tokio-specific | std-like | Minimal |
| **Production Use** | Extensive (AWS, Cloudflare) | Moderate | Growing |
| **Learning Curve** | Moderate | Easy (if know std) | Easy |
| **Best For** | Production servers | Prototypes/small apps | Embedded/lightweight |

**Decision Factors:**
- **Choose Tokio if:** Building production servers, need ecosystem support, scaling is priority
- **Choose async-std if:** Want std-like APIs, small project, learning async Rust
- **Choose smol if:** Embedded systems, minimal footprint critical, simple async needs

**LLM-Sentinel Recommendation:** Tokio (ecosystem, production maturity, scaling)

**Migration Difficulty:** High - runtimes are incompatible, mixing causes unpredictable behavior

---

## 2. Channel Implementation Trade-offs

### Performance Comparison

| Channel | Async Performance | Sync Performance | Safety | Maintenance |
|---------|------------------|------------------|--------|-------------|
| **crossfire** | Excellent (lockless) | Excellent | Safe | Active (v2.1, 2025) |
| **flume** | Very Good | Very Good | Safe (no unsafe) | Maintenance mode |
| **tokio::sync::mpsc** | Excellent (in-runtime) | N/A | Safe | Active |
| **crossbeam** | Good | Excellent | Safe | Active |

**Detailed Trade-offs:**

#### Crossfire
- **Pros:** Best async performance, lockless design, lighter notifications
- **Cons:** Newer (less battle-tested than others), smaller ecosystem
- **When to Use:** High-performance async workloads, lockless required

#### Flume
- **Pros:** No unsafe code, simple API, good performance, multi-producer multi-consumer
- **Cons:** Maintenance mode (no new features), slower than crossfire
- **When to Use:** Need safety guarantees, multi-producer/consumer, general purpose

#### tokio::sync::mpsc
- **Pros:** Best integration with Tokio, efficient context-switching, first-party support
- **Cons:** Tied to Tokio, not usable in blocking contexts
- **When to Use:** Within Tokio applications, async-only workloads

#### Crossbeam
- **Pros:** Best blocking performance, mature, well-tested
- **Cons:** Not async-native, requires spawning blocking tasks in async
- **When to Use:** CPU-bound work, blocking contexts, proven reliability

**LLM-Sentinel Recommendation:**
- **Primary:** crossfire (async message passing between components)
- **Secondary:** tokio::sync (intra-service, Tokio-native)
- **Fallback:** flume (if need no-unsafe guarantee)

---

## 3. Actor Framework Trade-offs

### Feature Comparison

| Feature | Ractor | Actix | Kameo | Coerce |
|---------|--------|-------|-------|--------|
| **Performance (Messaging)** | Very Good | Excellent | Very Good | Very Good |
| **Performance (Spawning)** | Good | Excellent | Good | Good |
| **Distribution** | Built-in | External crates | Built-in | Built-in |
| **Fault Tolerance** | Built-in (Erlang-style) | Manual | Manual | Manual |
| **Runtime** | Tokio + Async Std | Own (on Tokio) | Tokio | Tokio |
| **Supervision** | Yes (gen_server model) | Yes | Limited | Limited |
| **Maturity** | Newer | Very Mature | Newer | Moderate |

**Detailed Analysis:**

#### Ractor
- **Pros:** Erlang gen_server patterns, built-in distribution, fault tolerance, supervision trees
- **Cons:** Newer (less production history), slightly slower than Actix
- **Best For:** Distributed systems, fault-tolerant architectures, Erlang-inspired designs
- **Trade-off:** Slightly slower performance for much better fault tolerance

#### Actix
- **Pros:** Fastest performance, very mature, large ecosystem, proven in production
- **Cons:** No built-in distribution, own runtime (complexity), steeper learning curve
- **Best For:** Single-node high-performance, existing Actix ecosystem
- **Trade-off:** Maximum performance for less distributed-system features

#### Kameo
- **Pros:** Modern API, good performance, built-in distribution, Tokio-native
- **Cons:** Newer library, smaller community
- **Best For:** Modern Tokio applications, distributed systems
- **Trade-off:** Modern design for less production track record

#### Coerce
- **Pros:** Built-in distribution, Tokio-native, clean API
- **Cons:** Smaller ecosystem, less documentation
- **Best For:** Distributed Tokio applications
- **Trade-off:** Simplicity for smaller community

**Decision Matrix:**

| Use Case | Recommended | Reason |
|----------|-------------|--------|
| Distributed system with fault tolerance | **Ractor** | Erlang patterns, supervision |
| Maximum single-node performance | **Actix** | Fastest messaging/spawning |
| Modern Tokio-native distributed | **Kameo** | Clean API, good performance |
| Simple Tokio distributed | **Coerce** | Minimal complexity |

**LLM-Sentinel Recommendation:** Ractor (distributed metrics processing, fault tolerance critical)

**Alternative Path:** Start without actor framework, add later if needed (YAGNI principle)

---

## 4. Time-Series Database Trade-offs

### Comprehensive Comparison

| Database | InfluxDB 3 | Prometheus | QuestDB | TimescaleDB | GreptimeDB |
|----------|-----------|------------|---------|-------------|------------|
| **Language** | Rust | Go | Java/C++ | C (PostgreSQL) | Rust |
| **Write Pattern** | Push | Pull | Push | Push | Push |
| **Query Language** | SQL | PromQL | SQL | SQL | SQL + PromQL |
| **Ingestion Rate** | Millions/sec | High | Very High | High | Very High |
| **Cardinality** | Infinite | Limited | High | High | High |
| **Storage** | Object store (S3) | Local TSDB | Local | PostgreSQL | Local/Cloud |
| **Clustering** | Yes | Federation | Yes | Yes | Yes |
| **Cloud Native** | Yes | Yes | Moderate | Moderate | Yes |

**Detailed Trade-offs:**

#### InfluxDB 3 (Rust-based)
- **Pros:** Rust foundation, Arrow/DataFusion, SQL, infinite cardinality, cloud-native
- **Cons:** Newer architecture, migration from v2 complex, potentially higher cost
- **Best For:** New deployments, high cardinality, complex analytics, cloud deployments
- **Performance:** Millions writes/sec, sub-10ms queries
- **Trade-off:** Cutting-edge features for less v3 production track record

#### Prometheus
- **Pros:** Industry standard, excellent alerting, pull-based (less client config), lightweight
- **Cons:** Limited cardinality, no SQL, limited long-term storage, PromQL learning curve
- **Best For:** Kubernetes monitoring, operational metrics, real-time alerting
- **Performance:** Optimized for monitoring use case
- **Trade-off:** Simplicity and standards for limited analytics capabilities

#### QuestDB
- **Pros:** Very high performance, SQL support, time-series optimized, ILP protocol
- **Cons:** Smaller ecosystem, Java dependency, less mature clustering
- **Best For:** High-performance single-node, SQL analytics
- **Performance:** Excellent for ingestion and queries
- **Trade-off:** Raw performance for smaller community/ecosystem

#### TimescaleDB
- **Pros:** PostgreSQL compatibility, mature ecosystem, familiar tooling, good scaling
- **Cons:** PostgreSQL limitations, schema required, extra storage for aggregations
- **Best For:** Teams with PostgreSQL expertise, need ACID, complex relations
- **Performance:** Better than vanilla PostgreSQL, not as fast as purpose-built TSDB
- **Trade-off:** PostgreSQL familiarity for lower raw performance

#### GreptimeDB (Rust)
- **Pros:** Rust-based, unified metrics/logs/traces, open source, good performance
- **Cons:** Newer project, smaller community, less documentation
- **Best For:** Unified observability, Rust ecosystem, new deployments
- **Performance:** High performance (Rust advantages)
- **Trade-off:** Modern architecture for less maturity

**Multi-Database Strategy:**

Some organizations use multiple:
- **Prometheus:** Short-term operational metrics + alerting
- **InfluxDB/TimescaleDB:** Long-term storage + analytics

**LLM-Sentinel Recommendation:**
- **Primary:** InfluxDB 3 (Rust synergy, high cardinality for LLM metrics, analytics)
- **Monitoring:** Prometheus (operational metrics, alerting)
- **Consider:** GreptimeDB if want unified metrics/logs/traces in Rust

---

## 5. Message Queue Trade-offs

### Comparison Matrix

| Feature | Kafka (rdkafka) | NATS | RabbitMQ (lapin) |
|---------|----------------|------|------------------|
| **Performance** | Highest | High | Good |
| **Scalability** | Excellent | Excellent | Good |
| **Complexity** | High | Low | Medium |
| **Setup Time** | Hours/Days | Minutes | Medium |
| **Ordering** | Partition-level | Subject-level | Queue-level |
| **Persistence** | Disk (configurable) | Memory/File/JetStream | Disk (durable queues) |
| **Delivery Guarantees** | Exactly-once | At-most/least-once, JetStream: exactly-once | At-most/least-once |
| **Streaming** | Native | JetStream | Streams plugin |
| **Overhead** | Highest | Lowest | Medium |
| **Cloud Native** | Moderate | Excellent | Moderate |
| **Ops Expertise** | Significant | Minimal | Moderate |
| **Rust Client Quality** | Excellent (rdkafka) | Excellent | Good (lapin) |

**Detailed Analysis:**

#### Kafka (rdkafka)
- **Pros:**
  - Highest throughput and scalability
  - Exactly-once semantics
  - Battle-tested at massive scale
  - Excellent for event sourcing/streaming
  - Benefits from parallelism
- **Cons:**
  - Complex setup and operations
  - Requires Zookeeper/KRaft
  - Expensive managed offerings
  - Significant expertise needed
  - High resource usage
- **Best For:**
  - High-volume event streaming
  - Event sourcing architectures
  - Multiple consumers of same data
  - Need for replay capability
- **Performance:**
  - Outperforms alternatives in throughput
  - Scales horizontally well
- **Trade-off:** Maximum performance/guarantees for operational complexity

#### NATS
- **Pros:**
  - Excellent developer ergonomics
  - Setup in minutes
  - Most lightweight
  - Cloud-native optimized
  - Benefits from parallelism
  - Simple operations
- **Cons:**
  - Less mature than Kafka for streaming (JetStream newer)
  - Fewer ecosystem integrations
  - Less enterprise adoption
- **Best For:**
  - Microservices communication
  - Cloud-native applications
  - Low-latency messaging
  - Simple pub/sub patterns
  - Edge deployments
- **Performance:**
  - Lower absolute throughput than Kafka
  - Excellent for latency-sensitive apps
  - Lowest overhead
- **Trade-off:** Simplicity and speed for less ecosystem maturity

#### RabbitMQ (lapin)
- **Pros:**
  - Mature and stable
  - Rich routing capabilities
  - Enterprise support
  - Many managed offerings (AWS, etc.)
  - Raft consensus (improved in recent years)
  - Good Rust client (lapin with Tokio)
- **Cons:**
  - Performance degrades with increased parallelism
  - More complex than NATS
  - Less suited for streaming use cases
- **Best For:**
  - Traditional message queuing
  - Complex routing logic
  - Enterprise environments with RabbitMQ
  - Request/reply patterns
- **Performance:**
  - Good for moderate loads
  - Not optimized for max throughput
- **Trade-off:** Routing flexibility for lower performance ceiling

**Use Case Decision Tree:**

```
Need event streaming / event sourcing?
  └─ YES → Kafka
  └─ NO → Need complex routing?
           └─ YES → RabbitMQ
           └─ NO → NATS (or simple Kafka)

Need max throughput (>100k msg/s)?
  └─ YES → Kafka
  └─ NO → Need minimal ops overhead?
           └─ YES → NATS
           └─ NO → RabbitMQ (if existing infrastructure)
```

**LLM-Sentinel Recommendation:**
- **Primary:** Kafka (rdkafka) - high-volume metrics ingestion, event sourcing for audit
- **Alternative:** NATS - if simplicity more important than max throughput
- **Avoid:** RabbitMQ (unless existing enterprise requirement)

**Hybrid Approach:**
- Kafka for metrics ingestion pipeline (high volume)
- NATS for control plane / service-to-service
- Redis Pub/Sub for real-time alerts (lightweight)

---

## 6. HTTP Framework Trade-offs

### Performance Benchmarks (2025)

| Metric | Axum | Actix-web | Rocket |
|--------|------|-----------|--------|
| **Throughput** | ~55k req/s | ~60k req/s | ~45k req/s |
| **Latency P50** | ~12ms | ~10ms | ~15ms |
| **Latency P99** | ~45ms | ~42ms | ~58ms |
| **Memory Usage** | Lowest | Low | Higher |
| **Concurrent Connections** | Very High | Highest | High |
| **Memory per Connection** | Lowest | Low | Higher |

**Detailed Comparison:**

#### Axum
- **Pros:**
  - Modern, ergonomic API
  - Excellent Tokio integration
  - Best memory efficiency
  - Type-safe routing via Tower
  - Lower learning curve
  - Growing ecosystem
- **Cons:**
  - Slightly slower than Actix (marginal)
  - Fewer built-in features
  - Smaller middleware ecosystem
- **Best For:**
  - New projects
  - Tokio-native architectures
  - Type-safe APIs
  - Memory-constrained environments
- **Trade-off:** Ergonomics and memory efficiency for marginal performance difference

#### Actix-web
- **Pros:**
  - Highest throughput
  - Lowest latency
  - Handles most concurrent connections
  - Extensive middleware
  - Mature ecosystem
  - Complex routing support
- **Cons:**
  - Steeper learning curve
  - Uses Actix runtime (complexity)
  - More boilerplate
  - Heavier memory usage
- **Best For:**
  - Maximum performance critical
  - Complex middleware chains
  - Existing Actix expertise
  - High-scale APIs
- **Trade-off:** Maximum performance for complexity

#### Rocket
- **Pros:**
  - Easiest to learn
  - Great documentation
  - Productive development
  - Request guards
- **Cons:**
  - Slower than Axum/Actix
  - Higher resource usage
  - Less flexible
- **Best For:**
  - Rapid prototyping
  - Learning Rust web dev
  - Simple APIs
- **Trade-off:** Developer productivity for performance

**2025 Consensus:** Axum vs Actix-web are top-tier, choice depends on priorities

**Decision Factors:**

| Priority | Choose |
|----------|--------|
| Maximum performance | Actix-web |
| Memory efficiency | Axum |
| Type safety | Axum |
| Ecosystem maturity | Actix-web |
| Learning curve | Axum |
| Complex middleware | Actix-web |
| Modern patterns | Axum |

**LLM-Sentinel Recommendation:**
- **Primary:** Axum (memory efficiency for containers, type safety, modern API)
- **Alternative:** Actix-web (if max performance > everything else)
- **Consider:** Running both in benchmarks before deciding

**gRPC Consideration:**
- Tonic is the only mature option (no trade-off analysis needed)
- Can run Tonic alongside Axum or Actix for hybrid HTTP/gRPC

---

## 7. Machine Learning Library Trade-offs

### SmartCore vs Linfa vs Custom

| Aspect | SmartCore | Linfa | Custom (ndarray + algorithms) |
|--------|-----------|-------|-------------------------------|
| **Completeness** | Most complete | Comprehensive toolkit | À la carte |
| **Performance** | Good | Good | Optimizable |
| **Maintenance** | Active | Active | Self-maintained |
| **Algorithms** | Wide variety | Modular | Selected |
| **Learning Curve** | Moderate | Moderate | Steep |
| **Flexibility** | Limited | Good | Maximum |
| **Dependencies** | nalgebra OR ndarray | ndarray-focused | Minimal |

**Detailed Trade-offs:**

#### SmartCore
- **Pros:**
  - Most complete ML library in Rust
  - Classification, regression, clustering, dimensionality reduction
  - Model selection and evaluation tools
  - Supports both nalgebra and ndarray (0.4: ndarray-only)
- **Cons:**
  - Larger dependency footprint
  - Less modular (all-in-one)
  - Some algorithms less optimized than specialized crates
- **Best For:**
  - Need variety of algorithms
  - Production-ready implementations
  - Standardized API
- **Trade-off:** Completeness for larger binary size

#### Linfa
- **Pros:**
  - Modular toolkit (use only what you need)
  - Standardized API (easy to swap algorithms)
  - Good interoperability
  - Active development (2025 updates)
  - ndarray-focused
- **Cons:**
  - Less complete than SmartCore
  - Some algorithms still experimental
  - Fragmented across sub-crates
- **Best For:**
  - Need specific algorithms
  - Experimental work
  - Modular architecture
- **Trade-off:** Modularity for fragmentation

#### Custom Implementation
- **Pros:**
  - Maximum control and optimization
  - Minimal dependencies
  - Tailored to exact needs
  - No unnecessary code
- **Cons:**
  - High development time
  - Must maintain algorithms
  - Risk of bugs
  - No standard API
- **Best For:**
  - Very specific requirements
  - Performance critical
  - Small set of algorithms
- **Trade-off:** Control for development/maintenance cost

**For Anomaly Detection:**

| Algorithm | Best Library | Alternative |
|-----------|-------------|-------------|
| Isolation Forest | isolation_forest crate | Custom |
| Statistical (MAD, Hampel) | augurs-outlier | statrs + custom |
| Time-series specific | augurs-outlier | anomaly_detection crate |
| DBSCAN | augurs-outlier | linfa-clustering |
| Neural approaches | Custom with burn/candle | Python interop |

**LLM-Sentinel Recommendation:**
- **General ML:** SmartCore (comprehensive, production-ready)
- **Specialized:** Linfa for algorithms not in SmartCore
- **Anomaly Detection:** Specialized crates (augurs-outlier, isolation_forest)
- **Custom:** Only for highly-specific optimizations

**Python Interop Consideration:**
- Use PyO3 to call Python ML libraries if needed
- Trade-off: Access to scikit-learn/PyTorch for FFI overhead and deployment complexity
- Recommendation: Avoid unless absolutely necessary

---

## 8. Cache Strategy Trade-offs

### In-Memory vs Distributed vs Hybrid

| Strategy | Pros | Cons | Use Case |
|----------|------|------|----------|
| **In-Memory Only (Moka)** | Fastest, simplest, no network | Limited to single instance, lost on restart | Single instance, cache can rebuild |
| **Distributed Only (Redis)** | Shared across instances, persistent | Network latency, infrastructure complexity | Multi-instance, critical data |
| **Hybrid (Moka + Redis)** | Best of both, L1/L2 caching | More complex, cache coherency challenges | High-scale production |

**Moka (In-Memory) Details:**

**Eviction Policies:**
- **TinyLFU (default):** LRU eviction + LFU admission (best general purpose)
- **LRU:** Simple least-recently-used
- **LFU:** Least-frequently-used

**Features:**
- Time-based expiration (TTL)
- Size-based eviction
- Entry invalidation
- Statistics (hit rate, evictions)
- Concurrent access (thread-safe)
- Async API (future-aware)

**Redis (Distributed) Details:**

**Eviction Policies:**
- **allkeys-lru:** Evict any key, LRU (good default)
- **volatile-ttl:** Evict keys with TTL, by TTL
- **allkeys-lfu:** Evict any key, LFU
- **noeviction:** Return errors when memory full

**Features:**
- Persistence (RDB snapshots, AOF logs)
- Pub/Sub for invalidation
- Clustering for scale
- Sentinel for HA

**Hybrid Strategy:**

```
L1 Cache (Moka): Hot data, ms latency
  └─ Miss → L2 Cache (Redis): Warm data, single-digit ms latency
              └─ Miss → Database: Cold data, 10-100ms latency
```

**Cache Coherency Strategies:**

1. **TTL-based:** Short TTL in L1, longer in L2 (simple, eventual consistency)
2. **Pub/Sub Invalidation:** Redis pub/sub to invalidate Moka entries (complex, strong consistency)
3. **Write-through:** Update both on write (moderate complexity)

**Decision Matrix:**

| Deployment | Cache Strategy | Reasoning |
|------------|---------------|-----------|
| Single instance | Moka only | No distribution needed |
| Multi-instance, stateless | Redis only | Shared state required |
| Multi-instance, high read | Moka + Redis | L1/L2 for performance |
| Multi-instance, low latency | Moka + Redis + pub/sub | Coherency critical |

**LLM-Sentinel Recommendation:**
- **Start:** Moka only (for MVP, single instance)
- **Scale:** Add Redis when multiple instances deployed
- **Optimize:** Hybrid L1/L2 for high-scale (if needed)

**What to Cache:**

| Data | Cache Layer | TTL | Eviction |
|------|------------|-----|----------|
| Computed anomaly scores | Moka | 1-5 min | TinyLFU |
| Aggregated metrics | Moka + Redis | 5-15 min | TinyLFU/LRU |
| Model parameters | Redis | 1 hour | None (persist) |
| User sessions | Redis | 24 hours | TTL |
| Static config | Moka | 1 hour | Size-based |

---

## 9. Error Handling Strategy Trade-offs

### anyhow vs thiserror vs snafu vs custom

| Approach | Pros | Cons | Use Case |
|----------|------|------|----------|
| **anyhow** | Simple, quick, good context | Opaque errors, can't match variants | Applications |
| **thiserror** | Structured, matchable, derive macro | More boilerplate | Libraries |
| **snafu** | Context + structure, good ergonomics | Another dependency | Large projects |
| **custom** | Maximum control | High maintenance | Very specific needs |

**Detailed Comparison:**

#### anyhow
```rust
use anyhow::{Context, Result};

fn process() -> Result<()> {
    let data = load_data()
        .context("Failed to load data")?;
    Ok(())
}
```

**Pros:**
- Very quick to implement
- Automatic context chains
- Great error messages
- Minimal boilerplate

**Cons:**
- Can't pattern match on error types
- Less structured
- Not suitable for library APIs

**Best For:**
- Application code
- Rapid development
- Error propagation without handling

#### thiserror
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid data: {msg}")]
    InvalidData { msg: String },
}
```

**Pros:**
- Structured error types
- Pattern matching possible
- Automatic conversions with #[from]
- Good for public APIs

**Cons:**
- More boilerplate
- Manual error definition
- Context requires manual passing

**Best For:**
- Library crates
- Public APIs
- When callers need to handle specific errors

#### snafu
```rust
use snafu::{Snafu, ResultExt};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not load config from {path}"))]
    ConfigLoad { source: std::io::Error, path: String },
}

fn load() -> Result<Config, Error> {
    std::fs::read_to_string("config.toml")
        .context(ConfigLoadSnafu { path: "config.toml" })?;
}
```

**Pros:**
- Combines anyhow's context with thiserror's structure
- Great ergonomics
- Context + error variants
- Good for multi-crate projects

**Cons:**
- Another dependency
- Different error handling pattern
- Less common (team familiarity)

**Best For:**
- Large projects with multiple crates
- Need both context and structure
- Projects like GreptimeDB use it

**LLM-Sentinel Recommendation:**

```rust
// Internal library crates (src/lib/...)
use thiserror::Error;

// Application/binary crate (src/main.rs, src/bin/...)
use anyhow::{Context, Result};

// Pattern: Convert thiserror → anyhow at boundaries
fn main() -> anyhow::Result<()> {
    let config = load_config()
        .map_err(anyhow::Error::from)
        .context("Failed to initialize")?;
    Ok(())
}
```

**Hybrid Strategy Benefits:**
- Internal crates have structured errors (testable, matchable)
- Application layer has simple error propagation
- Best of both worlds
- Clear separation of concerns

---

## 10. Testing Strategy Trade-offs

### Built-in vs Property-based vs Mock-heavy

| Strategy | Pros | Cons | When to Use |
|----------|------|------|-------------|
| **Built-in only** | Simple, fast, no deps | Manual test cases, miss edge cases | Simple logic, clear paths |
| **+ Property testing** | Finds edge cases, less manual work | Slower, tricky to write properties | Algorithms, data structures |
| **+ Mock testing** | Isolate dependencies, fast tests | Complex setup, tight coupling | External dependencies |
| **Integration heavy** | Tests real behavior | Slow, complex setup, flaky | Critical paths, E2E |

**Test Pyramid for LLM-Sentinel:**

```
        /\
       /E2\       ← Few: Full system tests
      /----\
     /Integr\     ← Some: Component integration
    /--------\
   /   Unit   \   ← Many: Pure functions, algorithms
  /------------\
```

**Recommendations by Component:**

| Component | Strategy | Tools | Reasoning |
|-----------|----------|-------|-----------|
| **Anomaly algorithms** | Property + Unit | proptest + built-in | Find edge cases in math |
| **API handlers** | Mock + Integration | mockall + reqwest | Isolate, then integration |
| **Data processing** | Property + Unit | proptest | Variety of input data |
| **Configuration** | Unit | built-in | Clear success/failure |
| **Database layer** | Mock + Integration | mockall + testcontainers | Fast unit, real integration |
| **End-to-end** | Integration | built-in + helper crates | Real deployments |

**Property Testing Example:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn anomaly_score_bounded(values in prop::collection::vec(any::<f64>(), 1..100)) {
        let score = calculate_anomaly_score(&values);
        prop_assert!(score >= 0.0 && score <= 1.0);
    }
}
```

**Mock Testing Example:**

```rust
use mockall::*;

#[automock]
trait MetricsStore {
    fn save(&self, metric: Metric) -> Result<()>;
}

#[test]
fn test_processor_saves_metrics() {
    let mut mock = MockMetricsStore::new();
    mock.expect_save()
        .times(1)
        .returning(|_| Ok(()));

    let processor = Processor::new(mock);
    processor.process(data);
}
```

**testcontainers for Integration:**

```rust
use testcontainers::*;

#[tokio::test]
async fn test_with_real_redis() {
    let docker = clients::Cli::default();
    let redis = docker.run(images::redis::Redis::default());

    let port = redis.get_host_port_ipv4(6379);
    // Test with real Redis...
}
```

**Performance Testing:**

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_anomaly_detection(c: &mut Criterion) {
    c.bench_function("detect_anomalies_1000_points", |b| {
        b.iter(|| detect_anomalies(&data));
    });
}

criterion_group!(benches, benchmark_anomaly_detection);
criterion_main!(benches);
```

**LLM-Sentinel Testing Recommendation:**

1. **Unit tests:** Built-in for all pure functions
2. **Property tests:** Algorithms, data processing (proptest)
3. **Mocks:** External services during unit tests (mockall)
4. **Integration:** Real components (testcontainers for databases)
5. **Benchmarks:** Performance critical paths (criterion)
6. **E2E:** CI/CD with deployed environment

**Test Coverage Target:** 80%+ for core logic, 60%+ overall

---

## 11. Build & Deployment Trade-offs

### Optimization Levels

| Profile | Optimization | Compile Time | Binary Size | Performance | Use Case |
|---------|-------------|--------------|-------------|-------------|----------|
| **dev** | 0 | Fast | Large | Slow | Development |
| **release** | 3 | Slow | Medium | Fast | Production |
| **release + LTO** | 3 + LTO | Very slow | Small | Faster | Production optimized |
| **release + PGO** | 3 + PGO | Extremely slow | Small | Fastest | Production max perf |

**Cargo.toml Profiles:**

```toml
[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = false
codegen-units = 16

[profile.release-optimized]
inherits = "release"
lto = "thin"
codegen-units = 1
strip = true

[profile.release-max]
inherits = "release"
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

**Link-Time Optimization (LTO):**

- **"thin":** Good balance (smaller, faster compile than "fat")
- **"fat":** Maximum optimization (very slow compile)
- **false:** Fastest compile, larger binary

**Profile-Guided Optimization (PGO):**

1. Build instrumented binary
2. Run with typical workload
3. Rebuild with profile data
4. Result: 10-20% performance gain

**Trade-off:** 2-3x longer build time for 10-20% runtime improvement

**Docker Build Strategies:**

#### Strategy 1: Simple Multi-stage
```dockerfile
FROM rust:1.83 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-sentinel /usr/local/bin/
```

**Pros:** Simple, reliable
**Cons:** Rebuilds everything on code change

#### Strategy 2: Cached Dependencies
```dockerfile
FROM rust:1.83 AS builder
WORKDIR /app
# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build app
COPY . .
RUN touch src/main.rs
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-sentinel /usr/local/bin/
```

**Pros:** Faster rebuilds (cached deps)
**Cons:** More complex, fragile

#### Strategy 3: cargo-chef (Recommended)
```dockerfile
FROM rust:1.83 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-sentinel /usr/local/bin/
```

**Pros:** Reliable caching, faster CI
**Cons:** Requires cargo-chef

**LLM-Sentinel Build Recommendation:**

- **Development:** Standard `dev` profile
- **CI:** `release` profile with thin LTO
- **Production:** `release-optimized` profile (thin LTO, strip)
- **Max Performance:** `release-max` with PGO (if profiling data available)
- **Docker:** cargo-chef multi-stage builds

**CI/CD Build Time Optimization:**

1. Use sccache or cargo-cache
2. Parallel compilation (set CARGO_BUILD_JOBS)
3. Incremental compilation in CI (cache target/)
4. Split builds (check → test → build)
5. Use rust-lld linker (30%+ faster)

---

## 12. Deployment Architecture Trade-offs

### Monolith vs Microservices vs Modular Monolith

| Architecture | Pros | Cons | Complexity |
|--------------|------|------|------------|
| **Monolith** | Simple deploy, low latency, easy dev | Harder to scale components independently | Low |
| **Microservices** | Independent scaling, tech flexibility | Network overhead, distributed complexity | High |
| **Modular Monolith** | Clear boundaries, single deploy, can extract later | Requires discipline | Medium |

**Recommendation for LLM-Sentinel:** Start with **Modular Monolith**

**Modular Monolith Structure:**

```
llm-sentinel/
├── crates/
│   ├── ingestion/        # Metrics ingestion
│   ├── processing/       # Stream processing
│   ├── detection/        # Anomaly detection
│   ├── storage/          # Database layer
│   ├── api/              # REST/gRPC API
│   └── common/           # Shared types
├── src/
│   └── main.rs           # Binary entry point
└── Cargo.toml            # Workspace
```

**Benefits:**
- Clear module boundaries
- Can extract to microservice later
- Easy to develop and test
- Low operational overhead
- Fast inter-module communication

**When to Extract Microservices:**

1. **Independent scaling needs** (e.g., ingestion needs 10x instances vs API)
2. **Team growth** (separate teams own services)
3. **Technology needs** (different languages for specialized components)
4. **Deployment independence** (deploy ingestion without API restart)

**Kubernetes Deployment Patterns:**

#### Pattern 1: Single Deployment (Monolith)
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-sentinel
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: llm-sentinel
        image: llm-sentinel:latest
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
```

#### Pattern 2: Multiple Deployments (Microservices)
```yaml
# Ingestion service
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-sentinel-ingestion
spec:
  replicas: 10  # Scale independently
  ...

---
# API service
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-sentinel-api
spec:
  replicas: 3
  ...
```

**LLM-Sentinel Deployment Recommendation:**

1. **MVP:** Single deployment (modular monolith)
2. **Growth:** Monitor metrics to identify bottlenecks
3. **Scale:** Add replicas (horizontal scaling)
4. **Extract:** Move high-load components to separate services if needed

---

## 13. Observability Trade-offs

### OpenTelemetry vs Vendor-Specific vs Hybrid

| Approach | Pros | Cons | Lock-in |
|----------|------|------|---------|
| **OpenTelemetry only** | Vendor-neutral, standard, flexible backends | More setup, less integrated | None |
| **Vendor-specific (Datadog, New Relic)** | Integrated, easy setup, rich features | Vendor lock-in, expensive | High |
| **Hybrid (OTel + Vendor)** | Flexibility + integrated experience | Duplicate telemetry, complex | Medium |

**OpenTelemetry Benefits:**

- **Vendor neutral:** Switch backends without code changes
- **Standard:** Industry-standard traces/metrics/logs
- **Flexible:** Multiple exporters (Prometheus, Jaeger, Tempo, etc.)
- **Open source:** No licensing costs

**OpenTelemetry Challenges:**

- **Setup complexity:** More configuration than vendor SDKs
- **Less integrated:** Manual correlation across signals
- **Backend choice:** Must choose and operate backend(s)

**Vendor SDK Benefits:**

- **Quick start:** Minimal configuration
- **Integrated:** Automatic correlation, APM, dashboards
- **Support:** Professional support and documentation

**Vendor SDK Challenges:**

- **Lock-in:** Hard to switch vendors
- **Cost:** Can be expensive at scale
- **Less control:** Limited customization

**Recommended Stack for LLM-Sentinel:**

```
┌─────────────────────────────────────────┐
│         Application Code                │
│   (instrumented with OTel SDK)          │
└──────────────┬──────────────────────────┘
               │
       ┌───────┴────────┐
       │                │
       v                v
  ┌─────────┐      ┌─────────┐
  │ Metrics │      │ Traces  │
  │ (Prom)  │      │ (Tempo) │
  └─────────┘      └─────────┘
       │                │
       └────────┬───────┘
                │
         ┌──────v──────┐
         │   Grafana   │
         │  (Visualize)│
         └─────────────┘
```

**Trade-off Decision:**

- **Startup/MVP:** OpenTelemetry (flexibility, low cost)
- **Enterprise:** Hybrid (OTel + vendor for specific features)
- **Scale:** OpenTelemetry (avoid vendor costs at scale)

---

## 14. Configuration Management Trade-offs

### Figment vs config-rs vs custom

| Library | Flexibility | Ergonomics | Complexity |
|---------|------------|------------|------------|
| **Figment** | Very High | Excellent | Low |
| **config-rs** | High | Good | Medium |
| **Custom** | Maximum | Poor | High |

**Figment Advantages:**

- Tracks provenance (know where each value came from)
- Type-safe extraction
- Excellent merging strategy
- Great error messages

**config-rs Advantages:**

- Simpler API
- Fewer features (less to learn)
- Good for straightforward needs

**Recommendation:** Figment (better for production, provenance tracking valuable for debugging)

---

## 15. Summary Decision Framework

### Decision Tree Template

```
1. Is this for production?
   └─ YES → Choose mature, battle-tested options
   └─ NO → Choose fastest to prototype

2. Is performance critical?
   └─ YES → Benchmark before deciding
   └─ NO → Choose most ergonomic option

3. Is team familiar with technology?
   └─ YES → Leverage existing knowledge
   └─ NO → Evaluate learning curve

4. Is vendor lock-in acceptable?
   └─ YES → Consider integrated solutions
   └─ NO → Choose open standards (OpenTelemetry, SQL, etc.)

5. Can we start simple and evolve?
   └─ YES → Choose monolith, add complexity later
   └─ NO → Plan for complexity upfront
```

### Key Principles

1. **Start Simple:** Monolith → Modular Monolith → Microservices
2. **Measure First:** Benchmark before optimizing
3. **Standard Over Custom:** Use standards when possible
4. **Ergonomics Matter:** Developer productivity is valuable
5. **Avoid Lock-in:** Prefer portable solutions
6. **Production-Ready:** Choose mature, maintained crates
7. **Document Trade-offs:** Record why decisions were made

---

## Conclusion

This document provides the analysis needed to make informed trade-off decisions for LLM-Sentinel. Remember:

- **There are no perfect choices**, only trade-offs
- **Context matters**: What's right for MVP may differ from what's right at scale
- **Iterate**: Start with good-enough, optimize when needed
- **Measure**: Don't guess, benchmark and profile
- **Document**: Record decisions and rationale

Use this alongside TECHNICAL_STACK_RESEARCH.md for comprehensive technology evaluation.

---

**Last Updated:** November 2025
