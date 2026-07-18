use deepseek_tray::api::{fetch_balance_with_url, ApiError};
use httpmock::prelude::*;

fn mock_balance_server(body: &str, status: u16) -> MockServer {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET)
            .path("/user/balance")
            .header("Authorization", "Bearer sk-test-key");
        then.status(status)
            .header("Content-Type", "application/json")
            .body(body);
    });
    server
}

fn valid_response() -> &'static str {
    r#"{
        "is_available": true,
        "balance_infos": [
            {
                "currency": "CNY",
                "total_balance": "88.50",
                "topped_up_balance": "80.00",
                "granted_balance": "8.50"
            }
        ]
    }"#
}

#[tokio::test]
async fn test_fetch_balance_success() {
    let server = mock_balance_server(valid_response(), 200);
    let base_url = server.base_url();

    let balance = fetch_balance_with_url("sk-test-key", &base_url).await.unwrap();
    assert!((balance.total - 88.50).abs() < 0.01);
    assert!((balance.topped_up - 80.00).abs() < 0.01);
    assert!((balance.granted - 8.50).abs() < 0.01);
    assert_eq!(balance.currency, "CNY");
}

#[tokio::test]
async fn test_fetch_balance_401() {
    let server = mock_balance_server(r#"{"error": "unauthorized"}"#, 401);
    let base_url = server.base_url();

    let result = fetch_balance_with_url("sk-test-key", &base_url).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::Unauthorized => {}
        e => panic!("expected Unauthorized, got {:?}", e),
    }
}

#[tokio::test]
async fn test_fetch_balance_429() {
    let server = mock_balance_server(r#"{"error": "rate limited"}"#, 429);
    let base_url = server.base_url();

    let result = fetch_balance_with_url("sk-test-key", &base_url).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::RateLimited => {}
        e => panic!("expected RateLimited, got {:?}", e),
    }
}

#[tokio::test]
async fn test_fetch_balance_server_error() {
    let server = mock_balance_server(r#"{"error": "internal"}"#, 500);
    let base_url = server.base_url();

    let result = fetch_balance_with_url("sk-test-key", &base_url).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::ServerError(500) => {}
        e => panic!("expected ServerError(500), got {:?}", e),
    }
}
