use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct MessageResponse {
    pub id: String,
    pub message: String
}

impl MessageResponse {
    pub fn from_message(message: &Message) -> Self {
        MessageResponse {
            id: message._id.expect("Product from DB must have an ID").to_hex(),
            message: message.message.clone()
        }
    }
}