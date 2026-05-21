# Production Deployment Playbook

## Pre-Deployment Checklist

### Infrastructure Requirements
- [ ] Kubernetes cluster ready (K3s or managed service)
- [ ] PostgreSQL 15+ installed and configured
- [ ] Kafka 7.5+ cluster deployed
- [ ] etcd 3.5+ for distributed consensus
- [ ] Redis 7.0+ for caching
- [ ] Prometheus + Grafana for monitoring
- [ ] Network connectivity verified between all components

### Security Requirements
- [ ] TLS certificates generated (or HSM configured)
- [ ] RBAC policies created in etcd
- [ ] Firewall rules configured (see Network Security section)
- [ ] Secrets management configured (Vault/K8s Secrets)
- [ ] Audit logging enabled on PostgreSQL
- [ ] Network policies enabled in Kubernetes

### Operational Requirements
- [ ] On-call rotation scheduled
- [ ] Runbooks distributed to team
- [ ] Monitoring dashboards created
- [ ] Alert channels configured (PagerDuty/Slack)
- [ ] Backup procedures validated
- [ ] Disaster recovery plan reviewed

---

## Phase 1: Environment Setup (Day 1)

### Step 1.1: Deploy Infrastructure

```bash
# 1. Create Kubernetes namespace
kubectl create namespace sentinel-swarm-shield
kubectl label namespace sentinel-swarm-shield environment=production

# 2. Create persistent volumes for data
kubectl apply -f infra/kubernetes/persistent-volumes.yaml

# 3. Deploy database
kubectl apply -f infra/kubernetes/postgresql-deployment.yaml
kubectl wait --for=condition=ready pod -l app=postgres -n sentinel-swarm-shield --timeout=300s

# 4. Initialize database
kubectl exec -it postgres-0 -n sentinel-swarm-shield -- \
  psql -U postgres -f /migrations/001-initial-schema.sql

# 5. Deploy Kafka
kubectl apply -f infra/kubernetes/kafka-deployment.yaml
kubectl wait --for=condition=ready pod -l app=kafka -n sentinel-swarm-shield --timeout=300s

# 6. Deploy etcd
kubectl apply -f infra/kubernetes/etcd-deployment.yaml
kubectl wait --for=condition=ready pod -l app=etcd -n sentinel-swarm-shield --timeout=300s

# 7. Deploy Redis
kubectl apply -f infra/kubernetes/redis-deployment.yaml
kubectl wait --for=condition=ready pod -l app=redis -n sentinel-swarm-shield --timeout=300s

# 8. Verify all infrastructure is healthy
kubectl get pods -n sentinel-swarm-shield
kubectl get svc -n sentinel-swarm-shield
```

### Step 1.2: Configure Secrets

```bash
# Create TLS certificates
./scripts/generate-certificates.sh sentinel-swarm-shield

# Create Kubernetes secrets
kubectl create secret tls sentinel-tls \
  --cert=certs/ca.crt \
  --key=certs/ca.key \
  -n sentinel-swarm-shield

# Create database credentials
kubectl create secret generic db-credentials \
  --from-literal=username=postgres \
  --from-literal=password=$(openssl rand -base64 32) \
  -n sentinel-swarm-shield

# Create API credentials
kubectl create secret generic api-credentials \
  --from-literal=graphql-secret=$(openssl rand -base64 32) \
  -n sentinel-swarm-shield
```

### Step 1.3: Deploy Monitoring

```bash
# Deploy Prometheus
kubectl apply -f infra/monitoring/prometheus-deployment.yaml

# Deploy Grafana
kubectl apply -f infra/monitoring/grafana-deployment.yaml

# Deploy AlertManager
kubectl apply -f infra/monitoring/alertmanager-deployment.yaml

# Wait for monitoring stack
kubectl wait --for=condition=ready pod -l app=prometheus -n sentinel-swarm-shield --timeout=300s
kubectl wait --for=condition=ready pod -l app=grafana -n sentinel-swarm-shield --timeout=300s
```

---

## Phase 2: Deploy Services (Day 2)

