# Development Guide

## Prerequisites

- **Docker & Docker Compose** (for local dev environment)
- **Git** (for version control)
- **Make** (for build tasks)
- **Node.js 18+** (for frontend development)
- **Rust 1.70+** (for fusion engine)
- **Go 1.21+** (for orchestrator and API gateway)
- **Python 3.11+** (for AI services)

## Quick Start (5 minutes)

```bash
# Clone the repo
git clone https://github.com/ChaitanyaJoshi1769/Sentinel-Swarm-Shield
cd Sentinel-Swarm-Shield

# Set up development environment
make setup

# Start all services
make dev-up

# In another terminal, start the frontend
make frontend-dev

# Open http://localhost:3000 in your browser
```

Done! You now have:
- **API Gateway** running on `http://localhost:4000/graphql`
- **Command Center UI** running on `http://localhost:3000`
- **PostgreSQL** on `localhost:5432`
- **Kafka** on `localhost:9092`
- **Redis** on `localhost:6379`

## Development Workflow

### Building Services

```bash
# Build all services
make build

# Build specific service
make build-fusion
make build-orchestrator
make build-api-gateway
make build-ai
```

### Running Tests

```bash
# Run all tests
make test

# Test specific service
make test-fusion
make test-orchestrator
make test-api
```

### Starting/Stopping Dev Environment

```bash
# Start all services
make dev-up

# View logs
make dev-logs

# Stop services
make dev-down
```

### Code Formatting

```bash
# Format all code
make format

# Run linters
make lint
```

## Service Architecture for Development

### Rust Fusion Engine (`backend/fusion-engine/`)

**What it does**: Ingests multi-sensor data, performs real-time tracking and swarm clustering.

**Build**:
```bash
cd backend/fusion-engine
cargo build --release
cargo test
```

**Run locally** (requires PostgreSQL + Kafka):
```bash
POSTGRES_URL=postgresql://... cargo run --release
```

**Key files**:
- `src/main.rs` - Entry point and gRPC server
- `src/fusion/` - Core fusion algorithms
- `src/track/` - Track state management
- `src/kafka/` - Event publishing

### Python AI Services (`backend/ai-services/`)

**What it does**: Drone classification, threat prediction, anomaly detection.

**Build**:
```bash
cd backend/ai-services
pip install -r requirements.txt
pip install -e .
```

**Run locally**:
```bash
python -m uvicorn main:app --reload
```

**Key files**:
- `classification/` - Drone type classification
- `prediction/` - Trajectory and threat prediction
- `anomaly/` - Anomaly detection
- `api/` - FastAPI endpoints

### Go Orchestrator (`backend/orchestrator/`)

**What it does**: Threat prioritization, interceptor assignment, defense coordination.

**Build**:
```bash
cd backend/orchestrator
go build -o bin/orchestrator ./cmd/server
go test ./...
```

**Run locally**:
```bash
go run ./cmd/server
```

**Key files**:
- `cmd/server/main.go` - Entry point
- `pkg/threat/` - Threat prioritization
- `pkg/engagement/` - Engagement planning
- `pkg/coordination/` - Multi-site coordination

### Go API Gateway (`backend/api-gateway/`)

**What it does**: GraphQL endpoint, gRPC reflection, WebSocket subscriptions.

**Build**:
```bash
cd backend/api-gateway
go build -o bin/api-gateway ./cmd/server
```

**Run locally**:
```bash
go run ./cmd/server
```

**GraphQL endpoint**: `http://localhost:4000/graphql`

**Key files**:
- `cmd/server/main.go` - Entry point
- `pkg/graphql/` - GraphQL resolvers
- `pkg/websocket/` - WebSocket handler

### Next.js Command Center (`frontend/apps/command-center/`)

**What it does**: 3D tactical UI, real-time track visualization, operator controls.

**Build**:
```bash
cd frontend/apps/command-center
npm install
npm run build
```

**Run in dev mode** (hot reload):
```bash
npm run dev
```

**Open**: `http://localhost:3000`

**Key files**:
- `src/pages/` - Page components
- `src/components/Map/` - 3D airspace map
- `src/components/Panels/` - Control panels
- `src/hooks/` - GraphQL subscriptions

## Database Setup

### PostgreSQL Initialization

```sql
-- Create main database (done automatically in dev-compose.yml)
CREATE DATABASE sentinel;

-- Create schemas
CREATE SCHEMA tracks;
CREATE SCHEMA threats;
CREATE SCHEMA engagements;
CREATE SCHEMA audit;

-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create hypertables for time-series data
SELECT create_hypertable('tracks.raw', 'timestamp', if_not_exists => TRUE);
SELECT create_hypertable('threats.events', 'created_at', if_not_exists => TRUE);
```

### Connecting to PostgreSQL

