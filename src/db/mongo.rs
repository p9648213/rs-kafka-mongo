use crate::{auth::models::User, message::models::Message};
use crate::products::models::Product;
use futures::stream::TryStreamExt;
use mongodb::{
    Client, Collection, Database,
    bson::{Document, doc, oid::ObjectId},
    options::ClientOptions,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MongoError {
    #[error("MongoDB error: {0}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("Item not found")]
    NotFound,
    #[error("Duplicate key error: {0}")]
    DuplicateKey(String),
}

#[derive(Clone)]
pub struct MongoRepo {
    db: Database,
}

impl MongoRepo {
    pub async fn init(db_url: &str, db_name: &str) -> Result<Self, MongoError> {
        let mut client_options = ClientOptions::parse(db_url).await?;
        client_options.app_name = Some("rust-api".to_string());
        let client = Client::with_options(client_options)?;
        let db = client.database(db_name);
        println!("MongoDB connected successfully.");
        Ok(Self { db })
    }

    fn users_collection(&self) -> Collection<User> {
        self.db.collection::<User>("users")
    }

    fn products_collection(&self) -> Collection<Product> {
        self.db.collection::<Product>("products")
    }

    fn message_collection(&self) -> Collection<Message> {
        self.db.collection::<Message>("messages")
    }

    pub async fn create_user(&self, new_user: User) -> Result<ObjectId, MongoError> {
        let result = self.users_collection().insert_one(new_user).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_user_by_username(&self, username: &str) -> Result<Option<User>, MongoError> {
        let filter = doc! { "username": username };
        Ok(self.users_collection().find_one(filter).await?)
    }

    pub async fn create_product(&self, new_product: Product) -> Result<ObjectId, MongoError> {
        let result = self.products_collection().insert_one(new_product).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_product_by_id(&self, id: ObjectId) -> Result<Option<Product>, MongoError> {
        let filter = doc! { "_id": id };
        Ok(self.products_collection().find_one(filter).await?)
    }

    pub async fn find_all_products(&self) -> Result<Vec<Product>, MongoError> {
        let cursor = self.products_collection().find(doc! {}).await?;
        let products: Vec<Product> = cursor.try_collect().await?;
        Ok(products)
    }

    pub async fn update_product(
        &self,
        id: ObjectId,
        update_doc: Document,
    ) -> Result<bool, MongoError> {
        let filter = doc! { "_id": id };
        let update = doc! { "$set": update_doc };
        let result = self
            .products_collection()
            .update_one(filter, update)
            .await?;
        Ok(result.matched_count > 0)
    }

    pub async fn delete_product(&self, id: ObjectId) -> Result<bool, MongoError> {
        let filter = doc! { "_id": id };
        let result = self.products_collection().delete_one(filter).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn find_all_message(&self) -> Result<Vec<Message>, MongoError> {
        let cursor = self.message_collection().find(doc! {}).await?;
        let messages: Vec<Message> = cursor.try_collect().await?;
        Ok(messages)
    }

    pub async fn create_message(&self, new_message: Message) -> Result<ObjectId, MongoError> {
        let result = self.message_collection().insert_one(new_message).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }
}