### Step 2.1: Build and Push Docker Images

```bash
# Build all services
cd /path/to/sentinel-swarm-shield
make docker-build

# Push to registry (adjust registry URL)
REGISTRY=your-registry.azurecr.io
docker tag sentinel/fusion-engine:latest $REGISTRY/sentinel/fusion-engine:v1.0
docker tag sentinel/orchestrator:latest $REGISTRY/sentinel/orchestrator:v1.0
docker tag sentinel/api-gateway:latest $REGISTRY/sentinel/api-gateway:v1.0
docker tag sentinel/ai-services:latest $REGISTRY/sentinel/ai-services:v1.0
docker tag sentinel/defense-executor:latest $REGISTRY/sentinel/defense-executor:v1.0
docker tag sentinel/command-center:latest $REGISTRY/sentinel/command-center:v1.0

docker push $REGISTRY/sentinel/fusion-engine:v1.0
docker push $REGISTRY/sentinel/orchestrator:v1.0
docker push $REGISTRY/sentinel/api-gateway:v1.0
docker push $REGISTRY/sentinel/ai-services:v1.0
docker push $REGISTRY/sentinel/defense-executor:v1.0
docker push $REGISTRY/sentinel/command-center:v1.0
```

### Step 2.2: Deploy Core Services

```bash
# Deploy Fusion Engine
kubectl apply -f infra/kubernetes/deployment-fusion-engine.yaml
kubectl wait --for=condition=ready pod -l app=fusion-engine -n sentinel-swarm-shield --timeout=300s

# Deploy AI Services
kubectl apply -f infra/kubernetes/deployment-ai-services.yaml
kubectl wait --for=condition=ready pod -l app=ai-services -n sentinel-swarm-shield --timeout=300s

# Deploy Orchestrator
kubectl apply -f infra/kubernetes/deployment-orchestrator.yaml
kubectl wait --for=condition=ready pod -l app=orchestrator -n sentinel-swarm-shield --timeout=300s

# Deploy Defense Executor
kubectl apply -f infra/kubernetes/deployment-defense-executor.yaml
kubectl wait --for=condition=ready pod -l app=defense-executor -n sentinel-swarm-shield --timeout=300s

# Deploy API Gateway
kubectl apply -f infra/kubernetes/deployment-api-gateway.yaml
kubectl wait --for=condition=ready pod -l app=api-gateway -n sentinel-swarm-shield --timeout=300s

# Deploy Command Center
kubectl apply -f infra/kubernetes/deployment-command-center.yaml
kubectl wait --for=condition=ready pod -l app=command-center -n sentinel-swarm-shield --timeout=300s

# Verify all services
kubectl get pods -n sentinel-swarm-shield
kubectl get svc -n sentinel-swarm-shield
```

### Step 2.3: Health Check All Services

```bash
#!/bin/bash

echo "Checking service health..."

# Fusion Engine
curl -s http://fusion-engine:50051/health || echo "❌ Fusion Engine unhealthy"
echo "✓ Fusion Engine OK"

# AI Services
curl -s http://ai-services:8000/health || echo "❌ AI Services unhealthy"
echo "✓ AI Services OK"

# Orchestrator
curl -s http://orchestrator:50052/health || echo "❌ Orchestrator unhealthy"
echo "✓ Orchestrator OK"

# Defense Executor
curl -s http://defense-executor:50053/health || echo "❌ Defense Executor unhealthy"
echo "✓ Defense Executor OK"

# API Gateway
curl -s http://api-gateway:4000/health || echo "❌ API Gateway unhealthy"
echo "✓ API Gateway OK"

# Command Center
curl -s http://command-center:3000/health || echo "❌ Command Center unhealthy"
echo "✓ Command Center OK"

echo "All services healthy!"
```

---

## Phase 3: Integration Testing (Day 3)

### Step 3.1: Load Testing