```bash
# From your machine
psql -h localhost -U sentinel -d sentinel

# Inside Docker container
docker exec -it sentinel-postgres psql -U sentinel -d sentinel
```

## Testing

### Unit Tests

```bash
# Rust
cd backend/fusion-engine
cargo test

# Go
cd backend/orchestrator
go test ./...

# Python
cd backend/ai-services
pytest
```

### Integration Tests

```bash
# Run full integration test suite
make test
```

### Manual API Testing

```bash
# Using curl + jq
curl -X POST http://localhost:4000/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "query { tracks(limit: 10) { id latitude longitude } }"}'

# Using GraphQL client (graphiql)
# Open http://localhost:4000/graphql in your browser
```

## Debugging

### View Service Logs

```bash
# All services
make dev-logs

# Specific service
docker logs -f sentinel-fusion-engine
docker logs -f sentinel-api-gateway

# Follow logs
docker logs -f --tail=100 sentinel-api-gateway
```

### Enable Debug Logging

```bash
# Set log level for services
docker-compose -f infra/docker/dev-compose.yml down
RUST_LOG=debug docker-compose -f infra/docker/dev-compose.yml up
```

### Database Debugging

```bash
# Connect to PostgreSQL
docker exec -it sentinel-postgres psql -U sentinel -d sentinel

# Common queries
SELECT COUNT(*) FROM tracks;
SELECT * FROM threats ORDER BY created_at DESC LIMIT 10;
```

### Kafka Message Inspection

```bash
# List topics
docker exec sentinel-kafka kafka-topics --list --bootstrap-server kafka:9092

# Consume messages from a topic
docker exec sentinel-kafka kafka-console-consumer \
  --bootstrap-server kafka:9092 \
  --topic tracks.updated \
  --from-beginning
```

## Code Style & Standards

### Rust
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions

### Go
- Use `gofmt` for formatting
- Use `golangci-lint` for linting
- Follow Go idioms (CamelCase, etc.)

### Python
- Use `black` for formatting
- Use `pylint` for linting
- Follow PEP 8 style guide

### TypeScript/React
- Use `prettier` for formatting
- Use `eslint` for linting
- Follow React best practices

## Common Tasks

### Adding a New Protobuf Message

1. Edit `backend/shared/protos/*.proto`
2. Run `make protos` to regenerate
3. Update service implementations

### Adding a New GraphQL Query

1. Edit `backend/api-gateway/pkg/graphql/schema.graphql`
2. Implement resolver in Go
3. Test with GraphQL client

### Adding a New Database Table

1. Create schema in PostgreSQL
2. Update Go models
3. Add queries in relevant service
4. Write tests

### Deploying a New Model Version

1. Train model locally
2. Export to ONNX format
3. Upload to `backend/ai-services/models/`
4. Update model loader in Python service
5. Test inference latency

## Troubleshooting

### Services Won't Start

```bash
# Check service health
docker ps

# Check logs
docker logs sentinel-postgres
docker logs sentinel-kafka

# Restart everything
make dev-down
make dev-up
```

### PostgreSQL Connection Error

```bash
# Verify PostgreSQL is running and healthy
docker ps | grep postgres

# Check credentials in dev-compose.yml
# Default: user=sentinel password=sentinel_dev_password

# Test connection
docker exec sentinel-postgres psql -U sentinel -c "SELECT 1"
```

### Kafka Issues

```bash
# Check if Kafka is running
docker exec sentinel-kafka kafka-broker-api-versions --bootstrap-server localhost:9092

# Reset topics (development only!)
docker exec sentinel-kafka kafka-topics --delete --bootstrap-server kafka:9092 --topic tracks.updated
```

### Frontend Build Errors

```bash
# Clear Next.js cache
cd frontend/apps/command-center
rm -rf .next
npm run build

# Update dependencies
npm update
```

## Performance Profiling

### Rust (fusino engine)

```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --release
```

### Go

```bash
# Profile API gateway
go tool pprof http://localhost:6060/debug/pprof/profile
```

### Python

```bash
# Profile AI services
pip install py-spy
py-spy record -o profile.svg -- python -m uvicorn main:app
```

## Documentation

- **Architecture**: See `docs/ARCHITECTURE.md`
- **ADRs**: See `docs/adrs/`
- **Deployment**: See `docs/DEPLOYMENT.md`
- **Security**: See `docs/THREAT_MODEL.md`

## Contributing

1. Create a feature branch from `main`
2. Make changes with tests
3. Format code (`make format`)
4. Run tests (`make test`)
5. Submit pull request

## Getting Help

- Check existing GitHub issues
- Read architecture documentation
- Look at similar components
- Ask in discussions

---

**Happy developing!** 🚀

For questions, see [ARCHITECTURE.md](ARCHITECTURE.md) or the ADRs in `docs/adrs/`.
