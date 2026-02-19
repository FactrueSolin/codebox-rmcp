use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
};

use crate::worker_client::WorkerClient;

#[derive(Debug, Clone)]
pub struct PythonRunner {
    worker_client: WorkerClient,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RunPythonArgs {
    /// 需要执行的 Python 代码字符串
    pub code: String,
}

#[tool_router]
impl PythonRunner {
    fn public_url() -> String {
        std::env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:18081".to_string())
    }

    pub fn new(worker_client: WorkerClient) -> Self {
        Self {
            worker_client,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = r#"执行python代码并返回结果
示例代码<examples>
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
pprint([(k, v["title"]) for k, v in data.items()][:10])
</examples>
<extra_info>
1. 必须显式打印才会输出结果
2. 可以在代码顶部通过depends安装所需库
3. 如果需要提供文件给用户，可以在执行python文件时，把文件写入/shared/目录，文件可通过网络公开访问，具体访问URL会在执行结果中返回
</extra_info>
"#)]
    async fn run_python(
        &self,
        Parameters(RunPythonArgs { code }): Parameters<RunPythonArgs>,
    ) -> Result<CallToolResult, McpError> {
        let public_url = Self::public_url();
        let file_access_note = format!(
            "\n\n[文件访问] 写入到 /shared/ 的文件可通过 {}/public/<filename> 访问",
            public_url
        );

        match self.worker_client.execute(&code, None).await {
            Ok(result) => {
                let payload = serde_json::json!({
                    "stdout": format!("{}{}", result.stdout, file_access_note),
                    "stderr": result.stderr,
                    "exit_code": result.exit_code,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    payload.to_string(),
                )]))
            }
            Err(err) => {
                let payload = serde_json::json!({
                    "stdout": file_access_note,
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
        let public_url = Self::public_url();
        let instructions = format!(
            "MCP server for executing Python code . Files written to /shared/ directory are publicly accessible at {}/public/<filename>",
            public_url
        );

        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(instructions),
            ..Default::default()
        }
    }
}
