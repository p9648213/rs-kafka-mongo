use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub name: String,
    pub description: String,
    pub price: f64,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateProductRequest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub price: f64,
}

#[derive(Deserialize, Debug, Default, ToSchema)]
pub struct UpdateProductRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
}


#[derive(Serialize, Debug, Clone, Deserialize, ToSchema)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub created_at: String,
    pub updated_at: String,
}

impl ProductResponse {
    pub fn from_product(product: &Product) -> Self {
        ProductResponse {
            id: product._id.expect("Product from DB must have an ID").to_hex(),
            name: product.name.clone(),
            description: product.description.clone(),
            price: product.price,
            created_at: product.created_at.to_string(),
            updated_at: product.updated_at.to_string(),
        }
    }
}