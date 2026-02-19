# 使用 Docker 部署 codebox-rmcp

本文档说明如何使用 Docker / Docker Compose 部署与运行 **codebox-rmcp**：一个基于 Rust 构建、用于运行 Python 代码的 MCP（Model Context Protocol）工具。

项目提供：

- 健康检查端点：`GET /health`（返回 `OK`，公开访问；见 [`health()`](src/server.rs:19)）
- MCP 端点：`/mcp`（需要 Bearer Token 鉴权；路由见 [`run_server()`](src/server.rs:23)，鉴权见 [`auth_middleware()`](src/auth.rs:42)）
- 监听地址：由 `SERVER_HOST` 控制，默认 `0.0.0.0`（见 [`run_server()`](src/server.rs:24)）
- 监听端口：由 `SERVER_PORT` 控制，默认 `8080`（见 [`run_server()`](src/server.rs:25)；示例 `.env` 为 18081，见 [`.env`](.env:1)）

> 说明：Docker 运行时镜像已预装 `python3` 与 `uv`（见 [`Dockerfile`](Dockerfile:1)），用于在容器内执行 Python 代码。

---

## 1. 前提条件

- Docker >= 20.10
- Docker Compose >= 2.0（可选但推荐；使用 `docker compose` 命令）

---

## 2. 快速开始（使用 Docker Compose）

仓库已提供 Compose 配置（见 [`docker-compose.yml`](docker-compose.yml:1)），默认：

- `env_file` 加载 `.env`
- 端口映射 `18081:18081`
- 健康检查访问 `http://127.0.0.1:${SERVER_PORT:-18081}/health`

按下列步骤启动：

```bash
# 1. 克隆项目
git clone <repo-url>
cd codebox-rmcp

# 2. 配置环境变量
cp .env.example .env
# 编辑 .env 设置 AUTH_TOKENS 等

# 3. 构建并启动
docker compose up -d --build

# 4. 验证服务
curl http://localhost:18081/health
```

> 如果仓库未提供 `.env.example`，可参考 [`.env`](.env:1) 手动创建 `.env`（至少需要设置 `AUTH_TOKENS` 与 `SERVER_PORT`）。

健康检查预期输出：

```text
OK
```

---

## 3. 使用 Docker 手动构建与运行

### 3.1 构建镜像

在项目根目录执行：

```bash
docker build -t codebox-rmcp:latest .
```

> 镜像使用多阶段构建：编译阶段基于 `rust:1.85-bookworm`，运行阶段基于 `debian:bookworm-slim`（见 [`Dockerfile`](Dockerfile:1)）。

### 3.2 运行容器（推荐使用 `.env`）

确保本地存在 `.env`（可参考 [`.env`](.env:1)），然后运行：

```bash
docker run -d \
  --name codebox-rmcp \
  --env-file .env \
  -p 18081:18081 \
  --restart unless-stopped \
  codebox-rmcp:latest
```

关键点：

- `-p 18081:18081` 需要与容器内 `SERVER_PORT` 保持一致。若 `.env` 中 `SERVER_PORT=18081`，则映射无需额外调整。
- 如果你不使用 `.env`，可以直接通过 `-e` 显式指定端口与 token。

### 3.3 运行容器（不使用 `.env`，通过 `-e` 指定）

```bash
docker run -d \
  --name codebox-rmcp \
  -e AUTH_TOKENS='replace-me-please' \
  -e SERVER_HOST='0.0.0.0' \
  -e SERVER_PORT='18081' \
  -e EXECUTION_TIMEOUT='60' \
  -p 18081:18081 \
  --restart unless-stopped \
  codebox-rmcp:latest
```

---

## 4. 环境变量配置

服务在启动时读取以下环境变量（解析逻辑见 [`run_server()`](src/server.rs:23)，鉴权读取见 [`TokenStore::from_env()`](src/auth.rs:16)）：

| 变量名 | 说明 | 默认值 | 示例 |
|---|---|---:|---|
| `AUTH_TOKENS` | Bearer Token 白名单，多个 token 用英文逗号 `,` 分隔。用于保护 `/mcp` 端点。 | 空字符串（会导致 token 集合为空，`/mcp` 请求将全部返回 401；见 [`TokenStore::from_env()`](src/auth.rs:16) 与 [`auth_middleware()`](src/auth.rs:42)） | `tokenA,tokenB` |
| `SERVER_HOST` | HTTP 监听地址 | `0.0.0.0` | `0.0.0.0` |
| `SERVER_PORT` | HTTP 监听端口 | `8080` | `18081` |
| `EXECUTION_TIMEOUT` | Python 代码执行超时（秒） | `60` | `60` |
| `PUBLIC_URL` | 公共文件访问的基础 URL，用于静态文件服务（`/public/*`） | `http://localhost:18081` | `https://your-domain.com` |

> 提示：Compose 默认映射端口为 `18081:18081`（见 [`docker-compose.yml`](docker-compose.yml:7)），因此通常需要在 `.env` 中设置 `SERVER_PORT=18081`（示例见 [`.env`](.env:3)）。

---

## 5. 接口验证与鉴权示例

### 5.1 健康检查

```bash
curl http://localhost:18081/health
```

预期返回：`OK`（见 [`health()`](src/server.rs:19)）。

### 5.2 访问 MCP 端点（需要 Bearer Token）

`/mcp` 端点需要鉴权（路由保护见 [`run_server()`](src/server.rs:43)）。请求时需要添加：

```http
Authorization: Bearer <your-token>
```

例如（仅示意，具体 MCP 请求体取决于你的客户端/协议实现）：

```bash
TOKEN='your-token'
curl -i \
  -H "Authorization: Bearer ${TOKEN}" \
  http://localhost:18081/mcp
```

