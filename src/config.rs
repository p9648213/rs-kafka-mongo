use dotenvy::dotenv;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_addr: String,
    pub database_url: String,
    pub database_name: String,
    pub kafka_brokers: String,
    pub kafka_product_events_topic: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv().ok();

        Ok(Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8000".to_string()),
            database_url: env::var("DATABASE_URL")?,
            database_name: env::var("DATABASE_NAME")?,
            kafka_brokers: env::var("KAFKA_BROKERS")?,
            kafka_product_events_topic: env::var("KAFKA_PRODUCT_EVENTS_TOPIC")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a number"),
        })
    }
}