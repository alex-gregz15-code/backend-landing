use axum::{extract::State, response::Json, routing::{get, post}, Router};
use crate::api::auth::{self as auth_api, AppState};

pub fn auth() -> Router<AppState> {
    Router::new()
        .route("/api/hello", get(hello))
        .route("/api/auth/register", post(auth_api::register))
        .route("/api/auth/login", post(auth_api::login))
}

async fn hello(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "message": "Backend is running." }))
}
