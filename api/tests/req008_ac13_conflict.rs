// Test AC13: same Idempotency-Key with different body yields conflict
mod common;
use common::{post_json};
use serde_json::json;

#[tokio::test]
async fn req008_ac13_conflict() {
    let app = deposits_api::app();
    let req1 = json!({
        "account_id": "CA-2001",
        "amount": 4950.0,
        "reference": "topup-PL-1001"
    });
    let req2 = json!({
        "account_id": "CA-2001",
        "amount": 4000.0,
        "reference": "topup-PL-1001"
    });
    // first request
    let (status1, _) = post_json(&app, "/payouts", req1, [("Idempotency-Key", "k3")].as_ref()).await;
    assert_eq!(status1, axum::http::StatusCode::CREATED);
    // second request with same key but different amount
    let (status2, body2) = post_json(&app, "/payouts", req2, [("Idempotency-Key", "k3")].as_ref()).await;
    assert_eq!(status2, axum::http::StatusCode::CONFLICT);
    assert_eq!(body2["error"], "conflict");
}
