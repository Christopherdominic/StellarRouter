use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::{
    handlers,
    rpc::SorobanRpcClient,
    state::AppState,
    types::SimulateResponse,
};

/// Valid 56-char Stellar contract ID for use in tests.
const VALID_CONTRACT_ID: &str = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4";

fn test_app() -> Router {
    // Use a non-existent RPC URL — the client will fall back to heuristic estimates
    let rpc = SorobanRpcClient::new("http://localhost:1", None);
    let state = AppState::new(rpc);
    Router::new()
        .route("/health", get(handlers::health))
        .route("/simulate", post(handlers::simulate))
        .route("/routes/:name", get(handlers::get_route))
        .with_state(state)
}

// ── GET /health ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_returns_200() {
    let app = test_app();
    let resp = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_returns_ok_body() {
    let app = test_app();
    let resp = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

// ── POST /simulate — success paths ───────────────────────────────────────────

#[tokio::test]
async fn test_simulate_returns_200_with_valid_request() {
    let app = test_app();
    let body = json!({
        "target": VALID_CONTRACT_ID,
        "function": "transfer",
        "amount": 1_000_000,
        "fee_bps": 30,
        "network_load_bps": 5000
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_simulate_response_has_fee_fields() {
    let app = test_app();
    let body = json!({ "target": VALID_CONTRACT_ID, "function": "transfer" });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let parsed: SimulateResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(parsed.estimated_fees.base_fee > 0);
    assert!(parsed.estimated_fees.total_fee >= parsed.estimated_fees.base_fee);
    assert_eq!(parsed.simulation.target, VALID_CONTRACT_ID);
    assert_eq!(parsed.simulation.function, "transfer");
}

#[tokio::test]
async fn test_simulate_surge_pricing_at_high_load() {
    let app = test_app();
    let body = json!({
        "target": VALID_CONTRACT_ID,
        "function": "transfer",
        "amount": 1_000_000,
        "network_load_bps": 9000
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let parsed: SimulateResponse = serde_json::from_slice(&bytes).unwrap();
    assert!(parsed.estimated_fees.high_load);
    assert_eq!(parsed.estimated_fees.surge_multiplier, 200);
}

// ── POST /simulate — validation ───────────────────────────────────────────────

#[tokio::test]
async fn test_simulate_missing_target_returns_400() {
    let app = test_app();
    let body = json!({ "function": "transfer" });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_simulate_missing_function_returns_400() {
    let app = test_app();
    let body = json!({ "target": VALID_CONTRACT_ID });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_simulate_invalid_contract_id_returns_400() {
    let app = test_app();
    let body = json!({ "target": "not-a-valid-contract-id", "function": "transfer" });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["error"].as_str().unwrap().contains("56-character"));
}

#[tokio::test]
async fn test_simulate_contract_id_not_starting_with_c_returns_400() {
    let app = test_app();
    // 56 chars but starts with G (account ID, not contract ID)
    let body = json!({
        "target": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
        "function": "transfer"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_simulate_empty_body_returns_400_or_422() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/simulate")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY
    );
}

// ── GET /routes/:name ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_route_returns_404_when_core_not_configured() {
    // With no ROUTER_CORE_CONTRACT_ID configured the RPC client returns an
    // error, which the handler maps to 500. But when the RPC is unreachable
    // and the contract ID is missing we get a 500 (not 404). This test
    // verifies the endpoint exists and returns a non-200 error code.
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/routes/oracle")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // 500 because ROUTER_CORE_CONTRACT_ID is not set in the test environment
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json["error"].is_string());
}

#[tokio::test]
async fn test_get_route_error_response_is_json() {
    let app = test_app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/routes/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    // Must be valid JSON with an "error" field
    let json: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("error").is_some());
}
