# ADR-009: GraphQL + gRPC for API Gateway

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The API gateway must serve:
- **Web clients** (Command Center, analyst console)
- **Mobile clients** (future: tablet operations)
- **Third-party integrations** (ADOS Mission Control, defense systems)
- **Real-time subscriptions** (track updates, threat alerts)
- **Metrics & analytics** (observability, after-action review)

**Candidates**:
1. **REST**: Simple, but verbose, hard for real-time
2. **GraphQL + gRPC**: Flexible + efficient, slightly more complex
3. **WebSocket only**: Real-time but not browsable

## Decision

Use **GraphQL for clients** (web, mobile) and **gRPC for services** (internal APIs).

**Gateway Architecture**:
```
Web/Mobile Client
  ↓ HTTP/1.1 + WebSocket
GraphQL Gateway (Go)
  ↓ gRPC
Fusion Engine, Orchestrator, AI Services
```

## Rationale

### 1. GraphQL for Clients
- **Single query language**: Clients ask for exact fields needed
- **Real-time via subscriptions**: WebSocket-based updates
- **Introspection**: Clients can discover schema
- **Large ecosystem**: Apollo, Relay, etc.
- **Type safety**: Schema-driven code generation

### 2. gRPC for Services
- **Efficient**: Binary protobuf, smaller payloads
- **HTTP/2**: Multiplexing, server push
- **Streaming**: Perfect for sensor data
- **Type-safe**: Protobuf definitions
- **Language-agnostic**: Works across Rust, Go, Python

## Example: GraphQL Schema

```graphql
type Query {
  # Tracks
  tracks(
    limit: Int! = 50
    offset: Int = 0
    swarmId: ID
  ): [Track!]!
  
  track(id: ID!): Track
  
  # Threats
  threats(
    severity: ThreatSeverity
    status: ThreatStatus
  ): [Threat!]!
  
  threat(id: ID!): Threat
  
  # Engagements
  engagements(
    status: EngagementStatus
  ): [EngagementPlan!]!
  
  # System status
  status: SystemStatus!
}

type Mutation {
  # Defense actions
  engageThreat(threatId: ID!): EngagementPlan!
  
  assignInterceptor(
    threatId: ID!
    interceptorId: ID!
  ): EngagementPlan!
  
  abortEngagement(engagementId: ID!): Boolean!
  
  # Configuration
  updateROE(rules: ROEInput!): ROE!
  updateDefenseLayerState(input: DefenseStateInput!): DefenseState!
}

type Subscription {
  # Real-time updates
  trackUpdated: Track!
  trackDropped(trackId: ID!): String!
  
  threatDetected: Threat!
  threatResolved(threatId: ID!): String!
  
  engagementStatusChanged: EngagementPlan!
  
  alertTriggered: SystemAlert!
}

# Data types
type Track {
  id: ID!
  timestamp: DateTime!
  latitude: Float!
  longitude: Float!
  altitude: Float!
  velocity: Velocity!
  confidence: Float!
  swarmId: ID
  droneType: DroneType
  sensors: [String!]!
  threatLevel: ThreatLevel!
}

type Threat {
  id: ID!
  trackIds: [ID!]!
  droneType: DroneType
  classificationConfidence: Float!
  predictedVector: Geometry!
  attackProbability: Float!
  timeToImpact: Duration!
  priorityScore: Int!
  status: ThreatStatus!
}

type EngagementPlan {
  id: ID!
  threatId: ID!
  assignments: [InterceptorAssignment!]!
  successProbability: Float!
  collateralDamageEstimate: Float!
  cost: Int!
  status: EngagementStatus!
  approvedBy: String
  approvedAt: DateTime
}

enum ThreatSeverity { LOW MEDIUM HIGH CRITICAL }
enum ThreatStatus { DETECTED TRACKED TARGETED ENGAGED NEUTRALIZED }
enum DroneType { QUADCOPTER FIXED_WING LOITERING_MUNITION SWARM }
enum EngagementStatus { PROPOSED APPROVED EXECUTING COMPLETED FAILED ABORTED }
```

## Example: GraphQL Resolver (Go)

