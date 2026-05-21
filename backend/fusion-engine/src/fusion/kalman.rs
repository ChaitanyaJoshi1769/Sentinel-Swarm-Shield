use nalgebra::{Matrix3, Matrix6, Vector3, Vector6};
use std::time::{Duration, SystemTime};

/// Kalman filter for 3D constant-velocity tracking
/// State vector: [x, y, z, vx, vy, vz]
pub struct KalmanFilter {
    // State
    x: Vector6<f64>, // Position and velocity [x, y, z, vx, vy, vz]
    P: Matrix6<f64>, // Covariance matrix

    // System matrices
    F: Matrix6<f64>, // State transition (constant-velocity model)
    H: Matrix6<f64>, // Measurement matrix (measure position only)

    // Noise covariances
    Q: Matrix6<f64>, // Process noise covariance
    R: Matrix3<f64>, // Measurement noise covariance

    last_update: SystemTime,
}

impl KalmanFilter {
    /// Create a new Kalman filter initialized at position with zero velocity
    pub fn new(initial_position: Vector3<f64>) -> Self {
        // Initial state: position at measurement, zero velocity
        let x = Vector6::new(
            initial_position[0],
            initial_position[1],
            initial_position[2],
            0.0,
            0.0,
            0.0,
        );

        // Initial covariance: high uncertainty on velocity, low on position
        let mut P = Matrix6::identity();
        P[(0, 0)] = 1.0;   // x
        P[(1, 1)] = 1.0;   // y
        P[(2, 2)] = 1.0;   // z
        P[(3, 3)] = 10.0;  // vx (high uncertainty)
        P[(4, 4)] = 10.0;  // vy
        P[(5, 5)] = 10.0;  // vz

        // Constant-velocity state transition matrix
        // x_k+1 = x_k + dt * vx_k
        // Will be updated with actual dt in predict()
        let mut F = Matrix6::identity();
        F[(0, 3)] = 1.0; // x += vx * dt (dt=1 for now)
        F[(1, 4)] = 1.0; // y += vy * dt
        F[(2, 5)] = 1.0; // z += vz * dt

        // Measurement matrix: measure position only (first 3 states)
        let H = Matrix6::zeros();
        // Note: we'll use position-only measurements, so H will project to first 3 elements

        // Process noise: small, assumes constant velocity
        let mut Q = Matrix6::zeros();
        let sigma_q = 0.1; // m/s^2 acceleration noise
        Q[(0, 0)] = sigma_q;
        Q[(1, 1)] = sigma_q;
        Q[(2, 2)] = sigma_q;
        Q[(3, 3)] = sigma_q;
        Q[(4, 4)] = sigma_q;
        Q[(5, 5)] = sigma_q;

        // Measurement noise: position measurements have ~1m std dev
        let mut R = Matrix3::zeros();
        let sigma_r = 1.0; // meters
        R[(0, 0)] = sigma_r * sigma_r;
        R[(1, 1)] = sigma_r * sigma_r;
        R[(2, 2)] = sigma_r * sigma_r;

        Self {
            x,
            P,
            F,
            H,
            Q,
            R,
            last_update: SystemTime::now(),
        }
    }

    /// Predict next state (time update)
    pub fn predict(&mut self, dt: f64) {
        // Update state transition matrix with actual dt
        let mut F = Matrix6::identity();
        F[(0, 3)] = dt;
        F[(1, 4)] = dt;
        F[(2, 5)] = dt;

        // x_pred = F * x
        self.x = F * &self.x;

        // P_pred = F * P * F^T + Q
        self.P = &F * &self.P * F.transpose() + &self.Q;

        self.last_update = SystemTime::now();
    }

