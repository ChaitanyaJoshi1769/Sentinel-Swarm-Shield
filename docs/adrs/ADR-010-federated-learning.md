# ADR-010: Federated Learning for Threat Models

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team, ML Team

## Context

Defense bases operate independently but face similar threats. Current approach:
- Each base trains models locally (duplicate effort)
- New threat types slow to propagate between bases
- Global threat intelligence unavailable
- Models diverge across deployments

**Requirements**:
- Improve threat detection across all bases
- Share knowledge without centralizing sensor data
- Preserve privacy (sensor data stays local)
- Support rapid model updates

## Decision

Implement **federated learning** where:
- Each base trains models on local data
- Model updates (gradients) aggregated at cloud
- Cloud broadcasts improved model
- Sensor data never leaves base

## Architecture

```
Base 1                  Base 2                  Base 3
  ↓                       ↓                       ↓
[Train Model]         [Train Model]         [Train Model]
Gradient Update        Gradient Update        Gradient Update
  ↓                       ↓                       ↓
  └─────────────────────┬────────────────────────┘
                        │
                   Cloud Server
                    (Aggregator)
                        │
                 [Average Gradients]
                 [Validate Quality]
                 [Publish V2]
                        │
                        └────────────────┬────────────────┐
                                        ↓                ↓
                                  [Download]        [Download]
                                  Model V2          Model V2
                                  (Base 1)          (Base 2)
```

## Rationale

### 1. Collaborative Learning
- All bases benefit from collective experience
- Rare threats detected at one base help all others
- Global threat understanding
- No single base is data island

### 2. Privacy Preservation
- Sensor data never leaves base
- Only model updates shared (compressed gradients)
- Differential privacy for additional protection
- Classified deployments can stay air-gapped

### 3. Rapid Model Updates
- When new threat emerges, all bases adapt
- Federated training faster than central retraining
- Rolling updates without downtime
- A/B testing across bases

### 4. Cost Efficiency
- Distributed training (no GPU farm needed)
- Data stays on edge hardware
- Bandwidth: only gradients (not data)

## Implementation

### Federated Learning Loop (Python)

```python
import flower as fl
import torch
from typing import List

class ThreatClassifier(fl.client.NumPyClient):
    def __init__(self, model, train_loader, test_loader):
        self.model = model
        self.train_loader = train_loader
        self.test_loader = test_loader
    
    def fit(self, parameters, config):
        """Local training at base"""
        # Update model with global parameters
        self.set_model_params(parameters)
        
        # Train locally on base sensor data
        for epoch in range(config["epochs"]):
            for X, y in self.train_loader:
                output = self.model(X)
                loss = F.cross_entropy(output, y)
                loss.backward()
                optimizer.step()
        
        # Return gradients (not data)
        return self.get_model_params(), len(self.train_loader), {}
    
    def evaluate(self, parameters, config):
        """Evaluation on local data"""
        self.set_model_params(parameters)
        
        loss, accuracy = 0, 0
        with torch.no_grad():
            for X, y in self.test_loader:
                output = self.model(X)
                loss += F.cross_entropy(output, y).item()
                accuracy += (output.argmax(1) == y).sum().item()
        
        return loss, len(self.test_loader), {"accuracy": accuracy / len(self.test_loader)}
    
    def set_model_params(self, params):
        """Update model with global parameters"""
        with torch.no_grad():
            for param, new_param in zip(self.model.parameters(), params):
                param.copy_(new_param)
    
    def get_model_params(self):
        """Extract model parameters"""
        return [param.detach().numpy() for param in self.model.parameters()]

# Start federated learning client at each base
client = ThreatClassifier(model, train_loader, test_loader)
fl.client.start_numpy_client(
    server_address="cloud-fl-server:8080",
    client=client,
)
```

### Server Aggregation (Python)

