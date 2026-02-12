use std::{fmt, io::Write, process::Stdio, time::Duration};

use tempfile::Builder;
use tokio::{
    io::AsyncReadExt,
    process::Command,
    time::{error::Elapsed, timeout},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Debug)]
pub enum ExecutorError {
    TempFile(std::io::Error),
    WriteCode(std::io::Error),
    Spawn(std::io::Error),
    MissingPipe(&'static str),
    Wait(std::io::Error),
    ReadStdout(std::io::Error),
    ReadStderr(std::io::Error),
    Join(tokio::task::JoinError),
    Timeout { seconds: u64, source: Elapsed },
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TempFile(e) => write!(f, "创建临时文件失败: {e}"),
            Self::WriteCode(e) => write!(f, "写入 Python 代码失败: {e}"),
            Self::Spawn(e) => write!(f, "启动 uv 进程失败: {e}"),
            Self::MissingPipe(pipe) => write!(f, "未能获取子进程 {pipe} 管道"),
            Self::Wait(e) => write!(f, "等待子进程结束失败: {e}"),
            Self::ReadStdout(e) => write!(f, "读取 stdout 失败: {e}"),
            Self::ReadStderr(e) => write!(f, "读取 stderr 失败: {e}"),
            Self::Join(e) => write!(f, "异步任务 join 失败: {e}"),
            Self::Timeout { seconds, .. } => write!(f, "Python 执行超时（>{seconds} 秒）"),
        }
    }
}

impl std::error::Error for ExecutorError {}

pub async fn execute_python(code: &str, timeout_secs: u64) -> Result<ExecutionResult, ExecutorError> {
    let mut temp = Builder::new()
        .prefix("codebox-")
        .suffix(".py")
        .tempfile()
        .map_err(ExecutorError::TempFile)?;

    temp.write_all(code.as_bytes())
        .map_err(ExecutorError::WriteCode)?;
    temp.flush().map_err(ExecutorError::WriteCode)?;

    let path = temp.path().to_path_buf();

    let mut child = Command::new("uv")
        .arg("run")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(ExecutorError::Spawn)?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or(ExecutorError::MissingPipe("stdout"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or(ExecutorError::MissingPipe("stderr"))?;

    let stdout_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stdout
            .read_to_end(&mut buf)
            .await
            .map_err(ExecutorError::ReadStdout)?;
        Ok::<Vec<u8>, ExecutorError>(buf)
    });

    let stderr_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        stderr
            .read_to_end(&mut buf)
            .await
            .map_err(ExecutorError::ReadStderr)?;
        Ok::<Vec<u8>, ExecutorError>(buf)
    });

    let status = match timeout(Duration::from_secs(timeout_secs), child.wait()).await {
        Ok(result) => result.map_err(ExecutorError::Wait)?,
        Err(source) => {
            let _ = child.kill().await;
            return Err(ExecutorError::Timeout {
                seconds: timeout_secs,
                source,
            });
        }
    };

    let stdout_bytes = stdout_task.await.map_err(ExecutorError::Join)??;
    let stderr_bytes = stderr_task.await.map_err(ExecutorError::Join)??;

    Ok(ExecutionResult {
        stdout: String::from_utf8_lossy(&stdout_bytes).to_string(),
        stderr: String::from_utf8_lossy(&stderr_bytes).to_string(),
        exit_code: status.code().unwrap_or(-1),
    })
}

