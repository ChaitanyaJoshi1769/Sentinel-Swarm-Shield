package orchestrator

import (
	"context"
	"encoding/json"
	"fmt"
	"log"

	"github.com/segmentio/kafka-go"
	"go.etcd.io/etcd/client/v3"
)

// Orchestrator handles defense coordination
type Orchestrator struct {
	kafkaConn  *kafka.Conn
	kafkaReader *kafka.Reader
	etcdClient *clientv3.Client
	ctx        context.Context
}

// Threat represents a detected threat
type Threat struct {
	ID                 string
	DroneType          string
	AttackProbability  float64
	TimeToImpactSecs   float64
	PriorityScore      int
	PredictedVector    map[string]float64
}

// EngagementPlan represents defense action plan
type EngagementPlan struct {
	ID                   string
	ThreatID            string
	InterceptorAssignments []string
	SuccessProbability   float64
	CollateralDamage     float64
	Status              string
}

// New creates a new orchestrator
func New(ctx context.Context, kafkaBrokers, etcdEndpoint string) (*Orchestrator, error) {
	// Initialize Kafka reader for threat stream
	reader := kafka.NewReader(kafka.ReaderConfig{
		Brokers: []string{kafkaBrokers},
		Topic:   "threats.detected",
		GroupID: "orchestrator",
	})

	// Initialize etcd client
	etcdCli, err := clientv3.New(clientv3.Config{
		Endpoints: []string{etcdEndpoint},
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create etcd client: %w", err)
	}

	return &Orchestrator{
		kafkaReader: reader,
		etcdClient: etcdCli,
		ctx:        ctx,
	}, nil
}

// ProposalEngagement creates an engagement plan for a threat
func (o *Orchestrator) ProposalEngagement(threatID string, threat *Threat) (*EngagementPlan, error) {
	log.Printf("Proposing engagement for threat %s", threatID)

	plan := &EngagementPlan{
		ID:                   fmt.Sprintf("engagement-%s", threatID),
		ThreatID:            threatID,
		InterceptorAssignments: o.assignInterceptors(threat),
		SuccessProbability:   o.estimateSuccessProbability(threat),
		CollateralDamage:     0.0,
		Status:              "proposed",
	}

	// Store plan in etcd
	planJSON, _ := json.Marshal(plan)
	_, err := o.etcdClient.Put(o.ctx, fmt.Sprintf("/engagements/%s", plan.ID), string(planJSON))
	if err != nil {
		return nil, fmt.Errorf("failed to store engagement plan: %w", err)
	}

	return plan, nil
}

// assignInterceptors assigns interceptor resources to a threat
func (o *Orchestrator) assignInterceptors(threat *Threat) []string {
	// Simple heuristic: assign based on threat severity
	var assignments []string

	if threat.PriorityScore > 70 {
		// High priority: assign 2 interceptors
		assignments = append(assignments, "interceptor-1", "interceptor-2")
	} else if threat.PriorityScore > 40 {
		// Medium priority: assign 1 interceptor
		assignments = append(assignments, "interceptor-1")
	}

	return assignments
}

// estimateSuccessProbability estimates engagement success
func (o *Orchestrator) estimateSuccessProbability(threat *Threat) float64 {
	// Baseline from threat characteristics
	prob := threat.AttackProbability * 0.8

	// Adjust for time-to-impact (more time = higher success)
	timeBonus := 1.0 - (1.0 / (1.0 + threat.TimeToImpactSecs))
	prob += timeBonus * 0.2

	if prob > 1.0 {
		prob = 1.0
	}
	return prob
}

// ListEngagements lists active engagements
func (o *Orchestrator) ListEngagements() ([]*EngagementPlan, error) {
	resp, err := o.etcdClient.Get(o.ctx, "/engagements/", clientv3.WithPrefix())
	if err != nil {
		return nil, err
	}

	var plans []*EngagementPlan
	for _, kv := range resp.Kvs {
		var plan EngagementPlan
		if err := json.Unmarshal(kv.Value, &plan); err != nil {
			log.Printf("Failed to unmarshal engagement plan: %v", err)
			continue
		}
		plans = append(plans, &plan)
	}

	return plans, nil
}

// GetEngagementStatus retrieves status of an engagement
func (o *Orchestrator) GetEngagementStatus(engagementID string) (*EngagementPlan, error) {
	resp, err := o.etcdClient.Get(o.ctx, fmt.Sprintf("/engagements/%s", engagementID))
	if err != nil {
		return nil, err
	}

	if len(resp.Kvs) == 0 {
		return nil, fmt.Errorf("engagement not found: %s", engagementID)
	}

	var plan EngagementPlan
	if err := json.Unmarshal(resp.Kvs[0].Value, &plan); err != nil {
		return nil, err
	}

	return &plan, nil
}

// Close closes the orchestrator
func (o *Orchestrator) Close() error {
	o.kafkaReader.Close()
	return o.etcdClient.Close()
}