    /// Update state with measurement (measurement update)
    pub fn update(&mut self, measurement: &Vector3<f64>) -> f64 {
        // Measurement matrix: we only measure position (first 3 elements)
        let H = {
            let mut h = Matrix6::zeros();
            h[(0, 0)] = 1.0;
            h[(1, 1)] = 1.0;
            h[(2, 2)] = 1.0;
            h
        };

        // Predicted measurement
        let z_pred = Vector3::new(self.x[0], self.x[1], self.x[2]);

        // Innovation (measurement residual)
        let y = measurement - &z_pred;
        let innovation_norm = y.norm();

        // Innovation covariance: S = H * P * H^T + R
        let S = H.clone() * &self.P * H.transpose() + &self.R;

        // Kalman gain: K = P * H^T * S^-1
        // For efficiency, we use a 3x6 submatrix since we only have 3 measurements
        let H_3x6 = {
            let mut h = nalgebra::Matrix3x6::zeros();
            h[(0, 0)] = 1.0;
            h[(1, 1)] = 1.0;
            h[(2, 2)] = 1.0;
            h
        };

        // Simplified Kalman gain for position-only measurements
        let P_3x3 = {
            let mut p = Matrix3::zeros();
            for i in 0..3 {
                for j in 0..3 {
                    p[(i, j)] = self.P[(i, j)];
                }
            }
            p
        };

        let S_3x3 = &P_3x3 * 1.0 + &self.R;
        let S_inv = S_3x3
            .try_inverse()
            .unwrap_or_else(|| Matrix3::identity() * 0.1);

        let K_3x3 = &P_3x3 * S_inv.clone();

        // Update state: x = x + K * y
        for i in 0..3 {
            for j in 0..3 {
                self.x[i] += K_3x3[(i, j)] * y[j];
            }
        }

        // Update covariance: P = (I - K * H) * P
        let mut I_KH = Matrix6::identity();
        for i in 0..3 {
            for j in 0..6 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += K_3x3[(i, k)] * H[(k, j)];
                }
                I_KH[(i, j)] -= sum;
            }
        }

        self.P = &I_KH * &self.P;

        innovation_norm
    }

    /// Get current position estimate
    pub fn position(&self) -> Vector3<f64> {
        Vector3::new(self.x[0], self.x[1], self.x[2])
    }

    /// Get current velocity estimate
    pub fn velocity(&self) -> Vector3<f64> {
        Vector3::new(self.x[3], self.x[4], self.x[5])
    }

    /// Get position covariance (first 3x3 block of P)
    pub fn position_covariance(&self) -> f64 {
        (self.P[(0, 0)] + self.P[(1, 1)] + self.P[(2, 2)]) / 3.0
    }

    /// Get state vector [x, y, z, vx, vy, vz]
    pub fn state(&self) -> Vector6<f64> {
        self.x.clone()
    }

    /// Get confidence (inverse of covariance)
    pub fn confidence(&self) -> f64 {
        let pos_cov = self.position_covariance();
        1.0 / (1.0 + pos_cov)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kalman_constant_velocity() {
        let mut kf = KalmanFilter::new(Vector3::new(0.0, 0.0, 0.0));

        // Predict 1 second with unknown velocity
        kf.predict(1.0);

        // Should still be at origin (zero velocity)
        let pos = kf.position();
        assert!((pos[0]).abs() < 0.1);
        assert!((pos[1]).abs() < 0.1);
        assert!((pos[2]).abs() < 0.1);
    }

    #[test]
    fn test_kalman_velocity_estimation() {
        let mut kf = KalmanFilter::new(Vector3::new(0.0, 0.0, 0.0));

        // Simulate moving object: update with position measurements
        for i in 1..=10 {
            kf.predict(1.0);
            let measurement = Vector3::new(i as f64, 0.0, 0.0);
            kf.update(&measurement);
        }

        let velocity = kf.velocity();
        // Should estimate velocity ~1.0 m/s in x direction
        assert!((velocity[0] - 1.0).abs() < 0.2);
        assert!(velocity[1].abs() < 0.1);
    }
}
