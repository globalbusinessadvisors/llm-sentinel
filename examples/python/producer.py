#!/usr/bin/env python3
"""
LLM-Sentinel Python Telemetry Producer Example

This example demonstrates how to send LLM telemetry events to Kafka
for processing by LLM-Sentinel.

Requirements:
    pip install kafka-python

Usage:
    python producer.py --brokers localhost:9092 --topic llm.telemetry
"""

import argparse
import json
import logging
import random
import time
from datetime import datetime, timezone
from typing import Dict, Any
from kafka import KafkaProducer

logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class LLMTelemetryProducer:
    """Producer for sending LLM telemetry events to Kafka."""

    def __init__(self, brokers: list[str], topic: str):
        """Initialize the telemetry producer.

        Args:
            brokers: List of Kafka broker addresses
            topic: Kafka topic name
        """
        self.topic = topic
        self.producer = KafkaProducer(
            bootstrap_servers=brokers,
            value_serializer=lambda v: json.dumps(v).encode('utf-8'),
            acks='all',
            retries=3,
            max_in_flight_requests_per_connection=1,
        )
        logger.info(f"Connected to Kafka brokers: {brokers}")

    def create_telemetry_event(
        self,
        service_name: str,
        model_name: str,
        latency_ms: float,
        prompt_tokens: int,
        completion_tokens: int,
        cost_usd: float,
        user_id: str = "user-123",
        session_id: str = "session-456",
        request_id: str = None,
        prompt_text: str = None,
        response_text: str = None,
        metadata: Dict[str, Any] = None,
    ) -> Dict[str, Any]:
        """Create a telemetry event payload.

        Args:
            service_name: Name of the service generating the event
            model_name: LLM model name (e.g., "gpt-4", "claude-3")
            latency_ms: Request latency in milliseconds
            prompt_tokens: Number of tokens in the prompt
            completion_tokens: Number of tokens in the completion
            cost_usd: Cost of the request in USD
            user_id: User identifier
            session_id: Session identifier
            request_id: Unique request identifier
            prompt_text: Optional prompt text (will be sanitized)
            response_text: Optional response text (will be sanitized)
            metadata: Additional metadata

        Returns:
            Telemetry event dictionary
        """
        if request_id is None:
            request_id = f"req-{int(time.time() * 1000)}-{random.randint(1000, 9999)}"

        event = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "service_name": service_name,
            "model_name": model_name,
            "latency_ms": latency_ms,
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens,
            "cost_usd": cost_usd,
            "user_id": user_id,
            "session_id": session_id,
            "request_id": request_id,
        }

        if prompt_text:
            event["prompt_text"] = prompt_text

        if response_text:
            event["response_text"] = response_text

        if metadata:
            event["metadata"] = metadata

        return event

    def send_event(self, event: Dict[str, Any]) -> None:
        """Send a telemetry event to Kafka.

        Args:
            event: Telemetry event dictionary
        """
        try:
            future = self.producer.send(self.topic, value=event)
            record_metadata = future.get(timeout=10)
            logger.info(
                f"Sent event {event['request_id']} to {record_metadata.topic} "
                f"partition {record_metadata.partition} offset {record_metadata.offset}"
            )
        except Exception as e:
            logger.error(f"Failed to send event: {e}")
            raise

    def close(self):
        """Close the producer and flush pending messages."""
        self.producer.flush()
        self.producer.close()
        logger.info("Producer closed")


def simulate_normal_traffic(producer: LLMTelemetryProducer, num_events: int = 10):
    """Simulate normal LLM traffic patterns.

    Args:
        producer: Telemetry producer instance
        num_events: Number of events to generate
    """
    logger.info(f"Simulating {num_events} normal traffic events...")

    models = ["gpt-4", "gpt-3.5-turbo", "claude-3-opus", "claude-3-sonnet"]
    services = ["chat-api", "completion-api", "assistant-api"]

    for i in range(num_events):
        # Normal latency: 500-3000ms
        latency_ms = random.uniform(500, 3000)

        # Normal token counts
        prompt_tokens = random.randint(50, 500)
        completion_tokens = random.randint(100, 800)

        # Calculate cost (example pricing)
        model = random.choice(models)
        if "gpt-4" in model:
            cost_usd = (prompt_tokens * 0.00003 + completion_tokens * 0.00006)
        else:
            cost_usd = (prompt_tokens * 0.000001 + completion_tokens * 0.000002)

        event = producer.create_telemetry_event(
            service_name=random.choice(services),
            model_name=model,
            latency_ms=latency_ms,
            prompt_tokens=prompt_tokens,
            completion_tokens=completion_tokens,
            cost_usd=cost_usd,
            user_id=f"user-{random.randint(1, 100)}",
            session_id=f"session-{random.randint(1, 50)}",
            metadata={
                "region": random.choice(["us-east-1", "us-west-2", "eu-west-1"]),
                "api_version": "v1",
            }
        )

        producer.send_event(event)
        time.sleep(0.1)  # 100ms between events


