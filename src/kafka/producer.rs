use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("Kafka producer error: {0}")]
    ProducerError(#[from] rdkafka::error::KafkaError),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Kafka message delivery timed out")]
    DeliveryTimeout,
}

#[derive(Serialize, Debug, Clone)]
pub enum ProductEventType {
    Created,
    Updated,
    Deleted,
}

#[derive(Serialize, Debug, Clone)]
pub struct ProductEvent<T> {
    pub event_type: ProductEventType,
    pub product_id: String,
    pub payload: Option<T>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct AppKafkaProducer {
    pub producer: FutureProducer,
    pub topic: String,
}

impl AppKafkaProducer {
    pub fn new(brokers: &str) -> Result<Self, KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()?;
        println!("Kafka producer created successfully.");
        Ok(Self {
            producer,
            topic: String::new(),
        })
    }

    pub async fn send_product_event<T: Serialize>(
        &self,
        topic: &str,
        event: ProductEvent<T>,
    ) -> Result<(), KafkaError> {
        let payload = serde_json::to_string(&event)?;
        let key = event.product_id.clone();

        let record = FutureRecord::to(topic)
            .payload(&payload)
            .key(&key);

        match self.producer.send(record, Timeout::After(Duration::from_secs(5))).await {
            Ok(_) => {
                tracing::debug!("Kafka message sent successfully to topic '{}'", topic);
                Ok(())
            },
            Err((kafka_err, _owned_message)) => {
                tracing::error!("Failed to enqueue message in Kafka: {}", kafka_err);
                Err(KafkaError::ProducerError(kafka_err))
            }
        }
    }
}