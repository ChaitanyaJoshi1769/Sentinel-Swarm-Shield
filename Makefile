.PHONY: help deps build test docker-build dev-up dev-down clean

# Default target
help:
	@echo "Sentinel-Swarm-Shield Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  deps              Install all dependencies"
	@echo "  protos            Generate protobuf files"
	@echo "  build             Build all services"
	@echo "  build-fusion      Build Rust fusion engine"
	@echo "  build-orchestrator Build Go orchestrator"
	@echo "  build-api-gateway Build Go API gateway"
	@echo "  build-ai          Build Python AI services"
	@echo "  test              Run all tests"
	@echo "  test-fusion       Run Rust fusion tests"
	@echo "  test-orchestrator Run Go orchestrator tests"
	@echo "  test-api          Run API gateway tests"
	@echo "  docker-build      Build all Docker images"
	@echo "  dev-up            Start local dev environment (Docker Compose)"
	@echo "  dev-down          Stop local dev environment"
	@echo "  dev-logs          View docker-compose logs"
	@echo "  frontend-dev      Start Next.js frontend in dev mode"
	@echo "  clean             Clean build artifacts"
	@echo "  lint              Run linters on all code"
	@echo "  format            Format all code"

# Install dependencies
deps:
	@echo "Installing dependencies..."
	cd backend/fusion-engine && cargo fetch
	cd backend/orchestrator && go mod download
	cd backend/api-gateway && go mod download
	cd backend/ai-services && pip install -q -r requirements.txt
	cd frontend/apps/command-center && npm install

# Generate protobuf files
protos:
	@echo "Generating protobuf files..."
	mkdir -p backend/fusion-engine/src/generated
	mkdir -p backend/orchestrator/pkg/generated
	mkdir -p backend/api-gateway/pkg/generated
	mkdir -p backend/ai-services/generated
	mkdir -p frontend/apps/command-center/src/generated

	# Rust
	protoc --rust_out=backend/fusion-engine/src/generated \
		--prost-rpc_out=backend/fusion-engine/src/generated \
		backend/shared/protos/*.proto

	# Go
	protoc --go_out=backend/shared --go-grpc_out=backend/shared \
		backend/shared/protos/*.proto

	# Python
	python -m grpc_tools.protoc \
		-I backend/shared/protos \
		--python_out=backend/ai-services/generated \
		--grpc_python_out=backend/ai-services/generated \
		backend/shared/protos/*.proto

	@echo "✓ Protobuf generation complete"

# Build all services
build: protos
	@echo "Building all services..."
	$(MAKE) build-fusion
	$(MAKE) build-orchestrator
	$(MAKE) build-api-gateway
	$(MAKE) build-ai
	@echo "✓ All services built"

# Build Rust fusion engine
build-fusion:
	@echo "Building Rust fusion engine..."
	cd backend/fusion-engine && cargo build --release
	@echo "✓ Fusion engine built"

# Build Go orchestrator
build-orchestrator:
	@echo "Building Go orchestrator..."
	cd backend/orchestrator && go build -o bin/orchestrator ./cmd/server
	@echo "✓ Orchestrator built"

# Build Go API gateway
build-api-gateway:
	@echo "Building Go API gateway..."
	cd backend/api-gateway && go build -o bin/api-gateway ./cmd/server
	@echo "✓ API gateway built"

# Build Python AI services
build-ai:
	@echo "Building Python AI services..."
	cd backend/ai-services && pip install -q -e .
	@echo "✓ AI services ready"

# Run tests
test: protos
	@echo "Running all tests..."
	$(MAKE) test-fusion
	$(MAKE) test-orchestrator
	$(MAKE) test-api
	@echo "✓ All tests passed"

# Test Rust fusion engine
test-fusion:
	@echo "Testing Rust fusion engine..."
	cd backend/fusion-engine && cargo test --release
	@echo "✓ Fusion engine tests passed"

# Test Go orchestrator
test-orchestrator:
	@echo "Testing Go orchestrator..."
	cd backend/orchestrator && go test ./...
	@echo "✓ Orchestrator tests passed"

# Test Go API gateway
test-api:
	@echo "Testing Go API gateway..."
	cd backend/api-gateway && go test ./...
	@echo "✓ API gateway tests passed"

# Build Docker images
docker-build:
	@echo "Building Docker images..."
	docker build -f infra/docker/Dockerfile.fusion-engine -t sentinel/fusion-engine:latest backend/fusion-engine
	docker build -f infra/docker/Dockerfile.orchestrator -t sentinel/orchestrator:latest backend/orchestrator
	docker build -f infra/docker/Dockerfile.api-gateway -t sentinel/api-gateway:latest backend/api-gateway
	docker build -f infra/docker/Dockerfile.ai-services -t sentinel/ai-services:latest backend/ai-services
	docker build -f infra/docker/Dockerfile.command-center -t sentinel/command-center:latest frontend/apps/command-center
	@echo "✓ Docker images built"

# Start development environment
dev-up:
	@echo "Starting local development environment..."
	docker-compose -f infra/docker/dev-compose.yml up -d
	@echo ""
	@echo "✓ Development environment started!"
	@echo "  API Gateway (GraphQL): http://localhost:4000/graphql"
	@echo "  Command Center:        http://localhost:3000"
	@echo "  PostgreSQL:            localhost:5432"
	@echo "  Kafka:                 localhost:9092"
	@echo "  Redis:                 localhost:6379"
	@echo ""
	@echo "View logs: make dev-logs"
	@echo "Stop:      make dev-down"

# Stop development environment
dev-down:
	@echo "Stopping local development environment..."
	docker-compose -f infra/docker/dev-compose.yml down
	@echo "✓ Development environment stopped"

# View development logs
dev-logs:
	docker-compose -f infra/docker/dev-compose.yml logs -f

# Start frontend in dev mode
frontend-dev:
	@echo "Starting Next.js frontend in dev mode..."
	cd frontend/apps/command-center && npm run dev

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cd backend/fusion-engine && cargo clean
	cd backend/orchestrator && rm -rf bin/
	cd backend/api-gateway && rm -rf bin/
	cd backend/ai-services && rm -rf build/ dist/ *.egg-info
	cd frontend/apps/command-center && rm -rf .next/ node_modules/
	find . -name "*.pb.go" -delete
	find . -name "*.pb.rs" -delete
	find . -name "*_pb2.py" -delete
	@echo "✓ Clean complete"

# Linting
lint:
	@echo "Running linters..."
	cd backend/fusion-engine && cargo clippy --all-targets
	cd backend/orchestrator && golangci-lint run ./...
	cd backend/api-gateway && golangci-lint run ./...
	cd backend/ai-services && pylint --disable=all --enable=E,F backend/ai-services/
	@echo "✓ Linting complete"

# Code formatting
format:
	@echo "Formatting code..."
	cd backend/fusion-engine && cargo fmt
	cd backend/orchestrator && gofmt -s -w .
	cd backend/api-gateway && gofmt -s -w .
	cd backend/ai-services && black .
	cd frontend/apps/command-center && npm run format
	@echo "✓ Formatting complete"

# Development environment setup
setup: deps protos build
	@echo ""
	@echo "✓ Development environment ready!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Start local services: make dev-up"
	@echo "  2. In another terminal:  make frontend-dev"
	@echo "  3. Open http://localhost:3000"
	@echo ""

.PHONY: $(MAKECMDGOALS)
