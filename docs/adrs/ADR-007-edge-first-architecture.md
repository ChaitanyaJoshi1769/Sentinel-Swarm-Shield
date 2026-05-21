# ADR-007: Edge-First Distributed Architecture

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The system must operate in environments where:
- Cloud connectivity is unreliable (military, remote locations)
- Latency to cloud is unacceptable (<100ms required)
- Some sites operate GPS-denied, air-gapped
- Multiple independent installations need coordination
- Sensor data is too voluminous to send to cloud

## Decision

Adopt **edge-first architecture** where:
- Each site runs complete defense stack locally (Kubernetes + K3s)
- Cloud acts as coordinator and learner, not critical path
- No dependency on cloud for real-time defense
- Eventually-consistent synchronization via Kafka Connect

## Architecture

```
┌─────────────────────────────┐
│     Cloud Coordinator       │
│  (Model updates, Learning)  │
└────────────┬────────────────┘
             │ Kafka MirrorMaker
             │ (Async replication)
┌────────────▼──────────────────────────────────┐
│         Base Cluster Federation                │
├───────────────────────────────────────────────┤
│                                               │
│  ┌─────────────────┐  ┌──────────────────┐  │
│  │  Base 1 (K3s)   │  │  Base 2 (K3s)    │  │
│  │ Full local stack│  │ Full local stack │  │
│  │ - Fusion        │  │ - Fusion         │  │
│  │ - AI Services   │  │ - AI Services    │  │
│  │ - Orchestrator  │  │ - Orchestrator   │  │
│  │ - PostgreSQL    │  │ - PostgreSQL     │  │
│  │ - Kafka         │  │ - Kafka          │  │
│  │ - UI            │  │ - UI             │  │
│  └────────┬────────┘  └────────┬─────────┘  │
│           │                    │             │
│    gRPC, Kafka, PostgreSQL Replication       │
│           │                    │             │
│  ┌────────▼────────────────────▼─────────┐  │
│  │     Distributed State (Consensus)      │  │
│  │  - Shared threats (RAFT consensus)    │  │
│  │  - Engagement coordination            │  │
│  │  - Federated learning models          │  │
│  └────────────────────────────────────────┘  │
│                                               │
└───────────────────────────────────────────────┘
```

## Rationale

### 1. Reliability Under Disconnection
- No single point of failure in cloud
- Each site makes autonomous decisions
- Graceful degradation if network splits
- Historical precedent: military radio networks

### 2. Low Latency at Edge
- Sensor fusion happens locally (<100ms)
- Defense decisions made locally
- No cloud round-trip required
- Critical for sub-second engagement

### 3. Data Sovereignty
- Sensitive sensor data stays on-site
- Reduced cloud data transmission
- Military/government compliance
- Air-gap compatible

### 4. Operational Independence
- Installations can operate independently
- Fallback if HQ connection lost
- Each commander controls their airspace
- Coordinated but autonomous

## Implementation Pattern

### Local Stack (K3s)
```yaml
# K3s cluster at each base
apiVersion: v1
kind: Service
metadata:
  name: fusion-engine
spec:
  selector:
    app: fusion-engine
  ports:
    - port: 50051  # gRPC
      targetPort: 50051
  type: ClusterIP

---
apiVersion: v1
kind: Service
metadata:
  name: kafka
spec:
  selector:
    app: kafka
  ports:
    - port: 9092
      targetPort: 9092
  type: NodePort
```

### Cross-Site Replication (Kafka Connect)
```properties
# Kafka Connect configuration for multi-base federation
name=mirror-events-to-hq
connector.class=org.apache.kafka.connect.mirror.MirrorSourceConnector
source.cluster.alias=base1
target.cluster.alias=hq
source.consumer.group.id=mirrormaker-cluster

topics=threats.*,engagements.*,audit.*
groups=orchestrator
```

### Consensus Algorithm (RAFT via etcd)
```go
// Multi-site threat consensus
func (o *ThreatOrchestrator) federatedPrioritize(threats []Threat) {
    // Write to local etcd
    kv := clientv3.New(config)
    
    for _, threat := range threats {
        score := o.scoreThreat(threat)
        
        // Lease: expires if this base goes offline
        lease := kv.Lease.Grant(30)  // 30s TTL
        
        kv.Put(ctx, "/threats/"+threat.ID, 
            json.Marshal(score), 
            clientv3.WithLease(lease.ID))
    }
    
    // RAFT consensus: only winner acts
    // If multiple bases detect same threat,
    // only one will hold the lease and act
}
```

## Deployment Modes

### Mode 1: Autonomous Base (Air-Gapped)
```
┌───────────────────────────────────┐
│  Isolated K3s Cluster             │
│  - All services in containers     │
│  - Zero external dependencies     │
│  - Manual update delivery         │
│  - Operator controlled            │
└───────────────────────────────────┘
```

### Mode 2: Federated with Cloud
```
Base K3s ←→ Cloud Coordinator
(Kafka MirrorMaker)
- Real-time threat share
- Model updates pushed down
- Training data pulled up
```

### Mode 3: Hierarchical (Regiment → Company → Squad)
```
Regiment HQ
    ↑
    ├─ Company A (K3s cluster)
    ├─ Company B (K3s cluster)
    └─ Company C (K3s cluster)
```

## Tradeoffs

### Advantages
- Operates without cloud connectivity
- Sub-100ms local decisions
- Military-friendly (each command owns airspace)
- Scalable (add bases without cloud growth)
- Eventual consistency is acceptable for defense

### Disadvantages
- Distributed systems complexity
- Consensus overhead (network latency)
- Eventual consistency (temporary disagreement)
- More infrastructure to manage

## Mitigation

### Complexity
- Helm charts for easy K3s deployment
- Operator automation (SaltStack, Ansible)
- Standard procedures for network partition
- Well-documented escalation paths

### Consistency
- Use RAFT for critical decisions (threat coordination)
- Accept eventual consistency for sharing intelligence
- Immutable event log (Kafka) for auditability
- Timestamp-based conflict resolution

## Consequences

- System resilient to network failures
- Each installation truly autonomous
- Suitable for military/government deployment
- Scales to nationwide/global federation
- Learning (model updates) happens centrally, distribution happens at edge

## Related Decisions

- [ADR-004: Kafka for Event Streaming](ADR-004-kafka-event-streaming.md)
- [ADR-010: Federated Learning](ADR-010-federated-learning.md)
- [ADR-008: Zero-Trust Security](ADR-008-zero-trust.md)

## References

- [Kubernetes Edge Deployment](https://kubernetes.io/docs/concepts/architecture/nodes/)
- [K3s Lightweight Distribution](https://k3s.io/)
- [Kafka MirrorMaker](https://kafka.apache.org/documentation/#connect)
