package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"math/rand"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/segmentio/kafka-go"
)

// TelemetryEvent represents an LLM telemetry event
type TelemetryEvent struct {
	Timestamp        string                 `json:"timestamp"`
	ServiceName      string                 `json:"service_name"`
	ModelName        string                 `json:"model_name"`
	LatencyMs        float64                `json:"latency_ms"`
	PromptTokens     int                    `json:"prompt_tokens"`
	CompletionTokens int                    `json:"completion_tokens"`
	TotalTokens      int                    `json:"total_tokens"`
	CostUsd          float64                `json:"cost_usd"`
	UserID           string                 `json:"user_id"`
	SessionID        string                 `json:"session_id"`
	RequestID        string                 `json:"request_id"`
	PromptText       string                 `json:"prompt_text,omitempty"`
	ResponseText     string                 `json:"response_text,omitempty"`
	Metadata         map[string]interface{} `json:"metadata,omitempty"`
}

// TelemetryProducer sends LLM telemetry events to Kafka
type TelemetryProducer struct {
	writer *kafka.Writer
	topic  string
}

// NewTelemetryProducer creates a new telemetry producer
func NewTelemetryProducer(brokers []string, topic string) *TelemetryProducer {
	writer := &kafka.Writer{
		Addr:         kafka.TCP(brokers...),
		Topic:        topic,
		Balancer:     &kafka.LeastBytes{},
		RequiredAcks: kafka.RequireAll,
		MaxAttempts:  3,
		WriteTimeout: 10 * time.Second,
		ReadTimeout:  10 * time.Second,
	}

	log.Printf("Connected to Kafka brokers: %v", brokers)
	return &TelemetryProducer{
		writer: writer,
		topic:  topic,
	}
}

// CreateTelemetryEvent creates a telemetry event
func (p *TelemetryProducer) CreateTelemetryEvent(
	serviceName, modelName string,
	latencyMs float64,
	promptTokens, completionTokens int,
	costUsd float64,
	userID, sessionID string,
	metadata map[string]interface{},
) TelemetryEvent {
	requestID := fmt.Sprintf("req-%d-%d", time.Now().UnixMilli(), rand.Intn(10000))

	return TelemetryEvent{
		Timestamp:        time.Now().UTC().Format(time.RFC3339Nano),
		ServiceName:      serviceName,
		ModelName:        modelName,
		LatencyMs:        latencyMs,
		PromptTokens:     promptTokens,
		CompletionTokens: completionTokens,
		TotalTokens:      promptTokens + completionTokens,
		CostUsd:          costUsd,
		UserID:           userID,
		SessionID:        sessionID,
		RequestID:        requestID,
		Metadata:         metadata,
	}
}

// SendEvent sends a telemetry event to Kafka
func (p *TelemetryProducer) SendEvent(ctx context.Context, event TelemetryEvent) error {
	value, err := json.Marshal(event)
	if err != nil {
		return fmt.Errorf("failed to marshal event: %w", err)
	}

	msg := kafka.Message{
		Key:   []byte(event.RequestID),
		Value: value,
		Time:  time.Now(),
	}

	err = p.writer.WriteMessages(ctx, msg)
	if err != nil {
		return fmt.Errorf("failed to send event: %w", err)
	}

	log.Printf("Sent event %s to topic %s", event.RequestID, p.topic)
	return nil
}

// Close closes the producer
func (p *TelemetryProducer) Close() error {
	return p.writer.Close()
}

// SimulateNormalTraffic generates normal LLM traffic patterns
func SimulateNormalTraffic(ctx context.Context, producer *TelemetryProducer, numEvents int) {
	log.Printf("Simulating %d normal traffic events...", numEvents)

	models := []string{"gpt-4", "gpt-3.5-turbo", "claude-3-opus", "claude-3-sonnet"}
	services := []string{"chat-api", "completion-api", "assistant-api"}
	regions := []string{"us-east-1", "us-west-2", "eu-west-1"}

	for i := 0; i < numEvents; i++ {
		select {
		case <-ctx.Done():
			return
		default:
		}

		// Normal latency: 500-3000ms
		latencyMs := 500.0 + rand.Float64()*2500.0

		// Normal token counts
		promptTokens := 50 + rand.Intn(450)
		completionTokens := 100 + rand.Intn(700)

		// Calculate cost (example pricing)
		model := models[rand.Intn(len(models))]
		var costUsd float64
		if strings.Contains(model, "gpt-4") {
			costUsd = float64(promptTokens)*0.00003 + float64(completionTokens)*0.00006
		} else {
			costUsd = float64(promptTokens)*0.000001 + float64(completionTokens)*0.000002
		}

		event := producer.CreateTelemetryEvent(
			services[rand.Intn(len(services))],
			model,
			latencyMs,
			promptTokens,
			completionTokens,
			costUsd,
			fmt.Sprintf("user-%d", rand.Intn(100)),
			fmt.Sprintf("session-%d", rand.Intn(50)),
			map[string]interface{}{
				"region":      regions[rand.Intn(len(regions))],
				"api_version": "v1",
			},
		)

		if err := producer.SendEvent(ctx, event); err != nil {
			log.Printf("Error sending event: %v", err)
		}

		time.Sleep(100 * time.Millisecond)
	}
}

