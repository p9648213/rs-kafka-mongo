use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub String);