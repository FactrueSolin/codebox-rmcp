# codebox-rmcp

[![Rust](https://img.shields.io/badge/Rust-2024%20edition-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)
[![MCP](https://img.shields.io/badge/Protocol-MCP%20(Streamable%20HTTP)-informational)](https://modelcontextprotocol.io/)

一个基于 **MCP（Model Context Protocol）** 的远程 **Python 代码执行**服务，使用 **Rust** 构建。服务通过 **Streamable HTTP** 暴露 MCP 工具，并在服务端使用 `uv run` 执行传入的 Python 代码，返回 `stdout / stderr / exit_code`。

> 安全提示：该项目具备“执行任意 Python 代码”的高危能力，请务必开启鉴权并配合容器/网络/权限隔离使用。

---

## 功能特性

- **MCP Streamable HTTP 服务器**：提供 `/mcp` 端点对接 MCP 客户端。
- **Python 执行能力**：通过 `uv run` 运行代码（写入临时 `.py` 文件后执行）。
- **结构化执行结果**：返回 `stdout`、`stderr`、`exit_code`（以 JSON 文本形式封装）。
- **超时控制**：支持通过环境变量设置执行超时（默认 60 秒）。
- **Bearer Token 鉴权**：保护 `/mcp`，健康检查 `/health` 公开。
- **Docker 友好**：提供 Dockerfile 与 Compose 配置。

---

## 快速开始

### 前提条件

- **Rust 1.88+**（建议；项目使用 `edition = "2024"`，需要较新的 Rust 版本）
- **Python 3**
- **uv**（命令行需可直接执行 `uv`）

### 安装与运行（本地）

1) 克隆并进入项目目录：

```bash
git clone <repo-url>
cd codebox-rmcp
```

2) 配置环境变量（建议使用 `.env`）：

仓库已提供 `.env` 示例文件，你可以直接编辑它（请务必替换 token）：

```dotenv
AUTH_TOKENS=replace-me-please
SERVER_HOST=0.0.0.0
SERVER_PORT=18081
EXECUTION_TIMEOUT=60
```

3) 启动服务：

```bash
cargo run
```

启动后日志会输出监听地址，例如：

```text
MCP server listening on http://0.0.0.0:18081
```

### 验证服务

- 健康检查：

```bash
curl http://localhost:18081/health
```

预期输出：

```text
OK
```

---

## 配置说明

服务在启动时从环境变量读取配置（并支持 `.env` 文件加载）。

| 变量名 | 说明 | 默认值 | 示例 |
|---|---|---:|---|
| `AUTH_TOKENS` | Bearer Token 白名单，多个 token 用英文逗号 `,` 分隔。用于保护 `/mcp` 端点。 | 空字符串（token 集合为空，`/mcp` 将全部返回 401） | `tokenA,tokenB` |
| `SERVER_HOST` | HTTP 监听地址 | `0.0.0.0` | `0.0.0.0` |
| `SERVER_PORT` | HTTP 监听端口 | `8080` | `18081` |
| `EXECUTION_TIMEOUT` | Python 代码执行超时（秒） | `60` | `60` |

---

## API 端点

### `GET /health`

- 用途：健康检查（公开访问）
- 返回：`OK`

### `/mcp`（MCP Streamable HTTP）

- 用途：MCP 端点（Streamable HTTP Server）
- 鉴权：**必须**携带 `Authorization: Bearer <token>`

鉴权示例（仅演示 Header；具体 MCP 请求体取决于你的 MCP 客户端/协议实现）：

```bash
TOKEN='your-token'
curl -i \
  -H "Authorization: Bearer ${TOKEN}" \
  http://localhost:18081/mcp
```

---

## Docker 部署

Docker 相关说明请见 [`DOCKER.md`](./DOCKER.md)。项目提供：

- [`Dockerfile`](./Dockerfile)（多阶段构建，运行时镜像包含 `python3` 与 `uv`）
- [`docker-compose.yml`](./docker-compose.yml)（推荐使用 Compose 启动）

---

## 项目结构

```text
.
├── Cargo.toml              # Rust 包元数据与依赖
├── DOCKER.md               # Docker 部署说明
├── Dockerfile              # 容器镜像构建
├── docker-compose.yml      # Compose 编排
├── docs/
│   └── architecture.md     # 架构说明
└── src/
    ├── main.rs             # 二进制入口：加载 .env、初始化日志、启动服务
    ├── lib.rs              # 库入口：导出各模块
    ├── server.rs           # Axum 路由与 MCP Streamable HTTP 服务组装
    ├── auth.rs             # Bearer Token 鉴权中间件
    ├── tools.rs            # MCP 工具定义（run_python）
    └── executor.rs         # Python 执行器：临时文件 + `uv run` + 超时控制
```

关键模块说明：

- `src/server.rs`：
  - `/health` 路由与 `/mcp` 受保护路由
  - MCP Streamable HTTP 服务 `StreamableHttpService` 组装
- `src/tools.rs`：
  - MCP 工具 `run_python`：入参 `code: String`，返回 JSON 文本（stdout/stderr/exit_code）
- `src/executor.rs`：
  - 将代码写入临时 `.py` 文件并通过 `uv run <file>` 执行
  - 捕获 stdout/stderr 并设置执行超时
- `src/auth.rs`：
  - 读取 `AUTH_TOKENS` 并验证 `Authorization: Bearer ...`

更详细的架构说明请见：[`docs/architecture.md`](./docs/architecture.md)。

---

## 开发

### 本地开发

```bash
# 仅检查编译（推荐）
cargo check

# 运行
cargo run
```

可通过 `RUST_LOG` 控制日志级别，例如：

```bash
RUST_LOG=info cargo run
```

### 构建

```bash
cargo build --release
```

---

## 许可证

本项目采用 MIT 许可证，详见 [`LICENSE`](./LICENSE)。

