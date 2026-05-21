# Sentinel-Swarm-Shield: System Architecture

## Table of Contents

1. [System Overview](#system-overview)
2. [Core Components](#core-components)
3. [Data Flow](#data-flow)
4. [Service Architecture](#service-architecture)
5. [Technology Decisions](#technology-decisions)
6. [Scalability & Performance](#scalability--performance)
7. [Security Architecture](#security-architecture)
8. [Deployment Architectures](#deployment-architectures)
9. [Integration Points](#integration-points)

---

## System Overview

### Mission
Create a distributed, software-defined, AI-native operating system for detecting, tracking, classifying, predicting, coordinating against, and neutralizing hostile drone swarms in real time.

### Architecture Principles

1. **Real-time First**: Sub-100ms end-to-end latency for threat detection to engagement
2. **Edge-First**: Sensor fusion and initial threat assessment happen at the edge
3. **Autonomous-Ready**: AI-driven decision making with human-in-the-loop approval gates
4. **Distributed by Default**: No single point of failure; graceful degradation
5. **Defense-Grade Security**: Zero-trust, encrypted, immutable audit trails
6. **Modular & Composable**: Plugin architecture for defense mechanisms, sensors, integrations
7. **Operator-Centric**: Cinematic UI with clear threat visualization and control

---

## Core Components

### Three-Tier Layered Architecture

```
┌─────────────────────────────────────────────────┐
│          COMMAND CENTER & CONTROL               │
│  (Command-Center UI, Analyst Console, Dashboards)
└────────────────┬────────────────────────────────┘
                 │ WebSocket, GraphQL, gRPC
                 ↓
┌─────────────────────────────────────────────────┐
│         API GATEWAY & CONTROL PLANE              │
│  (GraphQL, gRPC, REST, WebSocket, Auth/Authz)  │
└────────────────┬────────────────────────────────┘
                 │ gRPC, Events
                 ↓
┌─────────────────────────────────────────────────┐
│      INTELLIGENCE & COORDINATION LAYER           │
│  (Orchestrator, AI Services, Threat Analysis)   │
└────────────────┬────────────────────────────────┘
                 │ Kafka Events
                 ↓
┌─────────────────────────────────────────────────┐
│         SENSOR FUSION & TRACKING LAYER           │
│  (Fusion Engine, Track Management, Clustering)  │
└────────────────┬────────────────────────────────┘
                 │ gRPC, Streams
                 ↓
┌─────────────────────────────────────────────────┐
│          SENSOR & DEFENSE LAYER                  │
│  (Radars, RF, EO/IR, Acoustic, Interceptors)    │
└─────────────────────────────────────────────────┘
```

### Layer Responsibilities

#### 1. Sensor & Defense Layer
**Responsibility**: Data generation and command execution

**Components**:
- Radar systems (various modes)
- RF detectors & SIGINT receivers
- EO/IR cameras
- Acoustic arrays
- LiDAR systems
- ADS-B decoders
- Interceptor drones
- Non-kinetic defense systems

**Interface**: gRPC for ingestion, Kafka topics for streaming

---

#### 2. Sensor Fusion & Tracking Layer
**Responsibility**: Multi-source data fusion, real-time tracking, swarm clustering

**Components**:
- **Fusion Engine** (Rust):
  - Multi-sensor correlation
  - Kalman filtering for track estimation
  - Data association (Hungarian algorithm)
  - Swarm detection (DBSCAN clustering)
  - Anomaly detection
  
**Data Model**:
```protobuf
message Track {
  string id = 1;                    // Unique track ID
  double latitude = 2;               // WGS84
  double longitude = 3;
  double altitude = 4;               // meters MSL
  google.protobuf.Timestamp timestamp = 5;
  
  message Velocity {
    double north_mps = 1;            // m/s, body frame
    double east_mps = 2;
    double down_mps = 3;
  }
  Velocity velocity = 6;
  
  double confidence = 7;             // 0.0-1.0
  repeated string sensor_ids = 8;    // Sources
  string swarm_id = 9;               // If part of swarm
  ThreatLevel threat_level = 10;
}

message Swarm {
  string id = 1;
  repeated string track_ids = 2;
  int32 drone_count = 3;
  double centroid_lat = 4;
  double centroid_lon = 5;
  double centroid_alt = 6;
  double formation_cohesion = 7;     // 0.0-1.0
  SwarmBehavior behavior = 8;
}
```

**Output**: Kafka topics:
- `tracks.new` - New track detected
- `tracks.updated` - Track state update
- `tracks.dropped` - Track lost
- `swarms.detected` - Swarm cluster detected
- `threats.anomaly` - Anomalous behavior

---

#### 3. Intelligence & Coordination Layer
**Responsibility**: Threat classification, prediction, defense coordination

**Components**:

##### AI Services (Python)
- **Drone Classification**: CNN + ensemble models
- **Behavior Analysis**: Graph neural networks for formation analysis
- **Intent Prediction**: LSTM-based attack vector prediction
- **Swarm Prediction**: Trajectory and formation evolution
- **Threat Scoring**: Multi-criteria threat assessment

##### Defense Orchestrator (Go)
- **Threat Prioritization**: Multi-criteria scoring
- **Interceptor Assignment**: Optimal bipartite matching
- **Engagement Simulation**: Monte Carlo ballistics
- **Distributed Coordination**: RAFT consensus for multi-site
- **ROE Enforcement**: Rules of Engagement validation
- **Human Approval Workflows**: Autonomous with escalation gates

**Data Model**:
```protobuf
message Threat {
  string id = 1;
  repeated string track_ids = 2;
  DroneType drone_type = 3;
  double classification_confidence = 4;
  AttackVector predicted_vector = 5;
  double attack_probability = 6;
  google.protobuf.Duration time_to_impact = 7;
  int32 priority_score = 8;
  ThreatStatus status = 9;
}

message EngagementPlan {
  string id = 1;
  string threat_id = 2;
  repeated InterceptorAssignment assignments = 3;
  double success_probability = 4;
  double collateral_damage_estimate = 5;
  int32 cost = 6;                    // Resources required
  EngagementStatus status = 7;
  string human_approval_required = 8; // "" = auto, field name = human role
}

message InterceptorAssignment {
  string interceptor_id = 1;
  double intercept_probability = 2;
  int32 engagements_queued = 3;
  int32 cooldown_remaining_ms = 4;
}
```

---

#### 4. API Gateway & Control Plane
**Responsibility**: Service exposition, authentication, rate limiting, real-time subscriptions

**Components**:
- **GraphQL Resolver** (Go): Query/Mutation/Subscription handlers
- **gRPC Gateway**: Service reflection and cross-language RPC
- **WebSocket Handler**: Real-time subscriptions
- **Authentication**: JWT, mTLS, hardware tokens
- **Authorization**: Role-based access control (RBAC)
- **Rate Limiting**: Per-operator, per-role quotas
- **Circuit Breakers**: Cascading failure prevention

**GraphQL Schema** (partial):
```graphql
type Query {
  tracks(limit: Int!, offset: Int): [Track!]!
  track(id: ID!): Track
  threats(severity: ThreatSeverity): [Threat!]!
  threat(id: ID!): Threat
  swarms: [Swarm!]!
  engagements(status: EngagementStatus): [EngagementPlan!]!
  status: SystemStatus!
}

type Mutation {
  assignInterceptor(threatId: ID!, interceptorId: ID!): EngagementPlan!
  engageThreat(threatId: ID!): EngagementPlan!
  abortEngagement(engagementId: ID!): Boolean!
  updateROE(rules: ROEInput!): ROE!
}

type Subscription {
  trackUpdated: Track!
  threatDetected: Threat!
  engagementStatusChanged: EngagementPlan!
}
```

---

#### 5. Command Center & Control
**Responsibility**: Operator interface, real-time visualization, threat assessment

**Components**:
- **3D Airspace Map** (Cesium.js):
  - Interactive globe with tactical overlays
  - Track vectors and prediction cones
  - Threat heatmaps
  - Interceptor positions and engagement ranges
  
- **Threat Panel**:
  - Ranked threat list
  - Confidence scores
  - Predicted impact zones
  - Attack vector analysis
  
- **Defense Controls**:
  - Interceptor assignment UI
  - Manual engagement triggers
  - Defense layer activation
  - Real-time status updates
  
- **Operator Collaboration**:
  - Multi-operator simultaneous control
  - Secure messaging
  - Incident annotations
  - Battle playback & forensics

---

## Data Flow

### Real-Time Threat Detection & Engagement Flow

```
1. SENSOR STREAMS
   Radar, RF, EO/IR, Acoustic → gRPC to Fusion Engine
   
2. FUSION ENGINE (Rust)
   Input: Multi-source sensor streams
   Processing:
   - Timestamp synchronization
   - Data association (link measurements to tracks)
   - Kalman filtering (state estimation)
   - Swarm clustering (DBSCAN)
   Output: Track updates → Kafka (tracks.*)
   
3. THREAT ANALYSIS (Python)
   Input: Fusion Engine tracks → Kafka
   Processing:
   - Drone classification (CNN inference)
   - Behavior anomaly detection
   - Attack vector prediction (LSTM)
   - Threat scoring
   Output: Threat objects → PostgreSQL, Kafka (threats.*)
   
4. DEFENSE ORCHESTRATION (Go)
   Input: Threats → Kafka, PostgreSQL
   Processing:
   - Threat prioritization
   - Interceptor assignment optimization
   - Engagement simulation
   - Approval workflow
   Output: Engagement plans → Kafka (engagements.*), PostgreSQL
   
5. DEFENSE EXECUTION
   Input: Engagement plans
   Processing:
   - Fire interceptor, activate jamming, etc.
   - Monitor engagement success
   Output: Engagement results → PostgreSQL audit log
   
6. COMMAND CENTER
   Input: All of the above via GraphQL subscriptions
   Display: Real-time airspace map, threats, engagements
   Operator: Can override, reassign, abort
```

### Latency Budget

```
Total Target: <100ms from detection to engagement recommendation

Sensor → Fusion:     15ms
Fusion → AI:         20ms
AI Inference:        20ms
Orchestration:       20ms
API Gateway:         10ms
Network round-trip:  15ms
─────────────────────────
Total:              100ms
```

---

## Service Architecture

### Sensor Fusion Engine

**Language**: Rust
**Runtime**: Tokio async
**Key Libraries**: ndarray, tonic, sqlx, anyhow

**Modules**:
```
fusion-engine/
├── src/
│   ├── main.rs              # Entry point, gRPC server
│   ├── fusion/
│   │   ├── kalman.rs        # Kalman filter impl
│   │   ├── data_assoc.rs    # Hungarian algorithm
│   │   └── clustering.rs    # DBSCAN for swarms
│   ├── sensor/
│   │   ├── ingest.rs        # Sensor stream handling
│   │   ├── sync.rs          # Timestamp sync
│   │   └── types.rs         # Sensor data types
│   ├── track/
│   │   ├── manager.rs       # Track lifecycle
│   │   ├── confidence.rs    # Track confidence scoring
│   │   └── state.rs         # Track state machine
│   ├── db/
│   │   └── postgres.rs      # Database ops
│   ├── kafka/
│   │   └── producer.rs      # Event publishing
│   └── config.rs            # Config loading
├── Cargo.toml
└── tests/
```

**Interface**: gRPC + Kafka
- gRPC: Sensor ingestion, health checks
- Kafka: Track streams, swarm events

---

### AI & ML Services

**Language**: Python
**Framework**: FastAPI
**ML Libraries**: PyTorch, TensorFlow, scikit-learn

**Modules**:
```
ai-services/
├── classification/
│   ├── drone_classifier.py  # CNN ensemble
│   ├── intent_detector.py   # Attack intent
│   └── models.py            # Model loading, serving
├── prediction/
│   ├── trajectory.py        # LSTM trajectory prediction
│   ├── formation.py         # GNN formation analysis
│   └── simulator.py         # Monte Carlo prediction
├── anomaly/
│   ├── detector.py          # Isolation forest
│   └── scorer.py            # Threat confidence
├── api/
│   └── main.py             # FastAPI endpoints
├── data/
│   └── kafka_consumer.py    # Threat stream consumer
└── requirements.txt
```

**Interface**: gRPC + Kafka
- gRPC: Real-time inference
- Kafka: Threat stream updates

---

### Defense Orchestrator

**Language**: Go
**Concurrency**: goroutines, channels
**Key Libraries**: etcd client, gRPC, kafka-go

**Modules**:
```
orchestrator/
├── main.go                   # Entry point
├── threat/
│   ├── prioritization.go     # Multi-criteria scoring
│   └── analyzer.go           # Threat analysis
├── engagement/
│   ├── assignment.go         # Bipartite matching
│   ├── simulator.go          # Ballistics, probability
│   └── plan.go               # Engagement planning
├── coordination/
│   ├── consensus.go          # RAFT for multi-site
│   ├── rollback.go           # Failure recovery
│   └── state.go              # Engagement state machine
├── roe/
│   ├── engine.go             # ROE enforcement
│   ├── rules.go              # Rule definitions
│   └── approval.go           # Approval workflows
├── api/
│   └── grpc_server.go        # gRPC handlers
└── go.mod
```

**Interface**: gRPC + Kafka + etcd
- gRPC: Threat queries, engagement commands
- Kafka: Threat stream, engagement events
- etcd: Distributed state, leader election

---

### API Gateway

**Language**: Go
**Framework**: Chi router
**Key Libraries**: graphql-go, gorilla/websocket

**Modules**:
```
api-gateway/
├── main.go
├── graphql/
│   ├── resolver.go           # Query/Mutation/Subscription
│   ├── schema.go             # Schema definitions
│   └── directives.go         # Custom directives (auth)
├── websocket/
│   └── handler.go            # WebSocket upgrades
├── middleware/
│   ├── auth.go               # JWT validation
│   ├── authz.go              # RBAC
│   ├── ratelimit.go          # Rate limiting
│   └── logging.go            # Request logging
├── service/
│   ├── tracks.go             # Track queries
│   ├── threats.go            # Threat queries
│   └── engagements.go        # Engagement mutations
└── go.mod
```

**Interface**: HTTP/1.1, HTTP/2 (gRPC), WebSocket
- GraphQL endpoint: `/graphql`
- gRPC endpoint: `:50051`
- WebSocket subscriptions: `/graphql/subscriptions`

---

### Command Center UI

**Framework**: Next.js 15
**Language**: TypeScript + React
**Key Libraries**: Cesium.js, Tailwind, GraphQL-request

**Structure**:
```
command-center/
├── src/
│   ├── pages/
│   │   ├── index.tsx         # Main command center
│   │   ├── analyst.tsx       # Analyst console
│   │   ├── operations.tsx    # Operations dash
│   │   └── api/
│   │       └── graphql.ts    # GraphQL endpoint
│   ├── components/
│   │   ├── Map/
│   │   │   ├── AirspaceMap.tsx
│   │   │   ├── TrackOverlay.tsx
│   │   │   ├── ThreatCone.tsx
│   │   │   └── InterceptorRange.tsx
│   │   ├── Panels/
│   │   │   ├── ThreatPanel.tsx
│   │   │   ├── DefenseControl.tsx
│   │   │   └── EngagementStatus.tsx
│   │   └── UI/
│   │       ├── Button.tsx
│   │       ├── Card.tsx
│   │       └── Gauge.tsx
│   ├── hooks/
│   │   ├── useAirspace.ts    # GraphQL subscription
│   │   ├── useThreat.ts
│   │   └── useEngagement.ts
│   ├── styles/
│   │   └── globals.css       # Tailwind + custom theme
│   └── lib/
│       ├── graphql/
│       │   ├── client.ts
│       │   └── queries.ts
│       └── utils/
│           ├── geo.ts        # Geospatial utilities
│           └── tactical.ts   # Tactical calculations
├── public/
│   └── assets/               # Maps, icons, etc.
└── package.json
```

---

## Technology Decisions

### Why Rust for Sensor Fusion?
- **Low-latency**: No garbage collector, predictable performance
- **Memory-safe**: Prevents buffer overflows, use-after-free
- **Concurrent**: Tokio async runtime, high throughput
- **Numerical**: ndarray for matrix ops, approaching C performance
- **Real-time guarantees**: Suitable for critical path processing

### Why Python for ML?
- **Industry standard**: PyTorch, TensorFlow, scikit-learn ecosystems
- **Rapid iteration**: Dynamic typing, Jupyter for exploration
- **GPU support**: CUDA, cuDNN, TensorRT
- **Community**: 1000+ pre-trained models, active research
- **Tradeoff**: Slightly higher latency acceptable for inference

### Why Go for Orchestration?
- **Concurrency**: Goroutines >> threads, excellent scalability
- **Distributed systems**: Native etcd, gRPC support
- **Simplicity**: Fast compile, minimal runtime
- **Reliability**: RAFT consensus, robust error handling
- **Operations**: Single binary, easy deployment

### Why Next.js for UI?
- **Server-side rendering**: Fast initial loads
- **Real-time support**: WebSocket integration, subscriptions
- **Developer experience**: Hot reload, TypeScript support
- **Performance**: Automatic code splitting, optimization
- **Ecosystem**: Massive library ecosystem

### Why PostgreSQL?
- **ACID guarantees**: Transactional consistency for audit logs
- **TimescaleDB extension**: Native time-series data (tracks, events)
- **JSON support**: Flexible schema for sensor data
- **Proven**: Used in critical infrastructure

### Why Kafka for Events?
- **Distributed**: Multi-broker, fault-tolerant
- **High-throughput**: 1M+ messages/sec per broker
- **Durability**: Log-based persistence
- **Replayability**: Full event history for analysis
- **Ecosystem**: Kafka Streams, Spark, etc.

---

## Scalability & Performance

### Horizontal Scalability

**Fusion Engine**:
- Shard by geographic region (lat/lon quadtree)
- Each shard processes tracks independently
- Shared state in PostgreSQL for cross-region queries

**AI Services**:
- Ray cluster for distributed inference
- Model serving with auto-scaling
- Batch processing for offline analysis

**Orchestrator**:
- Multi-leader RAFT for distributed decisions
- Event sourcing in Kafka for audit trail
- Shared state in etcd for leader election

**API Gateway**:
- Stateless design, horizontal scaling
- Load balancing via reverse proxy (nginx/traefik)

**Command Center**:
- Static assets via CDN
- WebSocket connections scaled via load balancer

### Performance Targets

| Metric | Target | Method |
|--------|--------|--------|
| Sensor → Track | < 50ms | Direct fusion pipeline |
| Track → Threat | < 30ms | Kafka streaming |
| Threat → Engagement | < 20ms | Orchestrator decision |
| API Latency | < 20ms | Direct DB queries |
| UI Update | < 50ms | WebSocket push |
| **Total (Sensor → UI)** | **< 100ms** | Full pipeline |

### Capacity

| Component | Capacity | Notes |
|-----------|----------|-------|
| Simultaneous Tracks | 1000+ | Depends on fusion engine CPU |
| Drone Swarms | 100+ | Clustering overhead |
| Concurrent Operators | 100+ | API gateway scaling |
| Events/second | 100K+ | Kafka + Fusion throughput |
| Threat Decisions/sec | 1K+ | Orchestrator throughput |

---

## Security Architecture

### Zero-Trust Network

All inter-service communication is authenticated and encrypted:

```
┌──────────────────┐  mTLS  ┌──────────────────┐
│  Fusion Engine   │◄─────►│  API Gateway     │
│  (Rust)          │  gRPC │  (Go)            │
└──────────────────┘        └──────────────────┘
         │
         │ Kafka over TLS
         ↓
┌──────────────────────────────────────┐
│  Message Broker (Kafka)              │
│  - Encrypted at rest                 │
│  - TLS in transit                    │
└──────────────────────────────────────┘
```

### Authentication Layers

1. **Service-to-Service**: mTLS with cert rotation
2. **Operator-to-Gateway**: JWT + optional hardware token
3. **Operator-to-UI**: Session cookies over HTTPS
4. **Operator-to-Defense**: RBAC with approval gates

### Encryption

- **At Rest**: AES-256-GCM for sensitive data
- **In Transit**: TLS 1.3 for all connections
- **Keys**: Managed by hardware security module (HSM) where available

### Audit & Logging

- **Immutable Audit Log**: Event sourcing in PostgreSQL
- **Engagement Log**: Every engagement decision with who/why/when
- **Alert Log**: All threat detections with classification confidence
- **Configuration Log**: All ROE and policy changes

### Supply Chain Security

- **Signed Artifacts**: Docker images signed with Cosign
- **SBOM**: Software Bill of Materials for every release
- **Dependency Scanning**: Continuous scanning for CVEs
- **Code Scanning**: SAST, DAST, supply-chain verification

---

## Deployment Architectures

### Architecture 1: Cloud SaaS

```
┌─────────────────────────────────────────┐
│         Cloud Provider (AWS/GCP/Azure)  │
├─────────────────────────────────────────┤
│  Kubernetes Cluster (Multi-AZ)          │
│  ┌────────────────────────────────────┐ │
│  │  API Gateway, Services             │ │
│  │  PostgreSQL (RDS), Redis           │ │
│  │  Kafka (MSK), TimescaleDB          │ │
│  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────┐ │
│  │  Command Center UI (CDN + LB)      │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
         │ TLS + VPN
         ↓
┌─────────────────────────────────────────┐
│     Military Base / Data Center         │
│  ┌────────────────────────────────────┐ │
│  │  Edge Sensor Ingestion (K3s)      │ │
│  │  Local Fusion Engine              │ │
│  │  ↓ Encrypted Stream to Cloud      │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### Architecture 2: Air-Gapped Military

```
┌──────────────────────────────────────────┐
│    Isolated Network (No External Access) │
├──────────────────────────────────────────┤
│  Kubernetes Cluster (On-Prem)            │
│  ┌────────────────────────────────────┐  │
│  │  All Services (Fusion, AI, Orch)   │  │
│  │  PostgreSQL, Redis, Kafka          │  │
│  │  API Gateway + Command Center UI   │  │
│  └────────────────────────────────────┘  │
│  ┌────────────────────────────────────┐  │
│  │  Sensor Ingestion (Radar, RF, etc) │  │
│  │  Defense Mechanism Integration     │  │
│  └────────────────────────────────────┘  │
│  ┌────────────────────────────────────┐  │
│  │  Manual Update Delivery            │  │
│  │  (Secure media, signed artifacts)  │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

### Architecture 3: Nationwide Federation

```
┌──────────────────────────┐
│   Command Center (Cloud) │
│   (Strategy, Learning)   │
└────────────┬─────────────┘
             │ Threat Intelligence,
             │ Model Updates
             ↓
┌──────────────────────────┐   ┌──────────────────┐
│  Base 1 (K3s Edge)       │──│  Base 2 (K3s)    │
│  Local Fusion + Defense  │  │  Local Fusion +  │
└──────────────────────────┘  │  Defense         │
                              └──────────────────┘
    │                                  │
    │ (Consensus on Shared Threats)    │
    └──────────────────┬───────────────┘
                       │
              ┌────────▼─────────┐
              │  Base 3 (K3s)    │
              │  Local Fusion +  │
              │  Defense         │
              └──────────────────┘
```

---

## Integration Points

### External Sensor Integration

**Protocol**: gRPC streaming
```protobuf
service SensorProvider {
  rpc StreamSensorData(SensorConfig) returns (stream SensorMeasurement);
}

message SensorMeasurement {
  string sensor_id = 1;
  google.protobuf.Timestamp timestamp = 2;
  oneof data {
    RadarReturn radar = 3;
    RFSignal rf = 4;
    ImageData image = 5;
    AcousticData acoustic = 6;
  }
}
```

### Defense Mechanism Integration

**Protocol**: gRPC command/control
```protobuf
service DefenseActuator {
  rpc EngageThreat(EngagementCommand) returns (EngagementResult);
  rpc GetStatus(Empty) returns (ActuatorStatus);
  rpc AbortEngagement(AbortRequest) returns (AbortResult);
}
```

### ADOS Mission Control Integration

**Protocol**: gRPC + Kafka

1. **Ingest ADOS Mission Streams**:
   - Subscribe to ADOS mission events
   - Map ADOS coordinates to local coordinate system
   - Feed mission objectives as constraints to orchestrator

2. **Export Swarm Intelligence**:
   - Publish detected swarms to ADOS
   - Share threat classifications
   - Feed engagement results back to ADOS

3. **Coordinate Defense Actions**:
   - ADOS can request specific defense actions
   - Orchestrator validates against ROE
   - Returns engagement probability estimates

---

## Example: Full Threat Engagement Flow

```
1. RADAR DETECTS SWARM
   Radar → SensorProvider gRPC → Fusion Engine
   
2. FUSION ENGINE PROCESSES
   Kalman filter + DBSCAN clustering
   Output: 12 tracks, 1 swarm cluster
   Kafka: tracks.new × 12, swarms.detected × 1
   
3. AI SERVICES CLASSIFY
   Kafka listener consumes tracks
   CNN classifier: [12× DJI M300, 2× Fixed-wing loitering]
   Intent detector: "Coordinated attack on power plant"
   Trajectory predictor: "ETA 4 minutes to facility"
   
4. THREAT ASSESSMENT
   Orchestrator consumes AI outputs
   Multi-criteria scoring:
   - Count: 12 drones = high
   - Speed: 20 m/s = medium urgency
   - Vector: Direct approach = high threat
   - Intent: Attack formation = highest
   
5. ENGAGEMENT PLANNING
   Orchestrator:
   - Assign 3 interceptors to highest-priority drones
   - Activate RF jamming on all
   - Query ROE: "Civilian area within 500m?"
   - Result: Requires human approval
   
6. APPROVAL WORKFLOW
   API Gateway: GraphQL Mutation engageThreat(threatId)
   UI: Operator sees 4-minute warning
   Shows engagement plan with 87% success probability
   Operator: "APPROVED"
   
7. ENGAGEMENT EXECUTION
   Orchestrator commands:
   → Interceptor drone 1: "Engage target A-1"
   → Interceptor drone 2: "Engage target A-2"
   → Jamming unit: "Activate RF disruption"
   
8. RESULTS TRACKING
   Kafka: engagements.status_updated
   → Target A-1: Hit, 23 seconds
   → Target A-2: Miss, lost signal
   → Jamming: 3 drones disabled
   
9. UI UPDATES
   WebSocket subscription triggered
   Map updates: target positions, engagement cones
   Threat panel: Remaining threat count: 7
   
10. INCIDENT LOG
    Event sourced: All decisions, approvals, outcomes
    Available for: After-action review, AI training
```

---

## Future Enhancements

- **Federated Learning**: Model updates across bases without central cloud
- **Quantum-Safe Crypto**: Post-quantum encryption algorithms
- **Autonomous Swarm Defense**: Deploy defensive drones coordinated by orchestrator
- **Advanced EW**: Electronic warfare integration (spoofing, jamming adaptation)
- **Autonomous Base Protection**: No-operator fallback mode
- **AI Deception**: Generate synthetic radar/RF signatures to confuse attackers

