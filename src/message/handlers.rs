use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{message::models::MessageResponse, state::AppState};

use super::utils::message_to_responses;

#[utoipa::path(
  get,
  path = "",
  tag = "message",
  responses(
      (status = 200, description = "List all message successfully", body = [MessageResponse])
  ),
  security(
      ("token" = [])
  )
)]
pub async fn list_messages(
  State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
  match state.db_repo.find_all_message().await {
      Ok(messages) => {
          Ok((StatusCode::OK, Json(message_to_responses(&messages))))
      }
      Err(e) => {
          tracing::error!("Failed to list products: {:?}", e);
          Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Failed to retrieve products".to_string(),
          ))
      }
  }
}