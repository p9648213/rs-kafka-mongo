use crate::config::Config;
use crate::db::mongo::MongoRepo;
use crate::kafka::producer::AppKafkaProducer;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_repo: MongoRepo,
    pub kafka_producer: AppKafkaProducer,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let db_repo = MongoRepo::init(&config.database_url, &config.database_name).await?;
        let kafka_producer = AppKafkaProducer::new(&config.kafka_brokers)?;

        Ok(Self {
            config,
            db_repo,
            kafka_producer,
        })
    }
}