```bash
# Run load test (5000 tracks/sec for 1 hour)
go run cmd/load-test/main.go \
  --api-endpoint=http://api-gateway:4000 \
  --tracks-per-second=5000 \
  --duration=3600s \
  --output=load-test-results.json

# Verify results
cat load-test-results.json | jq '.summary'
# Expected:
# {
#   "p50_latency_ms": 35,
#   "p95_latency_ms": 65,
#   "p99_latency_ms": 90,
#   "error_rate": 0.05,
#   "success": true
# }
```

### Step 3.2: End-to-End Testing

```bash
# Run E2E test suite
go test -tags=e2e ./tests/e2e -v -timeout=30m

# Expected output:
# ✓ TestThreatDetectionFlow
# ✓ TestOrchestrationFlow
# ✓ TestApprovalWorkflow
# ✓ TestFederationSync
# ✓ TestEdgeDeployment
# All tests passed!
```

### Step 3.3: Chaos Engineering Validation

```bash
# Kill random pods every 30 seconds
kubectl inject chaos \
  --namespace sentinel-swarm-shield \
  --failure-type pod-kill \
  --interval=30s \
  --duration=600s

# Monitor recovery
watch 'kubectl get pods -n sentinel-swarm-shield'

# Verify no data loss
./scripts/verify-data-integrity.sh sentinel-swarm-shield
```

---

## Phase 4: Enable Monitoring (Day 4)

### Step 4.1: Configure Prometheus

```bash
# Apply Prometheus config
kubectl apply -f infra/monitoring/prometheus-config.yaml

# Apply alert rules
kubectl apply -f infra/monitoring/alert-rules.yaml

# Verify scrape targets
curl -s http://prometheus:9090/api/v1/targets | jq '.data.activeTargets | length'
# Expected: 8 (all services)
```

### Step 4.2: Import Grafana Dashboards

```bash
# Get Grafana admin password
GRAFANA_PASSWORD=$(kubectl get secret grafana-admin -n sentinel-swarm-shield -o jsonpath='{.data.password}' | base64 -d)

# Import dashboards via API
for dashboard in infra/monitoring/dashboards/*.json; do
  curl -X POST http://admin:$GRAFANA_PASSWORD@grafana:3000/api/dashboards/db \
    -H "Content-Type: application/json" \
    -d @$dashboard
done

echo "Dashboards imported successfully"
```

### Step 4.3: Configure Alerting

```bash
# Configure AlertManager for Slack/PagerDuty
kubectl apply -f infra/monitoring/alertmanager-config.yaml

# Test alert
kubectl exec -it prometheus-0 -n sentinel-swarm-shield -- \
  promtool query instant 'up{job="api-gateway"}'

# Simulate alert
kubectl scale deployment api-gateway -n sentinel-swarm-shield --replicas=0
# Alert should fire within 2 minutes
sleep 120
# Verify alert in Grafana/Slack

# Restore deployment
kubectl scale deployment api-gateway -n sentinel-swarm-shield --replicas=3
```

---

## Phase 5: Production Cutover (Day 5)

### Step 5.1: Backup Procedures

```bash
# Backup database
kubectl exec postgres-0 -n sentinel-swarm-shield -- \
  pg_dump -U postgres sentinel_edge > sentinel_edge_backup_$(date +%s).sql

# Backup configuration
kubectl get configmap -n sentinel-swarm-shield -o yaml > configmaps-backup.yaml
kubectl get secret -n sentinel-swarm-shield -o yaml > secrets-backup.yaml

# Verify backups
ls -lh sentinel_edge_backup_*.sql
ls -lh configmaps-backup.yaml secrets-backup.yaml
```

### Step 5.2: Activate Monitoring

```bash
# Enable PagerDuty integration
kubectl set env deployment/alertmanager -n sentinel-swarm-shield \
  PAGERDUTY_KEY=$PAGERDUTY_KEY

# Enable Slack integration
kubectl set env deployment/alertmanager -n sentinel-swarm-shield \
  SLACK_WEBHOOK=$SLACK_WEBHOOK

# Test alert channels
kubectl exec alertmanager-0 -n sentinel-swarm-shield -- \
  amtool alert query
```

### Step 5.3: Start 24/7 Monitoring

