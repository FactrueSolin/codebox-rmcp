use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*, schemars, tool, tool_handler, tool_router,
};

use crate::executor::execute_python;

#[derive(Debug, Clone)]
pub struct PythonRunner {
    pub timeout_secs: u64,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RunPythonArgs {
    /// 需要执行的 Python 代码字符串
    pub code: String,
}

#[tool_router]
impl PythonRunner {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            timeout_secs,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = r#"执行python代码并返回结果
# 示例代码
# /// script
# dependencies = [
#   "requests<3",
#   "rich",
# ]
# ///

import requests
from rich.pretty import pprint

resp = requests.get("https://peps.python.org/api/peps.json")
data = resp.json()
pprint([(k, v["title"]) for k, v in data.items()][:10])"#)]
    async fn run_python(
        &self,
        Parameters(RunPythonArgs { code }): Parameters<RunPythonArgs>,
    ) -> Result<CallToolResult, McpError> {
        match execute_python(&code, self.timeout_secs).await {
            Ok(result) => {
                let payload = serde_json::json!({
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                    "exit_code": result.exit_code,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    payload.to_string(),
                )]))
            }
            Err(err) => {
                let payload = serde_json::json!({
                    "stdout": "",
                    "stderr": err.to_string(),
                    "exit_code": -1,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    payload.to_string(),
                )]))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for PythonRunner {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("MCP server for executing Python code via uv run".to_string()),
            ..Default::default()
        }
    }
}
