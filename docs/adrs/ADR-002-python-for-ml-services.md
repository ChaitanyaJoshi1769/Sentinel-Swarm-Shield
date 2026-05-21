# ADR-002: Use Python for ML Services

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

ML services (classification, prediction, anomaly detection) need to:
- Support pre-trained models (PyTorch, TensorFlow)
- Enable rapid iteration and experimentation
- Integrate with established ML ecosystem
- Scale inference across CPU and GPU
- Support federated learning and model updates

**Candidates**:
1. **Python**: Ecosystem, rapid iteration, GPU support
2. **Rust**: Performance, but immature ML libraries
3. **Java/C#**: Enterprise, but slow iteration
4. **Julia**: Scientific computing, but smaller ecosystem

## Decision

Use **Python** for ML services with **FastAPI** for serving.

**Stack**:
- **Inference**: PyTorch + ONNX
- **Framework**: FastAPI (async, built for ML)
- **Distributed**: Ray for scaling
- **Model Serving**: TorchServe or vLLM
- **Tracking**: MLflow

## Rationale

### 1. Ecosystem
- PyTorch: 100K+ pre-trained models
- TensorFlow: Mature production serving
- Scikit-learn: Classical ML algorithms
- huggingface: Pre-built transformers

### 2. Iteration Speed
- Dynamic typing enables rapid prototyping
- Jupyter notebooks for exploration
- 10x faster development than C++/Rust
- Critical for adversarial robustness research

### 3. GPU Support
- CUDA/cuDNN integration native
- TensorRT for optimized inference
- Multi-GPU scaling via Ray
- Latency: 10-50ms per inference (on GPU)

### 4. Community & Research
- Latest drone threat models published in Python
- Federated learning libraries (PySyft, Flower)
- Continuous model refinement from battle data
- Academic research directly applicable

### 5. Integration
- FastAPI: Native async, WebSocket support
- Kafka integration via confluent-kafka
- gRPC gateway via grpcio
- Easy integration with Rust/Go services

## Tradeoffs

### Advantages
- Fastest time to market for ML
- Access to entire research community
- Easy model updates and A/B testing
- Pre-trained models available immediately

### Disadvantages
- ~10-50ms inference latency vs <5ms for C++
- Higher CPU/GPU resources needed
- Dependency management complexity
- Cold-start for serverless

## Mitigation

### Latency
- Use GPU acceleration (latency: 5-20ms)
- Batch inference for offline workloads
- Model quantization (int8) for speed
- Cache inference results in Redis

### Dependencies
- Pin all versions in requirements.txt
- Use Poetry for deterministic builds
- Container-based deployment (no local Python)
- Security scanning for vulnerabilities

### Example: Threat Classification Pipeline

```python
from fastapi import FastAPI
import torch
from ray import serve
import mlflow

app = FastAPI()

@serve.deployment(num_replicas=4)  # 4 GPUs
class DroneClassifier:
    def __init__(self):
        self.model = torch.jit.load("models/classifier.pt")
        self.model.eval()
        
    async def classify(self, sensor_data):
        with torch.no_grad():
            logits = self.model(sensor_data)
            confidence, prediction = torch.max(logits, dim=1)
        return {
            "drone_type": prediction.item(),
            "confidence": confidence.item(),
            "threat_score": self._threat_score(prediction)
        }
    
    def _threat_score(self, drone_type):
        threat_map = {
            "quadcopter": 0.7,
            "fixed_wing": 0.9,
            "loitering_munition": 1.0
        }
        return threat_map.get(drone_type, 0.5)

# Deploy
classifier_serve = DroneClassifier.bind()
```

## Consequences

- ML pipeline can evolve rapidly as threats change
- Models updated without redeploying entire system
- Team can focus on threat detection vs infrastructure
- Federated learning enables collaborative defense

## Related Decisions

- [ADR-004: Kafka for Event Streaming](ADR-004-kafka-event-streaming.md)
- [ADR-006: PostgreSQL + TimescaleDB](ADR-006-postgres-timescaledb.md)

## References

- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [PyTorch Production Serving](https://pytorch.org/serve/)
- [Ray Serve for ML](https://docs.ray.io/en/latest/serve/index.html)
