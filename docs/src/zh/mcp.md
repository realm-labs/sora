# 模型上下文协议（MCP）

Sora 内置面向 AI 客户端的 MCP `2025-11-25` 服务。MCP、CLI 与 Studio 共用同一套
project、schema、data、Excel 和 build 应用服务；generated code 与 runtime bundle
始终只是输出，不能作为修改入口。

## 安装与连接

从仓库构建或安装 `sora`：

```bash
cargo install --path crates/sora-cli
```

本地客户端优先使用 stdio，并使用绝对路径：

```json
{
  "mcpServers": {
    "sora": {
      "command": "/absolute/path/to/sora",
      "args": [
        "mcp",
        "--project",
        "/absolute/path/to/project.toml"
      ]
    }
  }
}
```

stdout 只包含 JSON-RPC，审计日志写入 stderr。项目声明的 Lua parser 或 type-mapping
脚本默认不受信任。只有客户端完成 form Elicitation，或本地启动命令显式加入
`--trust-project-scripts`，Sora 才会执行它们。

支持 MCP Roots 的客户端可以省略 `--project`，随后调用 `sora_project_list` 和
`sora_project_open`。不支持 form Elicitation 的客户端必须显式传入 `root_id` 与
`relative_manifest`。

## Streamable HTTP

本地监听：

```bash
sora mcp --transport http --host 127.0.0.1 --port 8000
```

客户端连接 `http://127.0.0.1:8000/mcp`。若浏览器客户端来自其他 Origin，必须逐项加入：

```bash
sora mcp --transport http \
  --allowed-origin http://localhost:6274
```

非 loopback 监听必须提供 HTTPS public URL 与 OAuth resource server 配置：

```bash
sora mcp --transport http \
  --host 0.0.0.0 \
  --port 8443 \
  --public-url https://sora.example.com/mcp \
  --allowed-origin https://client.example.com \
  --oauth-issuer https://id.example.com \
  --oauth-audience https://sora.example.com/mcp \
  --oauth-scope sora:mcp
```

Sora 默认通过 RFC 8414 发现 JWKS；只有部署需要固定端点时才使用
`--oauth-jwks-uri`。Access token 必须是非对称签名 JWT，并通过签名、`iss`、`aud`、
`sub`、`exp`、可选 `nbf` 及全部 scope 校验。RFC 9728 protected resource metadata
位于 `/.well-known/oauth-protected-resource`。

HTTP 层限制 Host、Origin、body 大小、速率、并发、授权主体数量、session 数量、idle
时间和续传 event 保留时间。session 与不透明 event cursor 都绑定授权主体。Sora 不保留
旧式独立 HTTP+SSE transport。

## 推荐工作流

1. 列出并打开项目。
2. 读取 summary、revision、diagnostics 与 normalized schema resources。
3. 校验 schema 与 data。
4. 所有 schema、row、project、Excel 修改先调用对应 preview。
5. 审核 diagnostics、changes、affected files 与输入 revision。
6. 使用返回的 `plan_id` 和新的稳定 `idempotency_key` 调用 apply。
7. 运行 manifest 声明的 build，并读取返回的 immutable artifact resource。

Plan 有 TTL，绑定授权主体与 revision。输入 revision 变化后 plan 自动失效。commit 前
取消不会留下持久化修改；进入原子 commit 后要么完成，要么回滚。

## Tools 参考

所有 tool 都拒绝未知字段并返回严格结构化结果。业务失败会设置 MCP `isError: true`，
同时保留统一 diagnostics。每个 tool 都声明 read-only、destructive、idempotent 和
open-world hints。

| Tool | 主要输入 | 行为 |
|---|---|---|
| `sora_server_info` | `{}` | 协议与 workspace 摘要。 |
| `sora_project_list` | `{}` | 在允许的 Roots 内发现项目。 |
| `sora_project_open` | `root_id?`, `relative_manifest?` | 打开项目；缺失选择时可 Elicit。 |
| `sora_project_inspect` | `project_id` | 查看 schema identity、group、view、source、target 与 format。 |
| `sora_project_init` | `root_id`, `relative_directory`, `schema_id` | 预览新项目 scaffold。 |
| `sora_project_init_apply` | `plan_id`, `idempotency_key` | 原子创建 scaffold。 |
| `sora_schema_validate` | `project_id` | normalize 并校验 schema。 |
| `sora_schema_search` | `project_id` 加 kind/name/field/type/reference 过滤 | 搜索规范化实体。 |
| `sora_schema_preview` | `project_id`、期望 schema/manifest revision、`operations` | 生成 schema plan 与 text diff。 |
| `sora_schema_apply` | `project_id`, `plan_id`, `idempotency_key` | 原子应用 schema plan。 |
| `sora_data_validate` | `project_id`、可选 `view` 与 `tables` | 校验项目数据。 |
| `sora_table_query` | `project_id`, `table` 及可选 filter/key/index/select/order/cursor/limit/locale | 查询类型化数据；limit 为 1–500。 |
| `sora_data_diff` | `project_id`, `other_data_root` | 比较项目内 baseline。 |
| `sora_data_preview` | `project_id`、期望 schema/data revision、`operations` | 预览 row 与 localization 变更。 |
| `sora_data_apply` | `project_id`, `plan_id`, `idempotency_key` | 原子应用 data plan。 |
| `sora_excel_sync_preview` | `project_id`、期望 schema/data revision | 预览 workbook 同步。 |
| `sora_excel_sync_apply` | `project_id`, `plan_id`, `idempotency_key` | 原子同步 XLSX。 |
| `sora_build` | `project_id`、project revision、可选 view/output/target/format/clean | 执行声明的 build graph。 |
| `sora_codegen` | `project_id`, revision, `target`、可选 view/clean | 执行一个 generator。 |
| `sora_export` | `project_id`, revision, `format`、可选 view/clean | 执行一个 exporter。 |
| `sora_schema_lock` | `project_id`, revision、可选 view/clean | 构建 schema lock。 |
| `sora_excel_template` | `project_id`, revision、可选 view/clean | 构建 Excel template。 |

