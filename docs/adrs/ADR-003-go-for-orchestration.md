# ADR-003: Use Go for Defense Orchestration

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The defense orchestrator must:
- Coordinate multiple defense mechanisms across sites
- Maintain consistency across distributed nodes
- Make sub-100ms threat prioritization decisions
- Handle 1000+ threat decisions per second
- Support multi-site consensus via RAFT

**Candidates**:
1. **Go**: Goroutines, mature distributed libraries
2. **Rust**: Safety, but heavyweight async
3. **Java**: Ecosystem, but heavyweight GC
4. **Python**: Simplicity, but poor concurrency

## Decision

Use **Go** for the defense orchestrator and API gateway.

**Stack**:
- **Concurrency**: Goroutines + channels
- **Consensus**: etcd RAFT client
- **gRPC**: official Go implementation
- **Web**: GraphQL (graphql-go) + WebSocket
- **Build**: Go 1.21+

## Rationale

### 1. Goroutine Efficiency
- 100K+ goroutines in single process
- 2KB per goroutine (vs 1-2MB per thread)
- Instant context switching
- Perfect for coordinating 1000+ threats simultaneously

### 2. Distributed Systems
- etcd client for leader election
- RAFT consensus built-in
- Proven in Kubernetes, CoreDNS, etc.
- Multi-site coordination out of the box

### 3. Concurrency Model
- Channels enforce safe data sharing
- No manual locking needed
- "Share memory by communicating, not by sharing memory"
- Eliminates race conditions by design

### 4. Performance
- ~50ns goroutine wake-up latency
- Fast context switching (< 1µs)
- Minimal overhead per decision
- Can process 10K+ threat prioritizations/sec

### 5. Simplicity
- Single language for both orchestrator and API gateway
- Compiled binary, no runtime dependencies
- Fast compilation (seconds, not minutes)
- Excellent error handling via interfaces

## Example: Multi-Site Threat Consensus

```go
package main

import (
    "context"
    clientv3 "go.etcd.io/etcd/client/v3"
)

type ThreatOrchestrator struct {
    client *clientv3.Client
    threats chan Threat
}

func (o *ThreatOrchestrator) PrioritizeThreats(ctx context.Context, threats []Threat) error {
    // Use etcd for distributed consensus
    lease := o.client.Lease.Grant(ctx, 5) // 5 second TTL
    
    for _, threat := range threats {
        score := o.scoreThreat(threat)
        
        // Acquire distributed lock via etcd
        mutex := concurrency.NewMutex(o.session, "/threats/lock")
        if err := mutex.Lock(ctx); err != nil {
            return err
        }
        
        // Check if another site already handling this threat
        resp, _ := o.client.Get(ctx, "/threats/"+threat.ID)
        if resp.Count == 0 {
            // Assign locally
            o.client.Put(ctx, "/threats/"+threat.ID, 
                json.Marshal(score), clientv3.WithLease(lease.ID))
        }
        
        mutex.Unlock(ctx)
    }
    return nil
}

func (o *ThreatOrchestrator) scoreThreat(t Threat) int {
    return t.DroneCount*10 + int(t.Speed)*5 + t.ThreatLevel*20
}
```

## Tradeoffs

### Advantages
- Native goroutines beat thread-per-request model
- Distributed systems libraries mature
- Single binary deployment
- Excellent observability (pprof profiling)

### Disadvantages
- Memory model simpler but less flexible
- Fewer pre-built libraries vs Python
- GC pauses (1-10ms), though less than Java
- Learning curve for concurrent patterns

## Mitigation

### GC Tuning
```go
// Reduce GC pause time
debug.SetGCPercent(200)  // Default 100, higher = less frequent
```

### Library Selection
- etcd: distributed consensus ✓
- gRPC: RPC ✓
- Kafka: kafka-go ✓
- PostgreSQL: sqlc (type-safe queries) ✓

## Consequences

- Orchestrator can handle massive concurrency
- Multi-site coordination reliable and proven
- Single-service deployment (no separate microservices needed)
- Team learns production-grade distributed systems

## Related Decisions

- [ADR-007: Kafka for Event Sourcing](ADR-007-kafka-event-sourcing.md)
- [ADR-008: Zero-Trust Service Mesh](ADR-008-zero-trust.md)

## References

- [Effective Go](https://golang.org/doc/effective_go)
- [etcd API Documentation](https://github.com/etcd-io/etcd/tree/main/client/v3)
- [Go Concurrency Patterns](https://go.dev/blog/pipelines)
