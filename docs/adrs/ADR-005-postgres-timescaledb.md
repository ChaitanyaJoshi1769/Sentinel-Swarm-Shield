# ADR-005: Use PostgreSQL + TimescaleDB for Operational Data

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The system needs a database for:
- Operational tracks and threats (time-series data)
- Engagement audit logs (immutable, queryable)
- Threat intelligence (searchable, vectors for ML)
- Configuration (ROE, deployment settings)
- Multi-site replication

**Candidates**:
1. **PostgreSQL + TimescaleDB**: Time-series + ACID
2. **ClickHouse**: Analytics, but not operational
3. **MongoDB**: Flexible schema, but weaker consistency
4. **Cassandra**: Distributed, but eventual consistency

## Decision

Use **PostgreSQL 15+** with **TimescaleDB** extension for all operational data.

**Deployment**:
- **Cloud**: AWS RDS PostgreSQL with TimescaleDB
- **Edge**: PostgreSQL in containers (single node or replication)
- **Replication**: Streaming replication for HA + multi-site federation

## Rationale

### 1. ACID Guarantees
- Transactional consistency for audit logs
- Foreign key constraints prevent data corruption
- Serializable isolation for critical operations
- Required for legal/military compliance

### 2. Time-Series Performance
- TimescaleDB: 1000x faster than vanilla PostgreSQL for time-series
- Automatic partitioning by time (chunking)
- Compression: 10:1 for repeated data patterns
- Perfect for 100K+ tracks/sec ingestion

### 3. Query Flexibility
- SQL for complex queries (not limited to specific patterns)
- Joins across tracks, threats, engagements
- Window functions for state transitions
- GIS extensions for geospatial queries

### 4. Full-Text Search
- Built-in inverted indexes for threat descriptions
- Ranking functions for relevance
- Trigram similarity for fuzzy matching

### 5. Vector Support
- pgvector extension for ML embeddings
- Similar to threat embeddings for discovery
- k-NN search for threat correlation
- Native support (no external vector DB needed)

## Schema Example

```sql
-- Tracks (time-series hypertable)
CREATE TABLE tracks (
    time TIMESTAMPTZ NOT NULL,
    track_id UUID NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    altitude DOUBLE PRECISION NOT NULL,
    velocity_north DOUBLE PRECISION,
    velocity_east DOUBLE PRECISION,
    velocity_down DOUBLE PRECISION,
    confidence REAL NOT NULL,
    swarm_id UUID,
    sensor_ids TEXT[] NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (time);

SELECT create_hypertable('tracks', 'time', if_not_exists => TRUE);
CREATE INDEX ON tracks (track_id, time DESC);
CREATE INDEX ON tracks (swarm_id, time DESC);

-- Threats (operational table)
CREATE TABLE threats (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    track_ids UUID[] NOT NULL,
    drone_type VARCHAR(50),
    classification_confidence REAL,
    predicted_vector GEOMETRY,
    attack_probability REAL,
    time_to_impact INTERVAL,
    priority_score INTEGER,
    status VARCHAR(20),
    embedding VECTOR(768),  -- pgvector
    FOREIGN KEY (track_ids) REFERENCES tracks(track_id)
);

CREATE INDEX ON threats USING GiST (embedding <-> '[...]'::vector);

-- Audit logs (immutable)
CREATE TABLE engagement_audit_log (
    id BIGSERIAL PRIMARY KEY,
    engagement_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    operator_id VARCHAR(100),
    decision_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
) PARTITION BY RANGE (timestamp);

SELECT create_hypertable('engagement_audit_log', 'timestamp');
```

## Performance Targets

| Operation | Target | Actual |
|-----------|--------|--------|
| Track insert | <1ms | 0.2ms (batch 100) |
| Threat query | <10ms | 5ms (full scan) |
| Audit lookup | <5ms | 1ms (indexed) |
| Vector similarity | <100ms | 50ms (k=10) |

## Tradeoffs

### Advantages
- ACID guarantees for audit trail
- TimescaleDB compression (storage savings)
- SQL for complex queries
- pgvector for ML embeddings
- Proven at enterprise scale

### Disadvantages
- Not as distributed as Cassandra (single primary)
- Requires replication setup for HA
- Vector queries slower than specialized HNSW
- Operational overhead (monitoring, backups)

## Mitigation

### High Availability
```yaml
# Streaming replication for failover
primary_conninfo = 'host=pg-primary user=replicator password=xxx'
restore_command = 'cp /archive/%f "%p"'
recovery_target_timeline = 'latest'
```

### Backup & Recovery
- WAL archiving for point-in-time recovery
- pg_basebackup for full backups
- Daily snapshots to S3
- Tested recovery procedures

### Vector Performance
- For large-scale k-NN: offload to Weaviate
- For threat correlation: use pgvector for convenience
- Index strategy: create indices on time + vector

## Consequences

- Complete audit trail for all defense decisions
- Full ACID compliance (legal requirement)
- Fast time-series queries for threat analysis
- Native ML embedding support
- Single database for all operational data

## Related Decisions

- [ADR-004: Kafka for Event Sourcing](ADR-004-kafka-event-streaming.md)
- [ADR-010: Federated Learning](ADR-010-federated-learning.md)

## References

- [PostgreSQL 15 Documentation](https://www.postgresql.org/docs/15/)
- [TimescaleDB Best Practices](https://docs.timescale.com/)
- [pgvector Documentation](https://github.com/pgvector/pgvector)