build、codegen、export、schema-lock、Excel-template 与 Excel sync 支持可选 MCP Tasks。
客户端不支持 Tasks 时仍可通过普通 tool call、progress 与 cancellation 完成。

## Resources 参考

固定资源：

- `sora://server/info`
- `sora://workspace/projects`
- `sora://project/{project_id}/summary`
- `sora://project/{project_id}/manifest`
- `sora://project/{project_id}/capabilities`
- `sora://project/{project_id}/schema`
- `sora://project/{project_id}/diagnostics`
- `sora://project/{project_id}/revision`

模板资源：

- `sora://project/{project_id}/schema/{kind}/{name}`
- `sora://project/{project_id}/table/{table}/schema`
- `sora://project/{project_id}/table/{table}/rows?cursor=&limit=&select=`
- `sora://project/{project_id}/artifact/{artifact_id}`
- `sora://project/{project_id}/task/{task_id}`
- `sora://docs/overview` 与 `sora://docs/safety`

row cursor 绑定 revision；artifact 与 task 有 TTL，并绑定授权主体和 project。

## Prompts 与 Completion

标准 Prompts：

- `sora_create_table`
- `sora_add_field_with_migration`
- `sora_rename_entity_safely`
- `sora_fix_validation_errors`
- `sora_add_codegen_target`
- `sora_prepare_config_release`
- `sora_review_schema`

每个 Prompt 都要求 `project_id`，嵌入有界 project summary，并链接 schema 与 diagnostics。
Completion 覆盖 project、entity kind/name、table、field、enum、group、view、source、codegen
target、runtime format、export format、locale、artifact 和 mode。

## 安全与 trust

- Roots 是显式 capability；所有路径先 canonicalize，再检查 traversal 与外部 symlink。
- cell、comment、schema、prompt 和文档内容都是不可信数据，永远不会作为命令执行。
- Lua trust 精确绑定 script path 与 SHA-256 digest，并按授权主体隔离。
- formatter 只能来自内置 allowlist，使用参数数组，不经过 shell。
- `clean` 只能删除 manifest 声明且位于安全 output root 内的 generated outputs。
- mutation plan 绑定 authorization、project、revision、TTL 与 idempotency。
- audit event 记录 tool、project、授权指纹、revision、结果、耗时和有界 change count；
  不记录 bearer token 与请求 body。

## MCP Inspector 操作

[MCP Inspector](https://github.com/modelcontextprotocol/inspector) 需要当前 Node.js。

```bash
cargo build -p sora-cli
npx @modelcontextprotocol/inspector \
  target/debug/sora mcp \
  --project "$(pwd)/examples/showcase/project.toml"
```

在 UI 中依次：

1. 确认协议为 `2025-11-25`，查看 Tools、Resources 与 Prompts。
2. 调用 `sora_project_inspect`。
3. 读取项目的 `revision`、`schema` 和 `diagnostics`。
4. 对 `Item` 表调用 `sora_table_query`，设置 `limit: 2`。
5. 调用 preview tool，确认文件没有变化。
6. 只在临时项目应用 plan，并用同一 idempotency key 重试。
7. 启动 build，观察 progress、可选 Tasks 与 artifact links。

CLI smoke test：

```bash
npx @modelcontextprotocol/inspector --cli \
  target/debug/sora mcp --project "$(pwd)/examples/showcase/project.toml" \
  --method tools/list
```

本地 Streamable HTTP 要允许 Inspector Origin：

```bash
sora mcp --transport http \
  --allowed-origin http://localhost:6274
npx @modelcontextprotocol/inspector
```

在 Inspector 选择 `Streamable HTTP`，连接 `http://127.0.0.1:8000/mcp`。

## Showcase 闭环验证

```bash
cargo test -p sora-mcp --test read_only_workflow
cargo test -p sora-mcp --test build_workflow
```

第一个测试打开 showcase、检查项目、查询 Excel-backed validated data，并确认 revision
不变化；第二个测试执行事务 build、接收 progress，并通过 MCP 读取 immutable artifact。

事务写入会在替换或删除已有文件前，将旧文件备份到 `.sora/backups/<transaction-id>/`。该目录会自动从 Git 中忽略，并且只保留最近 20 个已完成备份批次；仍在执行或可恢复事务所需的备份不会被清理。
