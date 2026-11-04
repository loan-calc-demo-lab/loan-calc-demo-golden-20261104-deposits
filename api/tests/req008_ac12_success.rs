// Test AC12: successful payout creation
mod common;
use common::{post_json};
use serde_json::json;

#[tokio::test]
async fn req008_ac12_success() {
    let app = deposits_api::app();
    let req = json!({
        "account_id": "CA-2001",
        "amount": 4950.0,
        "reference": "topup-PL-1001"
    });
    let (status, body) = post_json(&app, "/payouts", req, [("Idempotency-Key", "k1")].as_ref()).await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    // Expected fields from contract
    assert_eq!(body["account_id"], "CA-2001");
    assert_eq!(body["amount"], 4950.0);
    assert_eq!(body["new_balance"], 6200.0);
    // payout_id and created_at are present
    assert!(body["payout_id"].is_string());
    assert!(body["created_at"].is_string());
}