// SimulateAnomalousTraffic generates anomalous LLM traffic patterns
func SimulateAnomalousTraffic(ctx context.Context, producer *TelemetryProducer, numEvents int) {
	log.Printf("Simulating %d anomalous traffic events...", numEvents)

	anomalyTypes := []struct {
		Type        string
		Description string
	}{
		{"high_latency", "Extremely high latency"},
		{"high_tokens", "Unusually high token count"},
		{"high_cost", "Abnormally high cost"},
		{"suspicious_pattern", "Suspicious usage pattern"},
	}

	for i := 0; i < numEvents; i++ {
		select {
		case <-ctx.Done():
			return
		default:
		}

		anomaly := anomalyTypes[rand.Intn(len(anomalyTypes))]
		var latencyMs float64
		var promptTokens, completionTokens int

		switch anomaly.Type {
		case "high_latency":
			// Anomalous: 20-60 seconds
			latencyMs = 20000.0 + rand.Float64()*40000.0
			promptTokens = 100 + rand.Intn(400)
			completionTokens = 200 + rand.Intn(600)

		case "high_tokens":
			// Anomalous: very high token count
			latencyMs = 5000.0 + rand.Float64()*10000.0
			promptTokens = 5000 + rand.Intn(10000)
			completionTokens = 8000 + rand.Intn(12000)

		case "high_cost":
			// Anomalous: extremely high cost
			latencyMs = 8000.0 + rand.Float64()*12000.0
			promptTokens = 8000 + rand.Intn(7000)
			completionTokens = 10000 + rand.Intn(15000)

		default: // suspicious_pattern
			// Multiple rapid requests from same user
			latencyMs = 1000.0 + rand.Float64()*2000.0
			promptTokens = 50 + rand.Intn(150)
			completionTokens = 50 + rand.Intn(150)
		}

		costUsd := float64(promptTokens)*0.00003 + float64(completionTokens)*0.00006

		event := producer.CreateTelemetryEvent(
			"chat-api",
			"gpt-4",
			latencyMs,
			promptTokens,
			completionTokens,
			costUsd,
			"user-suspicious",
			fmt.Sprintf("session-anomaly-%d", i),
			map[string]interface{}{
				"anomaly_type": anomaly.Type,
				"description":  anomaly.Description,
				"simulated":    true,
			},
		)

		if err := producer.SendEvent(ctx, event); err != nil {
			log.Printf("Error sending event: %v", err)
		}

		log.Printf("Sent anomalous event: %s", anomaly.Type)
		time.Sleep(500 * time.Millisecond)
	}
}

func main() {
	brokersFlag := flag.String("brokers", "localhost:9092", "Comma-separated list of Kafka brokers")
	topicFlag := flag.String("topic", "llm.telemetry", "Kafka topic name")
	normalEvents := flag.Int("normal-events", 20, "Number of normal events to generate")
	anomalousEvents := flag.Int("anomalous-events", 5, "Number of anomalous events to generate")
	continuous := flag.Bool("continuous", false, "Run continuously")
	flag.Parse()

	rand.Seed(time.Now().UnixNano())

	brokers := strings.Split(*brokersFlag, ",")
	producer := NewTelemetryProducer(brokers, *topicFlag)
	defer producer.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Handle graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-sigChan
		log.Println("Received interrupt signal, shutting down...")
		cancel()
	}()

	if *continuous {
		log.Println("Running in continuous mode (Ctrl+C to stop)...")
		for {
			select {
			case <-ctx.Done():
				log.Println("Shutting down...")
				return
			default:
				SimulateNormalTraffic(ctx, producer, *normalEvents)
				SimulateAnomalousTraffic(ctx, producer, *anomalousEvents)
				log.Println("Waiting 10 seconds before next batch...")
				time.Sleep(10 * time.Second)
			}
		}
	} else {
		SimulateNormalTraffic(ctx, producer, *normalEvents)
		SimulateAnomalousTraffic(ctx, producer, *anomalousEvents)
		log.Println("Finished generating events")
	}
}
