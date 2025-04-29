use bson::oid::ObjectId;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rs_kafka_mongo::config::Config;
use rs_kafka_mongo::db::mongo::MongoRepo;
use rs_kafka_mongo::message::models::Message as EventMessage;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    let db_repo = MongoRepo::init(&config.database_url, &config.database_name).await?;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "product-event-listener")
        .set("bootstrap.servers", "kafka:29092")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["product_events"])
        .expect("Can't subscribe to specified topic");

    println!("Listening to Kafka topic 'product-events'...");

    let mut message_stream = consumer.stream();
    
    while let Some(message_result) = message_stream.next().await {
        match message_result {
            Ok(message) => {
                if let Ok(payload) = message.payload_view::<str>().unwrap() {
                    println!("Received: {}", payload);
                    let new_message = EventMessage {
                        _id: Some(ObjectId::new()),
                        message: payload.to_string()
                    };
                    db_repo.create_message(new_message).await?;
                }
            }
            Err(e) => eprintln!("Error while reading from stream: {}", e),
        }
    }

    Ok(())
}