use nalgebra::Vector3;
use std::collections::HashSet;

/// DBSCAN clustering for swarm detection
pub struct SwarmDetector {
    epsilon: f64,  // Maximum distance between points in a cluster
    min_points: usize, // Minimum points to form a cluster
}

impl SwarmDetector {
    pub fn new(epsilon: f64, min_points: usize) -> Self {
        Self { epsilon, min_points }
    }

    /// Detect swarms (clusters) from track positions
    pub fn detect(&self, positions: &[(String, Vector3<f64>)]) -> Vec<Swarm> {
        let mut visited = HashSet::new();
        let mut clusters = Vec::new();

        for (idx, (id, pos)) in positions.iter().enumerate() {
            if visited.contains(&idx) {
                continue;
            }

            // Find neighbors (epsilon-neighborhood)
            let neighbors = self.find_neighbors(idx, positions);

            if neighbors.len() >= self.min_points {
                // Start a new cluster
                let mut cluster = vec![id.clone()];
                visited.insert(idx);

                // Expand cluster (simple DBSCAN)
                let mut queue = neighbors.clone();
                while let Some(neighbor_idx) = queue.pop() {
                    if visited.contains(&neighbor_idx) {
                        continue;
                    }

                    visited.insert(neighbor_idx);
                    let (neighbor_id, _) = &positions[neighbor_idx];
                    cluster.push(neighbor_id.clone());

                    // Check if neighbor is also a core point
                    let neighbor_neighbors = self.find_neighbors(neighbor_idx, positions);
                    if neighbor_neighbors.len() >= self.min_points {
                        queue.extend(neighbor_neighbors);
                    }
                }

                if cluster.len() >= self.min_points {
                    clusters.push(Swarm {
                        track_ids: cluster,
                    });
                }
            }
        }

        clusters
    }

    fn find_neighbors(&self, idx: usize, positions: &[(String, Vector3<f64>)]) -> Vec<usize> {
        let (_, center_pos) = &positions[idx];
        positions
            .iter()
            .enumerate()
            .filter_map(|(i, (_, pos))| {
                if i == idx {
                    return None;
                }
                let distance = (center_pos - pos).norm();
                if distance <= self.epsilon {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Swarm {
    pub track_ids: Vec<String>,
}

impl Swarm {
    pub fn centroid(&self, track_positions: &std::collections::HashMap<String, Vector3<f64>>) -> Vector3<f64> {
        let positions: Vec<_> = self
            .track_ids
            .iter()
            .filter_map(|id| track_positions.get(id))
            .collect();

        if positions.is_empty() {
            return Vector3::zeros();
        }

        let sum: Vector3<f64> = positions.iter().map(|p| **p).sum();
        sum / positions.len() as f64
    }

    pub fn cohesion(&self, track_positions: &std::collections::HashMap<String, Vector3<f64>>) -> f64 {
        let centroid = self.centroid(track_positions);
        let positions: Vec<_> = self
            .track_ids
            .iter()
            .filter_map(|id| track_positions.get(id))
            .collect();

        if positions.is_empty() {
            return 0.0;
        }

        let avg_distance: f64 = positions
            .iter()
            .map(|p| (centroid - **p).norm())
            .sum::<f64>()
            / positions.len() as f64;

        // Cohesion: 1/(1 + avg_distance)
        1.0 / (1.0 + avg_distance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_detection() {
        let detector = SwarmDetector::new(5.0, 3);

        let positions = vec![
            ("track1".to_string(), Vector3::new(0.0, 0.0, 0.0)),
            ("track2".to_string(), Vector3::new(1.0, 1.0, 1.0)),
            ("track3".to_string(), Vector3::new(2.0, 2.0, 2.0)),
            ("track4".to_string(), Vector3::new(100.0, 100.0, 100.0)),
        ];

        let swarms = detector.detect(&positions);
        assert_eq!(swarms.len(), 1);
        assert_eq!(swarms[0].track_ids.len(), 3);
    }
}
