# Operational Runbooks

## Emergency Response Procedures

### Page 1: Service Down - Immediate Response

**Severity**: CRITICAL
**Response Time**: < 5 minutes

#### Scenario: Fusion Engine Service Down

1. **Detect** (automated alert: `FusionEngineDown`)
   - Alert fired when health check fails for 1 minute

2. **Assess**
   ```bash
   kubectl get pod -n sentinel-edge -l app=fusion-engine-edge
   kubectl logs -n sentinel-edge deployment/fusion-engine-edge --tail=100
   kubectl describe pod -n sentinel-edge -l app=fusion-engine-edge
   ```

3. **Immediate Actions**
   ```bash
   # Attempt auto-recovery (kubelet restarts container)
   # If still failing after 2 minutes:
   kubectl delete pod -n sentinel-edge -l app=fusion-engine-edge
   # If restart fails, check persistent storage
   kubectl describe pvc -n sentinel-edge sensor-data-pvc
   ```

4. **Escalation** (if not recovered in 5 minutes)
   - Escalate to Level 2 (Infrastructure team)
   - Initiate failover to secondary site (if available)
   - Activate defense degradation mode (local-only, no federation)

5. **Communication**
   - Notify Command Center: "Fusion Engine degraded, operating in local-only mode"
   - Update status page: `Sensor Fusion: Degraded`

#### Other Critical Services
- **Orchestrator Down**: All threats coordinate individually
- **Defense Executor Down**: Switch to approval-only mode (all threats require human authorization)
- **Database Down**: Switch to in-memory + Kafka replay mode

### Page 2: High Latency Response

**Severity**: HIGH
**SLA Threshold**: 100ms end-to-end

#### Identify Bottleneck
```bash
# Check component latencies
# - Fusion Engine latency: histogram_quantile(0.95, fusion_engine_latency_ms)
# - AI Services latency: histogram_quantile(0.95, ai_inference_latency_ms)
# - Orchestrator latency: histogram_quantile(0.95, orchestrator_decision_latency_ms)

# If Fusion Engine slow:
kubectl top pod -n sentinel-edge -l app=fusion-engine-edge
# Check database query performance
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

# If AI Services slow:
nvidia-smi  # Check GPU utilization
curl http://localhost:8000/metrics | grep model_cache_hit

# If Orchestrator slow:
etcdctl alarm list  # Check etcd health
kafka-consumer-groups --bootstrap-server kafka-edge:29092 --group orchestrator-consumer --describe
```

### Page 3: Database Troubleshooting

#### Connection Pool Exhaustion
```bash
psql -h postgres-edge -U postgres \
  -c "SELECT datname, count(*) FROM pg_stat_activity GROUP BY datname;"

# Kill idle connections if safe
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle';"
```

#### Disk Space Critical
```bash
kubectl exec -it -n sentinel-edge postgres-edge -- df -h /var/lib/postgresql/data
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "VACUUM ANALYZE;"
```

#### Query Performance Degradation
```bash
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "ALTER SYSTEM SET log_statement = 'all';"
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "REINDEX DATABASE sentinel_edge;"
```

### Page 4: Kafka / Event Stream Issues

#### Consumer Group Lag High
```bash
kafka-consumer-groups --bootstrap-server kafka-edge:29092 \
  --group fusion-engine-consumer --describe

# Reset offset to latest if safe
kafka-consumer-groups --bootstrap-server kafka-edge:29092 \
  --group fusion-engine-consumer --reset-offsets --to-latest --execute
```

### Page 5: Approval Workflow Failures

#### Approvals Timing Out
```bash
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "SELECT status, COUNT(*) FROM engagements GROUP BY status;"

# Check pending approvals
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "SELECT * FROM approval_requests WHERE approval_decision IS NULL;"

# Manual approval if stuck (with logging)
psql -h postgres-edge -U postgres -d sentinel_edge \
  -c "UPDATE engagements SET status='approved' WHERE id='<engagement_id>';"
```

### Page 6: Disaster Recovery

#### Database Corruption Recovery
```bash
# Backup current database
kubectl exec -n sentinel-edge postgres-edge -- \
  pg_dump -U postgres sentinel_edge > /backup/sentinel_edge_$(date +%s).sql

# Restore from backup if available
psql -h postgres-edge -U postgres -d sentinel_edge < /backup/sentinel_edge_TIMESTAMP.sql
```

#### Complete Site Failure
1. Activate failover site (if multi-site deployment)
2. Restore database from backup
3. Verify data integrity (row counts, consistency checks)
4. Resume normal operations

### RTO/RPO Targets
| Component | RTO (Recovery Time) | RPO (Recovery Point) |
|-----------|-------------------|-------------------|
| Fusion Engine | 5 minutes | 1 minute (logs) |
| Database | 10 minutes | 6 hours (backups) |
| Kafka | 5 minutes | 1 hour (retention) |
| Full Site | 30 minutes | 6 hours (backups) |

### Escalation Matrix
| Severity | Detection | Response | Escalation |
|----------|-----------|----------|-----------|
| CRITICAL | < 1m | < 5m | Level 1 → Level 2 → Manager |
| HIGH | < 5m | < 15m | Level 1 → Level 2 |
| MEDIUM | < 15m | < 1h | Level 1 → Level 2 (if needed) |
| LOW | < 1h | < 4h | Level 1 |

---

*Last Updated: 2026-05-21*
