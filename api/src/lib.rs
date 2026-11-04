pub mod accounts;

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Shared state of the service: the ledger (in memory in this demo).
#[derive(Clone)]
pub struct AppState {
    pub ledger: Arc<Mutex<accounts::Ledger>>,
}

impl AppState {
    pub fn demo() -> Self {
        AppState { ledger: Arc::new(Mutex::new(accounts::Ledger::demo())) }
    }
}

/// The service with fresh demo data.
pub fn app() -> Router {
    app_with(AppState::demo())
}

pub fn app_with(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { Json(json!({"status": "ok"})) }))
        .route("/accounts/:account_id", get(account))
        .with_state(state)
}

async fn account(State(state): State<AppState>, Path(account_id): Path<String>) -> impl IntoResponse {
    match state.ledger.lock().unwrap().get(&account_id) {
        Some(a) => (StatusCode::OK, Json(serde_json::to_value(a).unwrap())),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": "not_found", "message": "unknown account"}))),
    }
}
