pub mod kalman;
pub mod clustering;
pub mod data_assoc;
pub mod track_manager;

use crate::database::Database;
use crate::kafka::Producer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::fusion_pb::fusion_engine_server::FusionEngine;
use crate::fusion_pb::*;

pub use track_manager::TrackManager;

/// FusionEngineImpl handles multi-sensor fusion and track management
#[derive(Clone)]
pub struct FusionEngineImpl {
    db: Arc<Database>,
    kafka: Arc<Producer>,
    tracks: Arc<RwLock<TrackManager>>,
}

impl FusionEngineImpl {
    pub fn new(db: Arc<Database>, kafka: Arc<Producer>) -> Self {
        Self {
            db,
            kafka,
            tracks: Arc::new(RwLock::new(TrackManager::new())),
        }
    }

    /// Process incoming sensor measurement and update tracks
    pub async fn process_measurement(&self, measurement: SensorMeasurement) -> anyhow::Result<()> {
        let mut tracks = self.tracks.write().await;

        // Parse measurement based on type
        match &measurement.data {
            Some(data) => {
                match data {
                    sensor_pb::SensorMeasurementData::Radar(radar) => {
                        self.process_radar(radar, &measurement, &mut tracks).await?;
                    }
                    sensor_pb::SensorMeasurementData::Rf(rf) => {
                        self.process_rf(rf, &measurement, &mut tracks).await?;
                    }
                    _ => {
                        tracing::warn!("Unhandled measurement type");
                    }
                }
            }
            None => {
                return Err(anyhow::anyhow!("Empty measurement data"));
            }
        }

        Ok(())
    }

    async fn process_radar(
        &self,
        radar: &crate::sensor_pb::RadarReturn,
        measurement: &SensorMeasurement,
        tracks: &mut TrackManager,
    ) -> anyhow::Result<()> {
        // Convert radar measurement to Cartesian coordinates
        let x = radar.range_m * radar.azimuth_deg.to_radians().cos();
        let y = radar.range_m * radar.azimuth_deg.to_radians().sin();
        let z = radar.range_m * radar.elevation_deg.to_radians().sin();

        let measurement_vector = nalgebra::Vector3::new(x, y, z);

        // Update tracks via Kalman filter and data association
        let (matched_track_id, innovation) = tracks.associate_measurement(&measurement_vector);

        if let Some(track_id) = matched_track_id {
            // Update existing track
            tracks.update_track(track_id, measurement_vector)?;

            // Publish track update to Kafka
            self.kafka
                .publish_track_updated(&tracks.get_track(track_id).unwrap())
                .await?;
        } else {
            // Create new track
            let track_id = Uuid::new_v4().to_string();
            tracks.create_track(track_id.clone(), measurement_vector)?;

            // Publish new track event
            self.kafka.publish_track_new(&track_id).await?;
        }

        Ok(())
    }

    async fn process_rf(
        &self,
        rf: &crate::sensor_pb::RFSignal,
        measurement: &SensorMeasurement,
        tracks: &mut TrackManager,
    ) -> anyhow::Result<()> {
        tracing::debug!("Processing RF signal: {:?}", rf);
        // RF classification and track augmentation
        Ok(())
    }
}

#[tonic::async_trait]
impl FusionEngine for FusionEngineImpl {
    async fn list_tracks(
        &self,
        request: Request<ListTracksRequest>,
    ) -> Result<Response<ListTracksResponse>, Status> {
        let req = request.into_inner();
        let tracks = self.tracks.read().await;

        let limit = req.limit as usize;
        let offset = req.offset as usize;
        let track_list = tracks.list_tracks(limit, offset);

        let response = ListTracksResponse {
            tracks: track_list.iter().map(|t| t.to_proto()).collect(),
            total_count: tracks.count() as i32,
        };

        Ok(Response::new(response))
    }

    async fn get_track(
        &self,
        request: Request<GetTrackRequest>,
    ) -> Result<Response<Track>, Status> {
        let req = request.into_inner();
        let tracks = self.tracks.read().await;

        let track = tracks
            .get_track(&req.track_id)
            .ok_or_else(|| Status::not_found("Track not found"))?;

        Ok(Response::new(track.to_proto()))
    }

    async fn list_swarms(
        &self,
        request: Request<ListSwarmsRequest>,
    ) -> Result<Response<ListSwarmsResponse>, Status> {
        let req = request.into_inner();
        let tracks = self.tracks.read().await;

        let limit = req.limit as usize;
        let offset = req.offset as usize;
        let swarm_list = tracks.list_swarms(limit, offset);

        let response = ListSwarmsResponse {
            swarms: swarm_list.iter().map(|s| s.to_proto()).collect(),
            total_count: swarm_list.len() as i32,
        };

        Ok(Response::new(response))
    }

    async fn get_swarm(
        &self,
        request: Request<GetSwarmRequest>,
    ) -> Result<Response<Swarm>, Status> {
        let req = request.into_inner();
        let tracks = self.tracks.read().await;

        let swarm = tracks
            .get_swarm(&req.swarm_id)
            .ok_or_else(|| Status::not_found("Swarm not found"))?;

        Ok(Response::new(swarm.to_proto()))
    }

    async fn get_status(
        &self,
        _request: Request<prost_types::Empty>,
    ) -> Result<Response<FusionStatus>, Status> {
        let tracks = self.tracks.read().await;

        let response = FusionStatus {
            is_healthy: true,
            started_at: Some(prost_types::Timestamp::now()),
            active_tracks: tracks.count() as i32,
            active_swarms: tracks.swarm_count() as i32,
            total_tracks_lifetime: 0,
            measurements_processed: 0,
            sensor_health: vec![],
            average_latency_ms: 5.0,
        };

        Ok(Response::new(response))
    }
}
