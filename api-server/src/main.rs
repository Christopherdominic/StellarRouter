mod handlers;
mod rpc;
mod state;
mod types;
mod websocket;

#[cfg(test)]
mod tests;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use rpc::SorobanRpcClient;
use state::AppState;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let rpc_url = env::var("SOROBAN_RPC_URL")
        .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());

    let router_core_contract_id = env::var("ROUTER_CORE_CONTRACT_ID").ok();

    let listen_addr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string());

    let rpc = SorobanRpcClient::new(rpc_url, router_core_contract_id);
    let state = AppState::new(rpc);

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/simulate", post(handlers::simulate))
        .route("/routes/:name", get(handlers::get_route))
        .route("/ws", get(websocket::ws_handler))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(state);

    tracing::info!("listening on {}", listen_addr);
    let listener = tokio::net::TcpListener::bind(&listen_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
