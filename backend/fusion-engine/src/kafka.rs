use crate::fusion::track_manager::TrackData;
use rdkafka::producer::FutureProducer;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
pub struct Producer {
    producer: Arc<FutureProducer>,
}

impl Producer {
    pub fn producer(&self) -> &FutureProducer {
        &self.producer
    }
}

pub async fn init(_config: &crate::config::Config) -> anyhow::Result<Arc<Producer>> {
    // In production, configure Kafka brokers from config
    let producer: FutureProducer = rdkafka::client::DefaultClientConfig::new()
        .create()?;

    Ok(Arc::new(Producer {
        producer: Arc::new(producer),
    }))
}

impl Producer {
    pub async fn publish_track_new(&self, track_id: &str) -> anyhow::Result<()> {
        let event = json!({
            "track_id": track_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.publish("tracks.new", track_id, &event.to_string())
            .await
    }

    pub async fn publish_track_updated(&self, track: &TrackData) -> anyhow::Result<()> {
        let event = json!({
            "track_id": track.id,
            "position": {
                "x": track.position[0],
                "y": track.position[1],
                "z": track.position[2],
            },
            "velocity": {
                "vx": track.velocity[0],
                "vy": track.velocity[1],
                "vz": track.velocity[2],
            },
            "confidence": track.confidence,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        self.publish("tracks.updated", &track.id, &event.to_string())
            .await
    }

    async fn publish(&self, topic: &str, key: &str, value: &str) -> anyhow::Result<()> {
        use rdkafka::message::Message;

        let record = rdkafka::message::Record::to_topic(topic)
            .key(key)
            .payload(value);

        let _result = self.producer.send(record, std::time::Duration::from_secs(5)).await;

        Ok(())
    }
}