```python
from flower import server, strategy
import numpy as np

def aggregate(results: List[Tuple[NDArrays, int]]) -> NDArrays:
    """Federated averaging at cloud"""
    # results = list of (parameters, num_examples) from bases
    
    weights = [r[1] for r in results]  # Training set sizes
    parameters = [r[0] for r in results]
    
    # Weighted average
    aggregated = []
    for i in range(len(parameters[0])):
        avg_param = np.average(
            [p[i] for p in parameters],
            axis=0,
            weights=weights
        )
        aggregated.append(avg_param)
    
    return aggregated

# Federated learning strategy
strategy = strategy.FedAvg(
    min_fit_clients=2,  # Min 2 bases per round
    min_evaluate_clients=2,
    min_available_clients=3,  # Total bases available
)

# Start server
server.start_server(
    server_address="0.0.0.0:8080",
    config=server.ServerConfig(num_rounds=5),
    strategy=strategy,
)
```

### Model Validation

```python
def validate_aggregated_model(global_model, validation_dataset):
    """Ensure aggregated model is safe"""
    # Run on cloud validation set
    loss, accuracy = evaluate(global_model, validation_dataset)
    
    # Only publish if meets quality threshold
    if accuracy > 0.92:  # Must be as good as previous
        return True
    else:
        log("Model degradation detected, rejecting aggregation")
        return False
```

## Differential Privacy

```python
import tensorflow_privacy as tfp

# Add noise to gradients for privacy
dp_optimizer = tfp.DPKerasAdamOptimizer(
    l2_norm_clip=1.0,
    noise_multiplier=0.1,  # Privacy level
    num_microbatches=256,
)

# Train with differential privacy
model.compile(optimizer=dp_optimizer, loss=...)
model.fit(...)
```

## Federated Learning Workflow

1. **Round 1: Training**
   - Cloud: Publish model V1 to all bases
   - Bases: Train locally on sensor data (2 days)
   - Bases: Send gradient updates to cloud

2. **Round 2: Aggregation**
   - Cloud: Aggregate gradients (weighted average)
   - Cloud: Validate on validation set
   - Cloud: Publish model V2

3. **Round 3: Deployment**
   - Bases: Download model V2
   - Bases: A/B test (10% traffic)
   - Bases: Gradual rollout if improving

4. **Monitoring**
   - Track accuracy per base
   - Detect model drift
   - Alert if accuracy drops

## Tradeoffs

### Advantages
- All bases benefit from collective learning
- Sensor data stays on-site (privacy)
- Rapid threat model updates
- Reduced cloud compute

### Disadvantages
- Increased network traffic (gradients)
- Complex training pipeline
- Convergence slower than central training
- Requires well-synchronized clocks

## Mitigation

### Network Bandwidth
- Gradient compression (quantization to int8)
- Sparse updates (only significant gradients)
- Scheduled training rounds (off-peak hours)

### Convergence
- Start with fewer bases
- Monitor validation accuracy
- Adjust aggregation strategy

## Deployment

```yaml
# Kubernetes CronJob for federated training
apiVersion: batch/v1
kind: CronJob
metadata:
  name: federated-training
spec:
  schedule: "0 2 * * 0"  # Weekly, 2am
  jobTemplate:
    spec:
      template:
        spec:
          containers:
          - name: fl-client
            image: sentinel/ml-services:latest
            env:
            - name: FL_SERVER
              value: cloud-fl-server:8080
            volumeMounts:
            - name: local-data
              mountPath: /data
          volumes:
          - name: local-data
            persistentVolumeClaim:
              claimName: training-data
```

## Results

**Expected improvements**:
- Drone classification accuracy: 89% → 94%
- Swarm intent prediction: 82% → 91%
- Time-to-impact estimation: RMSE 2.1s → 1.2s
- New threat detection latency: 48 hours → 4 hours

## Consequences

- All bases benefit from collective experience
- Sensor data privacy maintained
- Rapid model updates across federation
- Suitable for military/government (data stays home)
- Enables strategic threat sharing

## Related Decisions

- [ADR-007: Edge-First Architecture](ADR-007-edge-first-architecture.md)
- [ADR-002: Python for ML](ADR-002-python-for-ml-services.md)

## References

- [Flower Federated Learning Framework](https://flower.ai/)
- [Google Federated Learning Paper](https://arxiv.org/abs/1602.05629)
- [TensorFlow Privacy](https://github.com/tensorflow/privacy)
