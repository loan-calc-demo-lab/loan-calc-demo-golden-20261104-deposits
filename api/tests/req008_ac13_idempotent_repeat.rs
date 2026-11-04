// Test AC13: idempotent repeat request returns same result
mod common;
use common::{post_json};
use serde_json::json;

#[tokio::test]
async fn req008_ac13_idempotent_repeat() {
    let app = deposits_api::app();
    let req = json!({
        "account_id": "CA-2001",
        "amount": 4950.0,
        "reference": "topup-PL-1001"
    });
    // first request
    let (status1, body1) = post_json(&app, "/payouts", req.clone(), [("Idempotency-Key", "k2")].as_ref()).await;
    assert_eq!(status1, axum::http::StatusCode::CREATED);
    // repeat same request with same key
    let (status2, body2) = post_json(&app, "/payouts", req.clone(), [("Idempotency-Key", "k2")].as_ref()).await;
    assert_eq!(status2, axum::http::StatusCode::OK);
    assert_eq!(body1, body2);
    // ensure balance unchanged (new_balance same as first response)
    assert_eq!(body2["new_balance"], 6200.0);
}
