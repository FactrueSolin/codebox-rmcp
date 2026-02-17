use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    middleware,
    routing::get,
};
use rmcp::transport::{
    StreamableHttpServerConfig,
    streamable_http_server::{session::local::LocalSessionManager, tower::StreamableHttpService},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    auth::{TokenStore, auth_middleware},
    tools::PythonRunner,
    worker_client::WorkerClient,
};

async fn health() -> &'static str {
    "OK"
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let token_store = Arc::new(TokenStore::from_env());
    let worker_client = WorkerClient::from_env();

    let mcp_service: StreamableHttpService<PythonRunner, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(PythonRunner::new(worker_client.clone())),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default(),
        );

    let protected_mcp_router = Router::new()
        .nest_service("/mcp", mcp_service)
        .layer(middleware::from_fn_with_state(
            token_store,
            auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(health))
        .merge(protected_mcp_router)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("MCP server listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
