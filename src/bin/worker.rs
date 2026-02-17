use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use codebox_rmcp::executor::execute_python;
use serde::{Deserialize, Serialize};
use tracing_subscriber::prelude::*;

#[derive(Debug, Clone)]
struct AppState {
    default_timeout: u64,
}

#[derive(Debug, Deserialize)]
struct ExecuteRequest {
    code: String,
    timeout: Option<u64>,
}

#[derive(Debug, Serialize)]
struct ExecuteResponse {
    stdout: String,
    stderr: String,
    exit_code: i32,
    error: Option<String>,
}

async fn health() -> &'static str {
    "OK"
}

async fn execute(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteRequest>,
) -> (StatusCode, Json<ExecuteResponse>) {
    let timeout_secs = payload.timeout.unwrap_or(state.default_timeout);

    match execute_python(&payload.code, timeout_secs).await {
        Ok(result) => (
            StatusCode::OK,
            Json(ExecuteResponse {
                stdout: result.stdout,
                stderr: result.stderr,
                exit_code: result.exit_code,
                error: None,
            }),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ExecuteResponse {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: -1,
                error: Some(err.to_string()),
            }),
        ),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let host = std::env::var("WORKER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WORKER_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(9000);
    let timeout_secs = std::env::var("EXECUTION_TIMEOUT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60);

    let app = Router::new()
        .route("/health", get(health))
        .route("/execute", post(execute))
        .with_state(AppState {
            default_timeout: timeout_secs,
        });

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Worker server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}