```bash
# Export Prometheus for long-term storage (optional)
kubectl apply -f infra/monitoring/remote-write-config.yaml

# Setup SLA dashboard
open http://grafana:3000/d/sla-dashboard

# Verify all metrics are flowing
curl -s http://prometheus:9090/api/v1/query?query=up | jq '.data.result | length'
# Expected: 8+
```

### Step 5.4: Team Notification

```bash
# Notify on-call team
./scripts/notify-team.sh "Sentinel-Swarm-Shield production deployment complete"

# Send email to stakeholders
./scripts/send-deployment-report.sh \
  --recipients="ops-team@company.com" \
  --subject="Sentinel-Swarm-Shield v1.0 Production Deployment"
```

---

## Rollback Procedures

### If Issues Occur (Immediate Response)

```bash
# Scale down problematic service
kubectl scale deployment fusion-engine -n sentinel-swarm-shield --replicas=0

# Restore from backup
psql -h postgres.sentinel-swarm-shield.svc.cluster.local -U postgres \
  sentinel_edge < sentinel_edge_backup_TIMESTAMP.sql

# Verify data integrity
./scripts/verify-data-integrity.sh sentinel-swarm-shield

# Scale service back up
kubectl scale deployment fusion-engine -n sentinel-swarm-shield --replicas=3

# Monitor recovery
kubectl logs -f deployment/fusion-engine -n sentinel-swarm-shield
```

### Full Rollback (If Critical Issue)

```bash
# Delete all deployments
kubectl delete deployment --all -n sentinel-swarm-shield

# Restore from previous version (if needed)
git checkout v0.9.0

# Redeploy
make docker-build
# ... follow Phase 2 deployment steps with previous version
```

---

## Post-Deployment (Day 6-7)

### Step 6.1: Performance Baseline

```bash
# Collect 24-hour baseline metrics
./scripts/collect-baseline-metrics.sh --duration=24h

# Generate baseline report
./scripts/generate-baseline-report.sh baseline-metrics.json > baseline-report.md

# Review report
cat baseline-report.md
```

### Step 6.2: Team Training

```bash
# Schedule training sessions
./scripts/schedule-training.sh

# Distribute documentation
./scripts/distribute-docs.sh \
  --recipients="ops-team@company.com" \
  --files="docs/OPERATIONAL-RUNBOOKS.md,docs/DEVELOPMENT.md"
```

### Step 6.3: Handoff to Operations

```bash
# Create operations wiki
./scripts/create-ops-wiki.sh

# Transfer on-call duties
./scripts/transfer-oncall.sh --from=deployment-team --to=ops-team

# Verify ops team access
kubectl auth can-i get pods --as=ops-team@company.com --namespace=sentinel-swarm-shield
```

---

## Troubleshooting Guide

### Service Won't Start
```bash
# Check logs
kubectl logs deployment/SERVICE_NAME -n sentinel-swarm-shield

# Check resource limits
kubectl describe pod -n sentinel-swarm-shield -l app=SERVICE_NAME

# Check readiness probes
kubectl get pod -n sentinel-swarm-shield -l app=SERVICE_NAME -o yaml | grep readiness
```

### Database Connection Issues
```bash
# Test connection
kubectl exec -it postgres-0 -n sentinel-swarm-shield -- \
  psql -U postgres -c "SELECT version();"

# Check connection pool
psql -h postgres.sentinel-swarm-shield.svc.cluster.local -U postgres -d sentinel_edge \
  -c "SELECT datname, count(*) FROM pg_stat_activity GROUP BY datname;"
```

### High Latency
```bash
# Check service metrics
curl -s http://prometheus:9090/api/v1/query?query=histogram_quantile | jq '.data.result'

# Check resource utilization
kubectl top nodes
kubectl top pods -n sentinel-swarm-shield

# Check network latency
kubectl exec -it api-gateway-0 -n sentinel-swarm-shield -- \
  ping fusion-engine
```

---

**Deployment Checklist Complete ✓**

Expected Timeline: 7 days from infrastructure setup to full production operation

Next: Monitor SLAs, collect feedback, plan optimizations
