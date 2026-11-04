// Test AC13: negative amount returns Unprocessable Entity
mod common;
use common::{post_json};
use serde_json::json;

#[tokio::test]
async fn req008_ac13_negative_amount() {
    let app = deposits_api::app();
    let req = json!({
        "account_id": "CA-2001",
        "amount": -10.0,
        "reference": "topup-PL-1001"
    });
    let (status, body) = post_json(&app, "/payouts", req, [("Idempotency-Key", "k4")].as_ref()).await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "invalid_amount");
}
