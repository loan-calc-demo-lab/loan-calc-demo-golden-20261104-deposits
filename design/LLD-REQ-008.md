## Scope
- AC12-AC13 (Deposits): Idempotent payout handling and error mapping for payout creation.

## Files
- api/src/payouts.rs: new module implementing payout endpoint logic.
- api/src/lib.rs: add route `/payouts` POST handler linking to `payouts::create_payout`.

## Types and functions
```rust
// In api/src/payouts.rs
use axum::{extract::{State, Json, HeaderMap}, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PayoutRequest {
    pub account_id: String,
    pub amount: f64,
    pub reference: String,
}

#[derive(Debug, Serialize)]
pub struct PayoutResponse {
    pub payout_id: String,
    pub account_id: String,
    pub amount: f64,
    pub reference: String,
    pub new_balance: f64,
    pub created_at: String,
}

pub async fn create_payout(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PayoutRequest>,
) -> impl IntoResponse {
    // Idempotency handling using Deposits house pattern: IdempotencyKey stored in ledger.
    let key = match headers.get("Idempotency-Key") {
        Some(v) => v.to_str().unwrap_or(""),
        None => return (StatusCode::BAD_REQUEST, Json(json!({"error":"missing_idempotency_key","message":"Idempotency-Key header required"}))).into_response(),
    };
    // Validate amount positive
    if req.amount <= 0.0 {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(json!({"error":"invalid_amount","message":"Amount must be positive"}))).into_response();
    }
    // Call house pattern Deposits::record_payout(state, key, req)
    match state.ledger.lock().unwrap().record_payout(key, req) {
        Ok(resp) => (StatusCode::CREATED, Json(resp)).into_response(),
        Err(e) => match e {
            crate::accounts::PayoutError::Duplicate => (StatusCode::OK, Json(e.response())).into_response(),
            crate::accounts::PayoutError::Conflict => (StatusCode::CONFLICT, Json(e.response())).into_response(),
            crate::accounts::PayoutError::UnknownAccount => (StatusCode::NOT_FOUND, Json(e.response())).into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"internal","message":"unexpected error"}))).into_response(),
        },
    }
}
```

## Behaviour
- **AC12**: When a valid payout request is received with a new Idempotency-Key, service creates a payout, returns 201 with details, updates ledger.
- **AC13**: If the same Idempotency-Key and identical body is received again, service returns 200 with the original response, no additional credit. If same key with different body, returns 409 Conflict. Missing or empty Idempotency-Key returns 400 Bad Request. Negative amount returns 422.

## Test plan
- `api/tests/req008_ac12_success.rs`: test happy path creates payout, asserts 201 and ledger balance.
- `api/tests/req008_ac13_idempotent_repeat.rs`: repeat request with same key and body, expects 200 and unchanged balance.
- `api/tests/req008_ac13_conflict.rs`: same key different body, expects 409.
- `api/tests/req008_ac13_missing_key.rs`: no Idempotency-Key, expects 400.
- `api/tests/req008_ac13_negative_amount.rs`: amount <= 0, expects 422.

## Out of scope
- Integration with external payment rails.
- Authentication/authorization checks.
- Real persistence beyond in-memory ledger.