def simulate_anomalous_traffic(producer: LLMTelemetryProducer, num_events: int = 5):
    """Simulate anomalous LLM traffic patterns.

    Args:
        producer: Telemetry producer instance
        num_events: Number of anomalous events to generate
    """
    logger.info(f"Simulating {num_events} anomalous traffic events...")

    anomaly_types = [
        ("high_latency", "Extremely high latency"),
        ("high_tokens", "Unusually high token count"),
        ("high_cost", "Abnormally high cost"),
        ("suspicious_pattern", "Suspicious usage pattern"),
    ]

    for i in range(num_events):
        anomaly_type, description = random.choice(anomaly_types)

        if anomaly_type == "high_latency":
            # Anomalous: 20-60 seconds
            latency_ms = random.uniform(20000, 60000)
            prompt_tokens = random.randint(100, 500)
            completion_tokens = random.randint(200, 800)
        elif anomaly_type == "high_tokens":
            # Anomalous: very high token count
            latency_ms = random.uniform(5000, 15000)
            prompt_tokens = random.randint(5000, 15000)
            completion_tokens = random.randint(8000, 20000)
        elif anomaly_type == "high_cost":
            # Anomalous: extremely high cost
            latency_ms = random.uniform(8000, 20000)
            prompt_tokens = random.randint(8000, 15000)
            completion_tokens = random.randint(10000, 25000)
        else:  # suspicious_pattern
            # Multiple rapid requests from same user
            latency_ms = random.uniform(1000, 3000)
            prompt_tokens = random.randint(50, 200)
            completion_tokens = random.randint(50, 200)

        cost_usd = (prompt_tokens * 0.00003 + completion_tokens * 0.00006)

        event = producer.create_telemetry_event(
            service_name="chat-api",
            model_name="gpt-4",
            latency_ms=latency_ms,
            prompt_tokens=prompt_tokens,
            completion_tokens=completion_tokens,
            cost_usd=cost_usd,
            user_id="user-suspicious",
            session_id=f"session-anomaly-{i}",
            metadata={
                "anomaly_type": anomaly_type,
                "description": description,
                "simulated": True,
            }
        )

        producer.send_event(event)
        logger.warning(f"Sent anomalous event: {anomaly_type}")
        time.sleep(0.5)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="LLM-Sentinel telemetry producer example")
    parser.add_argument(
        "--brokers",
        default="localhost:9092",
        help="Comma-separated list of Kafka brokers (default: localhost:9092)"
    )
    parser.add_argument(
        "--topic",
        default="llm.telemetry",
        help="Kafka topic name (default: llm.telemetry)"
    )
    parser.add_argument(
        "--normal-events",
        type=int,
        default=20,
        help="Number of normal events to generate (default: 20)"
    )
    parser.add_argument(
        "--anomalous-events",
        type=int,
        default=5,
        help="Number of anomalous events to generate (default: 5)"
    )
    parser.add_argument(
        "--continuous",
        action="store_true",
        help="Run continuously"
    )

    args = parser.parse_args()

    brokers = args.brokers.split(",")
    producer = LLMTelemetryProducer(brokers=brokers, topic=args.topic)

    try:
        if args.continuous:
            logger.info("Running in continuous mode (Ctrl+C to stop)...")
            while True:
                simulate_normal_traffic(producer, args.normal_events)
                simulate_anomalous_traffic(producer, args.anomalous_events)
                logger.info("Waiting 10 seconds before next batch...")
                time.sleep(10)
        else:
            simulate_normal_traffic(producer, args.normal_events)
            simulate_anomalous_traffic(producer, args.anomalous_events)
            logger.info("Finished generating events")

    except KeyboardInterrupt:
        logger.info("Interrupted by user")
    finally:
        producer.close()


if __name__ == "__main__":
    main()
