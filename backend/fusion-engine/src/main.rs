mod fusion;
mod sensor;
mod track;
mod kafka;
mod database;
mod config;

use anyhow::Result;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing_subscriber::EnvFilter;

#[path = "generated/sentinel.fusion.rs"]
pub mod fusion_pb;

#[path = "generated/sentinel.sensor.rs"]
pub mod sensor_pb;

use fusion_pb::fusion_engine_server::FusionEngineServer;
use sensor_pb::sensor_provider_server::SensorProviderServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Sentinel Fusion Engine starting...");

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize database connection pool
    let db_pool = database::init(&config).await?;
    tracing::info!("Database initialized");

    // Initialize Kafka producer
    let kafka_producer = kafka::init(&config).await?;
    tracing::info!("Kafka producer initialized");

    // Initialize fusion engine
    let fusion_engine = fusion::FusionEngineImpl::new(db_pool.clone(), kafka_producer.clone());
    tracing::info!("Fusion engine initialized");

    // Initialize sensor provider
    let sensor_provider = sensor::SensorProviderImpl::new(
        fusion_engine.clone(),
        kafka_producer.clone(),
    );
    tracing::info!("Sensor provider initialized");

    // Bind to address
    let addr: SocketAddr = format!("0.0.0.0:50051").parse()?;
    tracing::info!("Binding to {}", addr);

    // Build gRPC server
    let server = Server::builder()
        .add_service(FusionEngineServer::new(fusion_engine))
        .add_service(SensorProviderServer::new(sensor_provider))
        .serve(addr);

    tracing::info!("Fusion Engine listening on {}", addr);

    // Run server
    server.await?;

    Ok(())
}
