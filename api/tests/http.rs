use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

async fn get(path: &str) -> (StatusCode, Value) {
    let res = deposits_api::app().oneshot(Request::get(path).body(Body::empty()).unwrap()).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn account_balance() {
    let (status, v) = get("/accounts/CA-2001").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["balance"], 1250.0);
    assert_eq!(v["currency"], "EUR");
}

#[tokio::test]
async fn unknown_account_is_not_found() {
    let (status, _) = get("/accounts/CA-9999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

mod common;

#[tokio::test]
async fn helpers_share_one_service_instance() {
    let app = deposits_api::app();
    let (status, v) = common::get_json(&app, "/accounts/CA-2002").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["balance"], 310.4);
}
