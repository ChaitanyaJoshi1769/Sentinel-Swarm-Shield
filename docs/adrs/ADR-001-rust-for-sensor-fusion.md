# ADR-001: Use Rust for Sensor Fusion Engine

**Status**: Accepted  
**Date**: 2026-05-21  
**Deciders**: Architecture Team

## Context

The sensor fusion engine is the critical path for real-time threat detection. It must:
- Process multi-source sensor data (radar, RF, EO/IR, acoustic)
- Perform real-time Kalman filtering and data association
- Maintain <100ms latency from sensor input to track output
- Handle high-frequency data streams (1000+ measurements/sec)
- Run on both cloud and edge hardware with predictable performance

**Candidates**:
1. **Rust**: Low-latency, memory-safe, concurrent
2. **C++**: Fast, but memory safety issues
3. **Go**: Good concurrency, but GC pauses
4. **Python**: Rich libraries, but too slow for real-time

## Decision

Use **Rust** for the sensor fusion engine.

**Stack**:
- **Runtime**: Tokio async
- **Numerics**: ndarray for matrix operations
- **gRPC**: tonic + prost
- **Database**: sqlx for async PostgreSQL
- **Build**: Cargo

## Rationale

### 1. Latency Guarantees
- No garbage collector (Rust has compile-time memory management)
- No unpredictable GC pauses
- ~1ns allocation overhead (vs Go's GC pauses: 1-50ms)
- Suitable for hard real-time constraints

### 2. Memory Safety
- Prevents buffer overflows, use-after-free
- Borrow checker catches data races at compile time
- Critical for defense systems where failures can be catastrophic

### 3. Concurrency
- Tokio async runtime handles 100K+ concurrent tasks
- Zero-copy message passing
- Lock-free data structures via crossbeam

### 4. Performance
- ndarray performs near-C speeds for numerical computation
- SIMD vectorization support
- Kalman filtering: <50µs per track update

### 5. Deployment
- Compiles to standalone binary (no runtime dependency)
- Cross-platform compilation (x86, ARM, RISC-V)
- Suitable for edge devices (K3s nodes)

## Tradeoffs

### Advantages
- Sub-100ms latency guaranteed
- Type-safe, compile-time verification
- Minimal production issues (memory leaks impossible)
- Excellent for teams familiar with systems programming

### Disadvantages
- Longer development time vs Python/Go
- Smaller ecosystem than Python for ML
- Steeper learning curve for non-systems engineers
- Slower iteration than Python

## Mitigation

- **Development**: Use rapid prototyping in Python for algorithms, then port to Rust
- **Learning**: Allocate 2-3 weeks for team Rust onboarding
- **Library**: Maintain common Rust libraries for Kalman filtering, data association
- **Testing**: Comprehensive unit + integration tests to catch issues early

## Example: Kalman Filter in Rust

```rust
use ndarray::Array1;

pub struct KalmanFilter {
    x: Array1<f64>,     // State vector
    P: Array2<f64>,     // Covariance
    F: Array2<f64>,     // State transition
    H: Array2<f64>,     // Measurement matrix
    R: Array2<f64>,     // Measurement noise
    Q: Array2<f64>,     // Process noise
}

impl KalmanFilter {
    pub fn update(&mut self, z: &Array1<f64>) -> Track {
        // Predict
        let x_pred = self.F.dot(&self.x);
        let P_pred = self.F.dot(&self.P).dot(&self.F.t()) + &self.Q;
        
        // Update
        let y = z - self.H.dot(&x_pred);  // Innovation
        let S = self.H.dot(&P_pred).dot(&self.H.t()) + &self.R;
        let K = P_pred.dot(&self.H.t()).dot(&S.inv());  // Kalman gain
        
        self.x = x_pred + K.dot(&y);
        self.P = (Array2::eye(4) - K.dot(&self.H)).dot(&P_pred);
        
        Track::from(&self.x)
    }
}
```

## Consequences

- Sensor fusion engine will be production-grade, reliable
- Lower support burden post-deployment
- Team must invest in Rust skills upfront
- Easier to scale to high-frequency sensor data

## Related Decisions

- [ADR-002: Python for ML Services](ADR-002-python-for-ml.md)
- [ADR-003: Go for Orchestration](ADR-003-go-for-orchestration.md)

## References

- [Tokio Documentation](https://tokio.rs/)
- [ndarray Guide](https://docs.rs/ndarray/)
- "Real-Time Collision Detection" - C. Akenine-Möller et al.
