use super::models::{Message, MessageResponse};

pub fn message_to_responses(messages: &[Message]) -> Vec<MessageResponse> {
  messages.iter().map(MessageResponse::from_message).collect()
}