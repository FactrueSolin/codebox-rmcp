use std::fmt;

use crate::executor::ExecutionResult;

#[derive(Debug, Clone)]
pub struct WorkerClient {
    base_url: String,
    client: reqwest::Client,
    default_timeout: u64,
}

#[derive(Debug)]
pub enum WorkerClientError {
    Request(reqwest::Error),
    ServerError { status: u16, message: String },
}

impl fmt::Display for WorkerClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(err) => write!(f, "请求 Worker 失败: {err}"),
            Self::ServerError { status, message } => {
                write!(f, "Worker 返回错误 (status={status}): {message}")
            }
        }
    }
}

impl std::error::Error for WorkerClientError {}

#[derive(Debug, serde::Serialize)]
struct ExecuteRequest {
    code: String,
    timeout: u64,
}

#[derive(Debug, serde::Deserialize)]
struct ExecuteResponse {
    stdout: String,
    stderr: String,
    exit_code: i32,
    error: Option<String>,
}

impl WorkerClient {
    pub fn from_env() -> Self {
        let base_url = std::env::var("WORKER_URL").unwrap_or_else(|_| "http://localhost:9000".to_string());
        let default_timeout = std::env::var("EXECUTION_TIMEOUT")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(60);

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
            default_timeout,
        }
    }

    pub async fn execute(
        &self,
        code: &str,
        timeout: Option<u64>,
    ) -> Result<ExecutionResult, WorkerClientError> {
        let timeout_secs = timeout.unwrap_or(self.default_timeout);
        let url = format!("{}/execute", self.base_url);

        let response = self
            .client
            .post(url)
            .json(&ExecuteRequest {
                code: code.to_string(),
                timeout: timeout_secs,
            })
            .send()
            .await
            .map_err(WorkerClientError::Request)?;

        let status = response.status();
        let body: ExecuteResponse = response.json().await.map_err(WorkerClientError::Request)?;

        if !status.is_success() {
            return Err(WorkerClientError::ServerError {
                status: status.as_u16(),
                message: body
                    .error
                    .unwrap_or_else(|| format!("{} {}", body.stdout, body.stderr).trim().to_string()),
            });
        }

        if let Some(message) = body.error {
            return Err(WorkerClientError::ServerError {
                status: status.as_u16(),
                message,
            });
        }

        Ok(ExecutionResult {
            stdout: body.stdout,
            stderr: body.stderr,
            exit_code: body.exit_code,
        })
    }
}