---

## 6. 常用操作（Docker Compose）

在项目根目录执行：

- 查看日志：

  ```bash
  docker compose logs -f
  ```

- 重启服务：

  ```bash
  docker compose restart
  ```

- 停止并删除容器（保留镜像）：

  ```bash
  docker compose down
  ```

- 重新构建并启动：

  ```bash
  docker compose up -d --build
  ```

---

## 7. 共享存储

MCP 容器（`codebox-mcp`）与 Worker 容器（`codebox-worker`）通过 Docker 命名卷 **`shared-data`** 共享 `/shared` 目录，用于两个容器之间的文件交换（配置见 [`docker-compose.yml`](docker-compose.yml:14) 与 [`docker-compose.yml`](docker-compose.yml:40)）。

### 7.1 挂载路径

| 容器 | 挂载点 |
|------|--------|
| `codebox-mcp` | `/shared` |
| `codebox-worker` | `/shared` |

### 7.2 权限说明

Worker 容器以 UID 10001 运行（见 [`docker-compose.yml`](docker-compose.yml:43)），共享目录已设置对应权限，确保 Worker 可以正常读写 `/shared` 路径。

### 7.3 持久化说明

- **数据保留**：使用 Docker 命名卷，数据在容器重启后保留
- **数据清除**：执行 `docker-compose down -v` 会删除命名卷，从而清除共享数据
- **仅删除容器**：执行 `docker-compose down`（不带 `-v`）会保留共享数据

### 7.4 静态文件服务

MCP 服务器提供静态文件服务，将 `/shared` 目录通过 HTTP 路径 `/public/*` 公开暴露：

- **访问方式**：`{PUBLIC_URL}/public/<文件名>`
- **示例**：文件保存在 `/shared/test.png`，可通过 `http://localhost:18081/public/test.png` 访问
- **认证**：该路由为公开访问，不需要 Bearer Token 认证
- **用途**：MCP 工具执行 Python 代码后，可将生成的文件（如图片、报告等）写入 `/shared` 目录，然后通过 HTTP 直接访问

> 注意：`PUBLIC_URL` 的默认值为 `http://localhost:18081`，如果通过反向代理或域名访问，需要设置对应的 `PUBLIC_URL` 环境变量。

---

## 8. 生产部署建议

### 7.1 安全

- **务必修改** `AUTH_TOKENS`（不要使用示例值；示例见 [`.env`](.env:1)）。
- 建议通过反向代理统一做 TLS（HTTPS）与访问控制。

### 7.2 网络

- 建议在公网部署时使用反向代理（如 Nginx / Caddy），仅对外暴露必要端口。
- 如果服务只被同机或内网访问，可限制监听与防火墙规则。

### 7.3 资源限制

可在 Compose 中增加资源限制（示意）：

```yaml
services:
  codebox-rmcp:
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
```

> 注意：`deploy` 在 Docker Swarm 中原生生效；在非 Swarm 场景下，资源限制可改用 `mem_limit`/`cpus` 等方式（取决于你的 Docker/Compose 版本）。

### 7.4 日志

- 生产环境建议配置日志驱动（例如 `json-file` 的滚动策略，或 `loki`/`fluentd` 等集中式方案）。

### 7.5 数据持久化

- 当前服务不需要持久化数据卷。

---

## 9. 故障排查

### 8.1 容器无法启动

优先查看日志：

```bash
docker compose logs --tail=200
```

或（手动运行容器场景）：

```bash
docker logs -n 200 codebox-rmcp
```

### 8.2 端口冲突

现象：启动报端口被占用。

处理方式：

1. 修改 `.env` 中 `SERVER_PORT`（见 [`.env`](.env:3)）。
2. 同步修改 [`docker-compose.yml`](docker-compose.yml:7) 的端口映射，例如改为 `- "28081:28081"`。

> 原则：**容器内端口（`SERVER_PORT`）与端口映射右侧**必须一致。

### 8.3 Python 执行失败

进入容器检查 `uv` 与 `python3`：

```bash
docker exec -it codebox-rmcp sh
uv --version
python3 --version
```

运行时镜像安装逻辑见 [`Dockerfile`](Dockerfile:15)。

### 8.4 健康检查失败

Compose 健康检查会访问：`http://127.0.0.1:${SERVER_PORT:-18081}/health`（见 [`docker-compose.yml`](docker-compose.yml:12)）。如果健康检查失败：

- 确认容器内实际监听地址为 `0.0.0.0`（即 `SERVER_HOST=0.0.0.0`；默认值见 [`run_server()`](src/server.rs:24)）。
- 确认 `SERVER_PORT` 与端口映射一致（Compose 映射见 [`docker-compose.yml`](docker-compose.yml:7)）。

---

## 10. 镜像信息（多阶段构建说明）

镜像构建分为两阶段（见 [`Dockerfile`](Dockerfile:1)）：

1. **builder 阶段**（`rust:1.85-bookworm`）
   - 拷贝 `Cargo.toml` / `Cargo.lock` / `src/`
   - 执行 `cargo build --release --locked`
   - 产出 release 二进制：`codebox-rmcp`

2. **runtime 阶段**（`debian:bookworm-slim`）
   - 安装运行时依赖：`ca-certificates`、`curl`、`python3`
   - 通过官方脚本安装 `uv`，并将其加入 `PATH`
   - 复制二进制到 `/usr/local/bin/codebox-rmcp`
   - 默认设置 `SERVER_HOST=0.0.0.0`、`SERVER_PORT=8080`

> 补充：`EXPOSE 8080` 仅为镜像元数据声明（见 [`Dockerfile`](Dockerfile:34)），实际对外端口由 `SERVER_PORT` 与运行时端口映射决定。
