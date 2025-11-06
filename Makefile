.PHONY: help build test lint format clean docker k8s-deploy k8s-delete dev-up dev-down

# Default target
.DEFAULT_GOAL := help

# Variables
BINARY_NAME := sentinel
DOCKER_IMAGE := sentinel
DOCKER_TAG := latest
NAMESPACE := sentinel

## help: Show this help message
help:
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' | sed -e 's/^/ /'

## build: Build release binary
build:
	@echo "Building release binary..."
	cargo build --release --bin $(BINARY_NAME)
	@echo "Binary: target/release/$(BINARY_NAME)"

## build-dev: Build debug binary
build-dev:
	@echo "Building debug binary..."
	cargo build --bin $(BINARY_NAME)

## test: Run all tests
test:
	@echo "Running tests..."
	cargo test --workspace --all-features

## test-integration: Run integration tests
test-integration:
	@echo "Running integration tests..."
	docker-compose up -d kafka influxdb rabbitmq redis
	@sleep 10
	cargo test --workspace --test '*'
	docker-compose down

## bench: Run benchmarks
bench:
	@echo "Running benchmarks..."
	cargo bench --workspace

## lint: Run clippy linter
lint:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

## format: Format code with rustfmt
format:
	@echo "Formatting code..."
	cargo fmt --all

## format-check: Check code formatting
format-check:
	@echo "Checking formatting..."
	cargo fmt --all -- --check

## audit: Run security audit
audit:
	@echo "Running security audit..."
	cargo audit
	cargo deny check

## clean: Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf target/

## coverage: Generate code coverage report
coverage:
	@echo "Generating coverage report..."
	cargo tarpaulin --workspace --all-features --out Html
	@echo "Coverage report: tarpaulin-report.html"

## docker-build: Build Docker image
docker-build:
	@echo "Building Docker image..."
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) .

## docker-run: Run Docker container
docker-run: docker-build
	@echo "Running Docker container..."
	docker run -d \
		--name sentinel \
		-p 8080:8080 \
		-p 9090:9090 \
		-e SENTINEL_LOG_LEVEL=debug \
		$(DOCKER_IMAGE):$(DOCKER_TAG)

## docker-stop: Stop and remove Docker container
docker-stop:
	@echo "Stopping Docker container..."
	docker stop sentinel || true
	docker rm sentinel || true

## dev-up: Start development environment
dev-up:
	@echo "Starting development environment..."
	docker-compose up -d
	@echo "Waiting for services to be ready..."
	@sleep 15
	@echo "Services ready!"
	@echo "Grafana:    http://localhost:3000 (admin/admin)"
	@echo "Prometheus: http://localhost:9090"
	@echo "Kafka UI:   http://localhost:8081"
	@echo "RabbitMQ:   http://localhost:15672 (admin/adminpass)"
	@echo "InfluxDB:   http://localhost:8086"

## dev-down: Stop development environment
dev-down:
	@echo "Stopping development environment..."
	docker-compose down

## dev-logs: View development environment logs
dev-logs:
	docker-compose logs -f

## dev-clean: Clean development environment (removes volumes)
dev-clean:
	@echo "Cleaning development environment..."
	docker-compose down -v

## k8s-deploy: Deploy to Kubernetes
k8s-deploy:
	@echo "Deploying to Kubernetes..."
	kubectl apply -k k8s/
	kubectl rollout status deployment/sentinel -n $(NAMESPACE) --timeout=5m

## k8s-delete: Delete Kubernetes deployment
k8s-delete:
	@echo "Deleting Kubernetes deployment..."
	kubectl delete -k k8s/

## k8s-logs: View Kubernetes logs
k8s-logs:
	kubectl logs -f deployment/sentinel -n $(NAMESPACE)

## k8s-status: Check Kubernetes deployment status
k8s-status:
	@echo "Deployment status:"
	kubectl get all -n $(NAMESPACE)
	@echo "\nPod status:"
	kubectl get pods -n $(NAMESPACE) -o wide
	@echo "\nHPA status:"
	kubectl get hpa -n $(NAMESPACE)

## k8s-port-forward: Port forward to Kubernetes service
k8s-port-forward:
	@echo "Port forwarding to sentinel service..."
	kubectl port-forward svc/sentinel 8080:8080 9090:9090 -n $(NAMESPACE)

## run: Run sentinel locally
run: build
	@echo "Running sentinel..."
	./target/release/$(BINARY_NAME) --config config/sentinel.yaml

## run-dev: Run sentinel in development mode
run-dev: build-dev
	@echo "Running sentinel (debug mode)..."
	RUST_LOG=debug ./target/debug/$(BINARY_NAME) --config config/sentinel.yaml

## install: Install binary to system
install: build
	@echo "Installing binary to /usr/local/bin..."
	sudo cp target/release/$(BINARY_NAME) /usr/local/bin/
	@echo "Installed: /usr/local/bin/$(BINARY_NAME)"

## uninstall: Uninstall binary from system
uninstall:
	@echo "Uninstalling binary..."
	sudo rm -f /usr/local/bin/$(BINARY_NAME)

## ci: Run CI checks locally
ci: format-check lint test audit
	@echo "All CI checks passed!"

## release: Create release build
release:
	@echo "Creating release build..."
	cargo build --release --bin $(BINARY_NAME)
	strip target/release/$(BINARY_NAME)
	@echo "Release binary: target/release/$(BINARY_NAME)"
	@ls -lh target/release/$(BINARY_NAME)
