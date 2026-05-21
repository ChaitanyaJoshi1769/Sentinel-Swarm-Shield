use super::kalman::KalmanFilter;
use super::clustering::{SwarmDetector, Swarm};
use super::data_assoc::DataAssociator;
use nalgebra::Vector3;
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct TrackData {
    pub id: String,
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub confidence: f64,
    pub age_seconds: u32,
    pub sensor_ids: Vec<String>,
    pub swarm_id: Option<String>,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub measurement_count: u32,
}

impl TrackData {
    pub fn to_proto(&self) -> crate::fusion_pb::Track {
        crate::fusion_pb::Track {
            id: self.id.clone(),
            timestamp: Some(prost_types::Timestamp::from_system_time(self.updated_at)),
            latitude: self.position[1], // Simplified: use Y as latitude
            longitude: self.position[0],
            altitude_m: self.position[2],
            velocity: Some(crate::fusion_pb::track::Velocity {
                north_mps: self.velocity[1],
                east_mps: self.velocity[0],
                down_mps: -self.velocity[2],
            }),
            confidence: self.confidence as f32,
            sensor_ids: self.sensor_ids.clone(),
            swarm_id: self.swarm_id.clone().unwrap_or_default(),
            drone_type: 0, // Will be set by AI service
            classification_confidence: 0.0,
            created_at: Some(prost_types::Timestamp::from_system_time(self.created_at)),
            updated_at: Some(prost_types::Timestamp::from_system_time(self.updated_at)),
            age_seconds: self.age_seconds as i32,
        }
    }
}

/// Track manager coordinates Kalman filtering, data association, and swarm clustering
pub struct TrackManager {
    tracks: HashMap<String, (TrackData, KalmanFilter)>,
    data_assoc: DataAssociator,
    swarm_detector: SwarmDetector,
    swarms: HashMap<String, Swarm>,
    max_track_age: u32, // seconds
}

impl TrackManager {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
            data_assoc: DataAssociator::new(),
            swarm_detector: SwarmDetector::new(50.0, 3), // 50m clusters, min 3 drones
            swarms: HashMap::new(),
            max_track_age: 30, // 30 seconds
        }
    }

    pub fn create_track(&mut self, id: String, position: Vector3<f64>) -> anyhow::Result<()> {
        let kf = KalmanFilter::new(position);
        let track = TrackData {
            id: id.clone(),
            position,
            velocity: Vector3::zeros(),
            confidence: kf.confidence(),
            age_seconds: 0,
            sensor_ids: vec![],
            swarm_id: None,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            measurement_count: 1,
        };

        self.tracks.insert(id, (track, kf));
        self.update_swarms();

        Ok(())
    }

    pub fn update_track(&mut self, id: &str, measurement: Vector3<f64>) -> anyhow::Result<()> {
        if let Some((track, kf)) = self.tracks.get_mut(id) {
            // Predict (assume 100ms between measurements)
            kf.predict(0.1);

            // Update with measurement
            let innovation = kf.update(&measurement);

            // Update track data
            track.position = kf.position();
            track.velocity = kf.velocity();
            track.confidence = kf.confidence();
            track.updated_at = SystemTime::now();
            track.measurement_count += 1;

            // Prune old tracks
            self.prune_old_tracks();

            // Update swarm assignments
            self.update_swarms();

            Ok(())
        } else {
            Err(anyhow::anyhow!("Track not found: {}", id))
        }
    }

    pub fn associate_measurement(&self, measurement: &Vector3<f64>) -> (Option<String>, f64) {
        let track_ids: Vec<_> = self.tracks.keys().cloned().collect();
        let positions: Vec<_> = self
            .tracks
            .values()
            .map(|(t, _)| t.position)
            .collect();
        let covariances: Vec<_> = self
            .tracks
            .values()
            .map(|(t, _)| t.confidence)
            .collect();

        match self.data_assoc.associate(measurement, &positions, &covariances) {
            Some((idx, distance)) => (Some(track_ids[idx].clone()), distance),
            None => (None, f64::MAX),
        }
    }

    pub fn get_track(&self, id: &str) -> Option<&TrackData> {
        self.tracks.get(id).map(|(t, _)| t)
    }

    pub fn get_swarm(&self, id: &str) -> Option<&Swarm> {
        self.swarms.get(id)
    }

    pub fn list_tracks(&self, limit: usize, offset: usize) -> Vec<TrackData> {
        self.tracks
            .values()
            .map(|(t, _)| t.clone())
            .skip(offset)
            .take(limit)
            .collect()
    }

    pub fn list_swarms(&self, limit: usize, offset: usize) -> Vec<Swarm> {
        self.swarms
            .values()
            .cloned()
            .skip(offset)
            .take(limit)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.tracks.len()
    }

    pub fn swarm_count(&self) -> usize {
        self.swarms.len()
    }

    fn update_swarms(&mut self) {
        // Re-cluster tracks into swarms
        let positions: Vec<_> = self
            .tracks
            .iter()
            .map(|(id, (t, _))| (id.clone(), t.position))
            .collect();

        let new_swarms = self.swarm_detector.detect(&positions);

        // Clear old swarms and assign new ones
        self.swarms.clear();
        for (i, swarm) in new_swarms.iter().enumerate() {
            let swarm_id = format!("swarm-{}", i);
            for track_id in &swarm.track_ids {
                if let Some((track, _)) = self.tracks.get_mut(track_id) {
                    track.swarm_id = Some(swarm_id.clone());
                }
            }
            self.swarms.insert(swarm_id, swarm.clone());
        }
    }

    fn prune_old_tracks(&mut self) {
        let now = SystemTime::now();
        self.tracks.retain(|_, (track, _)| {
            let age = now
                .duration_since(track.updated_at)
                .map(|d| d.as_secs() as u32)
                .unwrap_or(0);

            age < self.max_track_age
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_creation() {
        let mut manager = TrackManager::new();
        let id = "test-track-1".to_string();
        let pos = Vector3::new(10.0, 20.0, 30.0);

        manager.create_track(id.clone(), pos).unwrap();
        assert_eq!(manager.count(), 1);

        let track = manager.get_track(&id).unwrap();
        assert_eq!(track.position, pos);
    }

    #[test]
    fn test_swarm_detection() {
        let mut manager = TrackManager::new();

        // Create 5 tracks in close formation
        for i in 0..5 {
            let pos = Vector3::new(i as f64, i as f64, i as f64);
            manager
                .create_track(format!("track-{}", i), pos)
                .unwrap();
        }

        assert!(manager.swarm_count() > 0);
    }
}