```go
package graphql

import "context"

type Resolver struct {
    fusionClient fusionpb.FusionClient
    orchestrator orchestratorpb.OrchestratorClient
}

func (r *Resolver) Query() QueryResolver {
    return &queryResolver{r}
}

type queryResolver struct{ *Resolver }

func (r *queryResolver) Tracks(ctx context.Context, limit int, offset int) ([]*Track, error) {
    // Call fusion service via gRPC
    resp, err := r.fusionClient.ListTracks(ctx, &fusionpb.ListTracksRequest{
        Limit:  int32(limit),
        Offset: int32(offset),
    })
    if err != nil {
        return nil, err
    }
    
    // Convert protobuf to GraphQL types
    var tracks []*Track
    for _, pbTrack := range resp.Tracks {
        tracks = append(tracks, &Track{
            ID:         pbTrack.Id,
            Latitude:   pbTrack.Latitude,
            Longitude:  pbTrack.Longitude,
            // ... map other fields
        })
    }
    return tracks, nil
}

func (r *Resolver) Mutation() MutationResolver {
    return &mutationResolver{r}
}

type mutationResolver struct{ *Resolver }

func (r *mutationResolver) EngageThreat(ctx context.Context, threatID string) (*EngagementPlan, error) {
    // Call orchestrator via gRPC
    resp, err := r.orchestrator.ProposalEngagement(ctx, &orchestratorpb.ProposalRequest{
        ThreatId: threatID,
    })
    // ... error handling and type conversion
    return &EngagementPlan{
        ID:     resp.Id,
        Status: resp.Status.String(),
        // ...
    }, nil
}
```

## Real-Time Subscriptions

```go
// WebSocket subscription handler
func (gw *GraphQLGateway) handleSubscription(w http.ResponseWriter, r *http.Request) {
    // Upgrade HTTP to WebSocket
    ws, err := upgrader.Upgrade(w, r, nil)
    if err != nil {
        return
    }
    defer ws.Close()
    
    // Subscribe to Kafka topics
    reader := kafka.NewReader(kafka.ReaderConfig{
        Brokers: []string{"kafka:9092"},
        Topic:   "tracks.updated",
        GroupID: "graphql-gateway",
    })
    
    // Forward Kafka messages to WebSocket
    for {
        msg, _ := reader.ReadMessage(r.Context())
        
        // Parse Kafka message
        var track Track
        json.Unmarshal(msg.Value, &track)
        
        // Send to WebSocket client
        ws.WriteJSON(map[string]interface{}{
            "type": "data",
            "data": map[string]interface{}{
                "trackUpdated": track,
            },
        })
    }
}
```

## Performance Characteristics

| Operation | Protocol | Latency | Notes |
|-----------|----------|---------|-------|
| List tracks (50) | GraphQL | 20ms | Efficient query selection |
| Single track | gRPC | 2ms | Direct service call |
| Track subscription | WebSocket | 50ms | Kafka → WS push |
| Mutation (engage) | GraphQL | 100ms | Orchestrator decision |

## Tradeoffs

### GraphQL Advantages
- Clients request exact fields (no over-fetching)
- Single endpoint (no REST versioning)
- Real-time via subscriptions
- Schema documentation
- Large ecosystem

### GraphQL Disadvantages
- Query complexity (need rate limiting)
- Caching harder than REST
- Learning curve

### gRPC Advantages
- Efficient binary protocol
- HTTP/2 multiplexing
- Bidirectional streaming
- Language-agnostic

### gRPC Disadvantages
- Not browsable (need tools)
- Less REST integration
- Learning curve

## Mitigation

### Query Complexity
```go
// Rate limiting on GraphQL depth
const maxQueryDepth = 5

func validateQuery(query string) error {
    doc, _ := parser.Parse(query)
    depth := calculateQueryDepth(doc)
    if depth > maxQueryDepth {
        return errors.New("query too complex")
    }
    return nil
}
```

### Caching
- Cache query results in Redis
- Invalidate on subscription updates
- Use etag headers for freshness

## Consequences

- Efficient API for web clients
- Real-time capabilities via subscriptions
- Type-safe internal services (gRPC)
- Easy to add new queries without breaking clients
- Flexible for third-party integrations

## Related Decisions

- [ADR-006: Next.js UI](ADR-006-nextjs-command-center.md)
- [ADR-009: API Gateway](ADR-009-graphql-api-gateway.md)

## References

- [GraphQL Official](https://graphql.org/)
- [gRPC Official](https://grpc.io/)
- [Apollo Server Documentation](https://www.apollographql.com/docs/apollo-server/)
