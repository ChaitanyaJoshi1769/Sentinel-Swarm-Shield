# ADR-004: Use Apache Kafka for Event Streaming

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The system needs event streaming for:
- Sensor data distribution (100K+ events/sec)
- Track updates and threat notifications
- Engagement audit trail (immutable event log)
- Multi-site data synchronization
- Federated learning model updates
- Offline availability (replay battles for analysis)

**Candidates**:
1. **Kafka**: Distributed, durable, high-throughput
2. **RabbitMQ**: Message-oriented, but less durable
3. **NATS**: Ultra-fast, but no persistence
4. **Redis Streams**: In-memory, limited durability

## Decision

Use **Apache Kafka** for all event streaming and audit logging.

**Deployment**:
- **Cloud**: AWS MSK (Managed Streaming for Kafka)
- **Edge**: Confluent Community Kafka in containers
- **Replication**: 3 replicas minimum (production)

## Rationale

### 1. Durability & Immutability
- Log-based persistence (not in-memory)
- Events stored on disk (survives failures)
- Perfect for immutable audit logs
- Replay capability for incident analysis

### 2. Distribution & Scalability
- Broker replication for fault tolerance
- Partition sharding for parallelism
- Add brokers without downtime
- Handles 1M+ events/sec per cluster

### 3. Multi-Site Federation
- Kafka Connect for cross-cluster replication
- Geo-distributed with MirrorMaker
- Eventual consistency across sites
- Perfect for federated learning

### 4. Data Replay
- Full event history available
- Rebuild state by replaying events
- A/B test threat models (time-travel)
- Training data generation for ML models

### 5. Integration Ecosystem
- Kafka Streams for real-time processing
- Spark Streaming for batch analysis
- Hundreds of connectors (SQL, S3, etc.)
- Battle-tested in production (Netflix, Uber, etc.)

## Event Topics

```
tracks.new              → New track detected
tracks.updated          → Track state changed
tracks.dropped          → Track lost
swarms.detected         → Swarm cluster formed
swarms.dispersed        → Swarm broken up
threats.detected        → Threat identified
threats.updated         → Threat reassessment
threats.resolved        → Threat neutralized
engagements.proposed    → Engagement recommended
engagements.approved    → Human approved
engagements.executed    → Engagement started
engagements.completed   → Engagement finished
engagements.failed      → Engagement failed
roe.updated             → ROE changed
model.updated           → ML model version changed
audit.engagement        → Immutable engagement log
audit.decision          → Immutable decision log
```

## Example: Event Producer (Rust)

```rust
use rdkafka::producer::FutureProducer;
use serde_json::json;

async fn publish_track_event(producer: &FutureProducer, track: &Track) {
    let event = json!({
        "timestamp": track.timestamp,
        "track_id": track.id,
        "position": {
            "lat": track.latitude,
            "lon": track.longitude,
            "alt": track.altitude
        },
        "confidence": track.confidence,
        "sources": track.sensor_ids
    });
    
    producer.send_future(
        rdkafka::message::FutureRecord::to("tracks.updated")
            .key(&track.id)
            .payload(&event.to_string())
    ).await.expect("Failed to send");
}
```

## Tradeoffs

### Advantages
- Battle-tested at massive scale
- Immutable audit trail (legal compliance)
- Multi-site replication built-in
- Perfect for federated learning

### Disadvantages
- Operational complexity (ZooKeeper, multiple brokers)
- Network overhead (multiple brokers)
- Learning curve for Kafka concepts
- Storage requirements (3 replicas + retention)

## Mitigation

### Complexity
- **Cloud**: Use AWS MSK (fully managed)
- **Edge**: Use Kafka in containers with Helm
- **Monitoring**: Confluent Control Center for ops

### Storage
```yaml
# Retention policy
retention.ms: 604800000  # 7 days
compression.type: snappy # Reduce disk usage
log.segment.bytes: 1073741824  # 1GB segments
```

## Consequences

- Complete audit trail for all defense decisions
- Incident investigation via event replay
- ML training data from real battles
- Multi-site coordination via Kafka Connect
- Regulatory compliance (immutable logs)

## Related Decisions

- [ADR-007: Event Sourcing Architecture](ADR-007-event-sourcing.md)
- [ADR-010: Federated Learning](ADR-010-federated-learning.md)

## References

- [Kafka Architecture](https://kafka.apache.org/intro)
- [Kafka Best Practices](https://kafka.apache.org/documentation/#bestpractices)
- [Event Sourcing Pattern](https://martinfowler.com/eaaDev/EventSourcing.html)
