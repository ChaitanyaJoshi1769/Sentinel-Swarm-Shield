use nalgebra::Vector3;

/// Data association: match measurements to existing tracks
/// Uses nearest neighbor with gating
pub struct DataAssociator {
    gate_threshold: f64, // Mahalanobis distance gate for association
}

impl DataAssociator {
    pub fn new() -> Self {
        Self {
            gate_threshold: 5.0, // ~99% gate for 3D measurement
        }
    }

    /// Associate measurement to nearest track
    /// Returns track index and distance
    pub fn associate(
        &self,
        measurement: &Vector3<f64>,
        track_positions: &[Vector3<f64>],
        track_covariances: &[f64],
    ) -> Option<(usize, f64)> {
        let mut best_track = None;
        let mut best_distance = f64::MAX;

        for (idx, (pos, cov)) in track_positions.iter().zip(track_covariances.iter()).enumerate()
        {
            // Euclidean distance
            let delta = measurement - pos;
            let distance = delta.norm();

            // Gate: only associate if within threshold
            let gate = self.gate_threshold * cov.sqrt();
            if distance < gate && distance < best_distance {
                best_distance = distance;
                best_track = Some(idx);
            }
        }

        best_track.map(|idx| (idx, best_distance))
    }

    /// Greedy nearest-neighbor association
    pub fn associate_multiple(
        &self,
        measurements: &[Vector3<f64>],
        track_positions: &[Vector3<f64>],
        track_covariances: &[f64],
    ) -> Vec<Option<usize>> {
        measurements
            .iter()
            .map(|meas| {
                self.associate(meas, track_positions, track_covariances)
                    .map(|(idx, _)| idx)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_association() {
        let assoc = DataAssociator::new();

        let measurement = Vector3::new(1.0, 0.0, 0.0);
        let tracks = vec![
            Vector3::new(1.1, 0.0, 0.0), // Close
            Vector3::new(10.0, 10.0, 10.0), // Far
        ];
        let covs = vec![1.0, 1.0];

        let result = assoc.associate(&measurement, &tracks, &covs);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 0); // Should match first track
    }
}
