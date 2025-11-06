# Python Telemetry Producer Example

This example demonstrates how to send LLM telemetry events to Kafka for processing by LLM-Sentinel.

## Installation

```bash
pip install -r requirements.txt
```

## Usage

### Basic Usage

Generate 20 normal events and 5 anomalous events:

```bash
python producer.py --brokers localhost:9092 --topic llm.telemetry
```

### Continuous Mode

Run continuously to simulate ongoing traffic:

```bash
python producer.py --brokers localhost:9092 --continuous
```

### Custom Configuration

```bash
python producer.py \
  --brokers kafka-0:9092,kafka-1:9092,kafka-2:9092 \
  --topic llm.telemetry \
  --normal-events 50 \
  --anomalous-events 10
```

## Event Schema

The producer generates events with the following schema:

```json
{
  "timestamp": "2024-01-15T10:30:45.123456Z",
  "service_name": "chat-api",
  "model_name": "gpt-4",
  "latency_ms": 1234.56,
  "prompt_tokens": 150,
  "completion_tokens": 300,
  "total_tokens": 450,
  "cost_usd": 0.0135,
  "user_id": "user-123",
  "session_id": "session-456",
  "request_id": "req-1234567890-5678",
  "metadata": {
    "region": "us-east-1",
    "api_version": "v1"
  }
}
```

## Simulated Anomalies

The producer simulates the following types of anomalies:

1. **High Latency**: Requests taking 20-60 seconds (normal: 0.5-3 seconds)
2. **High Tokens**: Requests using 13,000-35,000 tokens (normal: 150-1,300)
3. **High Cost**: Requests costing $0.30-$0.75 (normal: $0.002-$0.03)
4. **Suspicious Pattern**: Multiple rapid requests from the same user

## Integration with Your Application

To integrate with your LLM application:

```python
from producer import LLMTelemetryProducer

# Initialize producer
producer = LLMTelemetryProducer(
    brokers=["kafka:9092"],
    topic="llm.telemetry"
)

# After each LLM call
event = producer.create_telemetry_event(
    service_name="my-service",
    model_name="gpt-4",
    latency_ms=response_time_ms,
    prompt_tokens=usage.prompt_tokens,
    completion_tokens=usage.completion_tokens,
    cost_usd=calculated_cost,
    user_id=user.id,
    session_id=session.id,
)

producer.send_event(event)
```

## Environment Variables

You can also use environment variables:

```bash
export KAFKA_BROKERS="localhost:9092"
export KAFKA_TOPIC="llm.telemetry"

python producer.py
```

## Testing with Docker Compose

If you're using the docker-compose.yaml from the repository:

```bash
# Start infrastructure
docker-compose up -d

# Run producer
python producer.py --brokers localhost:29092 --continuous
```

## Notes

- The producer uses `acks=all` for guaranteed delivery
- Events are serialized as JSON
- Sensitive fields (prompt_text, response_text) are optional and will be sanitized by Sentinel
- The producer implements automatic retries for transient failures
