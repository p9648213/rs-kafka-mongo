use crate::{
  auth::{
      models::{AuthResponse, LoginRequest, SignupRequest, User},
      utils::{create_jwt, hash_password, verify_password},
  },
  state::AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use mongodb::bson::oid::ObjectId;
use tracing::{error, info, warn};

#[utoipa::path(
    post,
    path = "/signup",
    tag = "user",
    responses(
        (status = 200, description = "Signup successfully")
    ),
)]
pub async fn signup(
  State(state): State<AppState>,
  Json(payload): Json<SignupRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
  if payload.username.is_empty() || payload.password.is_empty() {
      return Err((
          StatusCode::BAD_REQUEST,
          "Username and password cannot be empty".to_string(),
      ));
  }

  match state.db_repo.find_user_by_username(&payload.username).await {
      Ok(Some(_)) => {
          warn!("Signup attempt with existing username: {}", payload.username);
          return Err((
              StatusCode::CONFLICT,
              "Username already exists".to_string(),
          ));
      }
      Ok(None) => {}
      Err(e) => {
          error!("Database error checking username: {:?}", e);
          return Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Failed to check username availability".to_string(),
          ));
      }
  }

  let password_hash = match hash_password(&payload.password) {
      Ok(hash) => hash,
      Err(e) => {
          error!("Failed to hash password: {:?}", e);
          return Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Failed to process registration".to_string(),
          ));
      }
  };

  let new_user = User {
      _id: Some(ObjectId::new()),
      username: payload.username.clone(),
      password_hash,
  };

  match state.db_repo.create_user(new_user).await {
      Ok(user_id) => {
          info!("New user created with ID: {}", user_id);
          Ok((StatusCode::CREATED, "User created successfully".to_string()))
      }
      Err(e) => {
          error!("Failed to create user in database: {:?}", e);
          Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Failed to register user".to_string(),
          ))
      }
  }
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "user",
    responses(
        (status = 200, description = "Login successfully", body = AuthResponse)
    ),
)]
pub async fn login(
  State(state): State<AppState>,
  Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
  let user = match state.db_repo.find_user_by_username(&payload.username).await {
      Ok(Some(user)) => user,
      Ok(None) => {
          warn!("Login attempt for non-existent user: {}", payload.username);
          return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
      }
      Err(e) => {
          error!("Database error during login: {:?}", e);
          return Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Login failed".to_string(),
          ));
      }
  };

  match verify_password(&payload.password, &user.password_hash) {
      Ok(true) => {}
      Ok(false) => {
          warn!("Incorrect password attempt for user: {}", payload.username);
          return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
      }
      Err(e) => {
          error!("Password verification error for user {}: {:?}", payload.username, e);
          return Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Login failed".to_string(),
          ));
      }
  }

  let user_id = user._id.expect("User from DB should have an ID").to_hex();
  match create_jwt(&user_id, &state.config) {
      Ok(token) => {
          info!("User logged in successfully: {}", payload.username);
          let response = AuthResponse {
              token,
              token_type: "Bearer".to_string(),
          };
          Ok((StatusCode::OK, Json(response)))
      }
      Err(e) => {
          error!("Failed to create JWT for user {}: {:?}", payload.username, e);
          Err((
              StatusCode::INTERNAL_SERVER_ERROR,
              "Login failed".to_string(),
          ))
      }
  }
}