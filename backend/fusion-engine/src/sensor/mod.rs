use crate::fusion::FusionEngineImpl;
use crate::kafka::Producer;
use tonic::{Request, Response, Status};

use crate::sensor_pb::sensor_provider_server::SensorProvider;
use crate::sensor_pb::*;

#[derive(Clone)]
pub struct SensorProviderImpl {
    fusion: FusionEngineImpl,
    kafka: std::sync::Arc<Producer>,
}

impl SensorProviderImpl {
    pub fn new(fusion: FusionEngineImpl, kafka: std::sync::Arc<Producer>) -> Self {
        Self { fusion, kafka }
    }
}

#[tonic::async_trait]
impl SensorProvider for SensorProviderImpl {
    type StreamSensorDataStream =
        Box<dyn tokio_stream::Stream<Item = Result<SensorMeasurement, Status>> + Send>;

    async fn stream_sensor_data(
        &self,
        request: Request<SensorStreamRequest>,
    ) -> Result<Response<Self::StreamSensorDataStream>, Status> {
        let _req = request.into_inner();

        // For now, return empty stream
        // In production, this would subscribe to sensor data streams
        let stream = tokio_stream::iter(vec![]);
        Ok(Response::new(Box::new(stream)))
    }

    async fn get_sensor_status(
        &self,
        _request: Request<prost_types::Empty>,
    ) -> Result<Response<SensorStatus>, Status> {
        let status = SensorStatus {
            sensor_id: "sensor-1".to_string(),
            is_operational: true,
            last_measurement: Some(prost_types::Timestamp::now()),
            measurements_total: 0,
            measurements_per_sec: 100,
            health_percentage: 100.0,
            last_error: "".to_string(),
        };

        Ok(Response::new(status))
    }

    async fn configure_sensor(
        &self,
        _request: Request<SensorConfig>,
    ) -> Result<Response<prost_types::Empty>, Status> {
        Ok(Response::new(prost_types::Empty {}))
    }
}
