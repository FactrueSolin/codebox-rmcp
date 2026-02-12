use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::State,
    http::{Request, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

#[derive(Debug, Clone)]
pub struct TokenStore {
    valid_tokens: HashSet<String>,
}

impl TokenStore {
    pub fn from_env() -> Self {
        let raw = std::env::var("AUTH_TOKENS").unwrap_or_default();
        let valid_tokens = raw
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(ToOwned::to_owned)
            .collect::<HashSet<_>>();

        Self { valid_tokens }
    }

    pub fn is_valid(&self, token: &str) -> bool {
        self.valid_tokens.contains(token)
    }
}

pub fn extract_token<B>(request: &Request<B>) -> Option<String> {
    request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| auth.strip_prefix("Bearer "))
        .map(ToOwned::to_owned)
}

pub async fn auth_middleware(
    State(token_store): State<Arc<TokenStore>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    match extract_token(&request) {
        Some(token) if token_store.is_valid(&token) => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

