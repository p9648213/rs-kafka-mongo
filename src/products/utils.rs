use crate::kafka::producer::{AppKafkaProducer, ProductEvent};

use super::models::{Product, ProductResponse};

pub async fn send_kafka_event<T: serde::Serialize + std::fmt::Debug + Clone + Send + 'static>(
  kafka_producer: &AppKafkaProducer,
  topic: &str,
  event: ProductEvent<T>,
) {
  let producer = kafka_producer.clone();
  let event_clone = event.clone();
  let topic = topic.to_string();

  tokio::spawn(async move {
      match producer.send_product_event(&topic, event_clone).await {
          Ok(_) => tracing::info!("Successfully sent Kafka event: {:?}", event.event_type),
          Err(e) => tracing::error!("Failed to send Kafka event {:?}: {:?}", event.event_type, e),
      }
  });
}

pub fn products_to_responses(products: &[Product]) -> Vec<ProductResponse> {
  products.iter().map(ProductResponse::from_product).collect()
}