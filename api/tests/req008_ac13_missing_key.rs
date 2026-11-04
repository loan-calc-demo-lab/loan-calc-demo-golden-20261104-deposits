// Test AC13: missing Idempotency-Key returns Bad Request
mod common;
use common::{post_json};
use serde_json::json;

#[tokio::test]
async fn req008_ac13_missing_key() {
    let app = deposits_api::app();
    let req = json!({
        "account_id": "CA-2001",
        "amount": 4950.0,
        "reference": "topup-PL-1001"
    });
    let (status, body) = post_json(&app, "/payouts", req, &[] as &[(&str, &str)]).await;
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "missing_idempotency_key");
}
