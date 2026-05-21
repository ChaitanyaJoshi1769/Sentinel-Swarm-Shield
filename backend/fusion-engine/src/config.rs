use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub postgres_url: String,
    pub kafka_brokers: String,
    pub grpc_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            postgres_url: env::var("POSTGRES_URL")
                .unwrap_or_else(|_| "postgresql://sentinel:sentinel_dev_password@localhost:5432/sentinel".to_string()),
            kafka_brokers: env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "localhost:9092".to_string()),
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()
                .unwrap_or(50051),
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
        })
    }
}
