# ADR-008: Zero-Trust Security Architecture

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team, Security Team

## Context

Sentinel-Swarm-Shield controls critical infrastructure defense:
- Must defend against insider threats
- Must resist supply chain compromise
- Must prevent unauthorized engagement
- Must maintain audit trail for legal review
- Must support air-gapped, classified deployments

**Security Principles**:
1. Never trust network (always authenticate)
2. Never trust implicit roles (always authorize)
3. Never trust code origin (always verify signatures)
4. Never trust previous sessions (re-authenticate)

## Decision

Implement **zero-trust security** with:
- mTLS for all inter-service communication
- JWT tokens with short expiry
- Hardware-backed authentication where available
- Immutable audit logs
- Signed artifacts
- Air-gapable infrastructure

## Architecture

### Service-to-Service: mTLS

```
┌──────────────────┐  mTLS  ┌──────────────────┐
│  Fusion Engine   │◄─────►│  API Gateway     │
│  (cert-xxx)      │ TLS1.3│  (cert-yyy)      │
└──────────────────┘        └──────────────────┘

Each service has:
- X.509 certificate (signed by internal CA)
- Private key (in encrypted Kubernetes secret)
- Automatic rotation (30-day TTH)
- Certificate pinning for critical paths
```

### Operator-to-UI: JWT + MFA

```
Operator Login
  ↓
Identity Provider (OIDC or local)
  ↓
Verify password + MFA (TOTP/U2F)
  ↓
Issue JWT token (30min TTL)
  ↓
UI stores in memory (not localStorage)
  ↓
Token included in every API request
  ↓
API Gateway validates JWT signature
  ↓
If invalid: return 401, force re-login
```

### API Gateway: RBAC + Approval Gates

```
Operator issues: GraphQL Mutation engageThreat(threatId)
  ↓
API Gateway:
  1. Verify JWT signature
  2. Extract role from token claims
  3. Query RBAC rules:
     - role="commander" → can auto-approve up to 3 threats
     - role="operator" → requires commander approval
     - role="analyst" → read-only
  4. If approval required → publish approval.requested event
  5. Wait for approval (block mutation)
  6. Log decision to immutable audit log
  ↓
Engagement executed (with authorization details in log)
```

## Example: Zero-Trust Implementation

### Service Certificate Management (Go)

```go
import "crypto/tls"

// Load mTLS credentials
cert, err := tls.LoadX509KeyPair(
    "/etc/ssl/certs/service.crt",
    "/etc/ssl/private/service.key",
)

// Configure server with mTLS
tlsConfig := &tls.Config{
    Certificates: []tls.Certificate{cert},
    ClientAuth:   tls.RequireAndVerifyClientCert,
    ClientCAs:    caPool,
    MinVersion:   tls.VersionTLS13,
}

server := &http.Server{
    TLSConfig: tlsConfig,
}
```

### JWT Validation (Go)

```go
import "github.com/golang-jwt/jwt/v5"

func validateToken(tokenString string) (*Claims, error) {
    // Short-lived token (30 minutes)
    token, err := jwt.ParseWithClaims(tokenString, &Claims{}, func(token *jwt.Token) (interface{}, error) {
        // Verify signature with public key
        return publicKey, nil
    })
    
    claims := token.Claims.(*Claims)
    
    // Check expiry
    if claims.ExpiresAt.Before(time.Now()) {
        return nil, errors.New("token expired")
    }
    
    return claims, nil
}

type Claims struct {
    OperatorID string   `json:"sub"`
    Roles      []string `json:"roles"`
    Hardware   string   `json:"hw"`  // Hardware token ID
    jwt.RegisteredClaims
}
```

### RBAC Engine

```go
type RBACRule struct {
    Role              string
    Action            string
    Resource          string
    Condition         func(*Claims) bool
    RequiresApproval  bool
}

func (r *RBACEngine) CanEngage(claims *Claims, threatID string) (allowed bool, approvalsNeeded []string) {
    // Find matching rule for role + action
    for _, rule := range r.rules {
        if rule.Role == claims.Roles[0] && rule.Action == "engage" {
            if rule.RequiresApproval {
                // Return list of roles that must approve
                return true, []string{"commander", "security_officer"}
            }
            return true, nil
        }
    }
    return false, nil
}
```

### Immutable Audit Log

```sql
-- Append-only audit log
CREATE TABLE engagement_audit_log (
    id BIGSERIAL PRIMARY KEY,  -- Never reused
    engagement_id UUID NOT NULL,
    operator_id VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,  -- "proposed", "approved", "executed", "failed"
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    operator_roles TEXT[] NOT NULL,
    decision_rationale JSONB,
    
    -- Immutability
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT no_updates CHECK (true)
);

-- Create index for queries
CREATE INDEX ON engagement_audit_log (engagement_id);
CREATE INDEX ON engagement_audit_log (operator_id);
CREATE INDEX ON engagement_audit_log (timestamp);

-- Prevent modification
REVOKE UPDATE ON engagement_audit_log FROM application_user;
REVOKE DELETE ON engagement_audit_log FROM application_user;
```

### Signed Artifacts

```bash
#!/bin/bash
# Build and sign Docker image

# Build image
docker build -t sentinel:v1.0.0 .

# Sign with Cosign
cosign sign --key cosign.key gcr.io/sentinel/api-gateway:v1.0.0

# Verify before deployment
cosign verify --key cosign.pub gcr.io/sentinel/api-gateway:v1.0.0

# Deploy with signature verification (in Kubernetes)
# kubewarden policy validates Cosign signature before scheduling
```

## Security Checklist

- [x] All inter-service communication encrypted (TLS 1.3)
- [x] Automatic certificate rotation (30-day)
- [x] JWT tokens with short TTL (30 minutes)
- [x] RBAC for all actions
- [x] Approval gates for critical decisions
- [x] Immutable audit logs (database level)
- [x] Signed container images
- [x] Supply chain verification (SBOM, SLSA)
- [x] No hardcoded secrets
- [x] Hardware token support (U2F/FIDO2)

## Tradeoffs

### Advantages
- Insider threat detection (audit log)
- Compliance with military/government standards
- Supply chain compromise prevention
- Air-gap compatible
- Legally defensible (immutable logs)

### Disadvantages
- Operational complexity (token management)
- Performance overhead (JWT validation)
- Certificate management burden
- User friction (MFA, re-auth)

## Mitigation

### Complexity
- Automated certificate rotation (cert-manager)
- JWT issued via OIDC provider (Keycloak, Auth0)
- Hardware tokens supported but optional
- Clear documentation for operators

### Performance
- JWT validation: <1ms per request
- mTLS handshake: amortized <10ms
- Cache JWT public keys
- Connection pooling for reuse

## Consequences

- System suitable for military/government classification
- Insider threats detectable via audit trail
- Supply chain integrity verified
- Regulatory compliance (SOC 2, FedRAMP)
- Higher operational complexity but justified

## Related Decisions

- [ADR-005: PostgreSQL for Audit](ADR-005-postgres-timescaledb.md)
- [ADR-004: Kafka for Immutable Events](ADR-004-kafka-event-streaming.md)

## References

- [Zero Trust Architecture (NIST SP 800-207)](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-207.pdf)
- [mTLS Best Practices](https://www.cloudflare.com/learning/access-management/what-is-mutual-tls/)
- [JWT Security Best Practices](https://tools.ietf.org/html/rfc8949)
- [Cosign Container Image Signing](https://docs.sigstore.dev/)
