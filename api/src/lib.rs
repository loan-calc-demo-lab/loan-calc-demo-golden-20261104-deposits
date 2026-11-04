pub mod accounts;
pub mod payouts;

use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}, Json, Router};
use serde_json::json;
use std::sync::{Arc, Mutex};

/// Shared state of the service: the ledger and the payouts (in memory in this demo).
#[derive(Clone)]
pub struct AppState {
    pub ledger: Arc<Mutex<accounts::Ledger>>,
    pub payouts: Arc<Mutex<payouts::PayoutBook>>,
}

impl AppState {
    pub fn demo() -> Self {
        AppState {
            ledger: Arc::new(Mutex::new(accounts::Ledger::demo())),
            payouts: Arc::new(Mutex::new(payouts::PayoutBook::default())),
        }
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
        .route("/payouts", post(payout))
        .with_state(state)
}

async fn account(State(state): State<AppState>, Path(account_id): Path<String>) -> impl IntoResponse {
    match state.ledger.lock().unwrap().get(&account_id) {
        Some(a) => (StatusCode::OK, Json(serde_json::to_value(a).unwrap())),
        None => (StatusCode::NOT_FOUND, Json(json!({"error": "not_found", "message": "unknown account"}))),
    }
}

/// REQ-008 AC5: POST /payouts (contract deposits-payouts-v1). Logs identifiers only (ARC-9).
async fn payout(State(state): State<AppState>, headers: HeaderMap, Json(req): Json<payouts::PayoutRequest>) -> impl IntoResponse {
    let err = |status: StatusCode, code: &str, message: &str| (status, Json(json!({"error": code, "message": message})));
    let Some(key) = headers.get("Idempotency-Key").and_then(|v| v.to_str().ok()).filter(|k| !k.is_empty()) else {
        return err(StatusCode::BAD_REQUEST, "missing_idempotency_key", "The Idempotency-Key header is required.");
    };
    let now_secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let mut ledger = state.ledger.lock().unwrap();
    let outcome = state.payouts.lock().unwrap().pay(&mut ledger, key, req, &payouts::iso_utc(now_secs));
    match outcome {
        Ok(payouts::PayoutOutcome::Created(p)) => (StatusCode::CREATED, Json(serde_json::to_value(p).unwrap())),
        Ok(payouts::PayoutOutcome::Repeated(p)) => (StatusCode::OK, Json(serde_json::to_value(p).unwrap())),
        Err(payouts::PayoutError::UnknownAccount) => err(StatusCode::NOT_FOUND, "not_found", "unknown account"),
        Err(payouts::PayoutError::KeyConflict) => err(StatusCode::CONFLICT, "conflict", "This Idempotency-Key was used for a different payout."),
        Err(payouts::PayoutError::InvalidAmount) => err(StatusCode::UNPROCESSABLE_ENTITY, "invalid_amount", "amount must be greater than 0"),
    }
}
