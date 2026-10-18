# 模型上下文协议（MCP）

Sora 仅支持面向 AI 客户端的 MCP `2026-07-28` 协议。MCP、CLI 与 Studio 共用同一套
project、schema、data、Excel 和 build 应用服务；generated code 与 runtime bundle
始终只是输出，不能作为修改入口。客户端使用 `server/discover` 与每次请求自包含的
协议元数据，不再使用旧 initialize 握手。

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

stdout 只包含 JSON-RPC，审计日志写入 stderr。项目声明的 Lua parser、type-mapping
或 source-loader 脚本默认不受信任。只有客户端完成 form Elicitation，或本地启动命令显式加入
`--trust-project-scripts`，Sora 才会执行它们。

脚本摘要与执行使用同一份不可变源码快照，Manifest 的脚本声明也一并捕获。
打开获批快照时不会再从磁盘重新加载可执行脚本；批准前发生的变化需要重新确认，
授权后发生的文件变化不能替换本次执行的代码。只有已加载的 Manifest、脚本及运行选项
与获批快照一致时才会复用已有会话，否则打开操作会明确失败。

支持 MCP Roots 的客户端可以省略 `--project`，随后调用 `sora_project_list` 和
`sora_project_open`。不支持 form Elicitation 的客户端必须显式传入 `root_id` 与
`relative_manifest`。

## 客户端输入与重试

stdio 与 HTTP 均使用 MCP `2026-07-28` 原生多轮输入流程处理 Roots、项目选择和脚本信任。
服务端返回 `resultType: "input_required"`，不再等待反向 `roots/list` 或
`elicitation/create` JSON-RPC 请求。客户端在本地处理内嵌 `inputRequests`，然后发起
**新的** `tools/call`：保持 tool、arguments 与 capability 声明不变，原样回传 opaque
`requestState`，并在 `inputResponses` 中只提供本轮要求的响应。打开项目可能依次需要
Roots、项目选择、脚本信任三轮输入。不支持 Roots 时仅使用服务端配置的目录；不支持
form Elicitation 时必须显式选择项目，且仍拒绝尚未信任的脚本。

例如，客户端首先发送：

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"sora_project_list","arguments":{},"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{"roots":{}}}}}
```

服务端返回以下结构（`requestState` 是示例 opaque handle）：

```json
{"jsonrpc":"2.0","id":1,"result":{"resultType":"input_required","inputRequests":{"roots":{"method":"roots/list"}},"requestState":"7b5779e8-fddd-4d34-8582-415ab87baf3e"}}
```

客户端重新调用原 tool，而不是 POST 一条独立 JSON-RPC response：

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sora_project_list","arguments":{},"requestState":"7b5779e8-fddd-4d34-8582-415ab87baf3e","inputResponses":{"roots":{"roots":[{"uri":"file:///work/project","name":"project"}]}},"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{"roots":{}}}}}
```

项目选择与信任输入的 method 为 `elicitation/create`，`params` 为 form 请求。
响应分别形如 `{"action":"accept","content":{"root_id":"root-…","relative_manifest":"project.toml"}}`
与 `{"action":"accept","content":{"trust_project_scripts":true}}`，使用对应的服务端输入
key（`project_selection` 或 `script_trust`）和最新 handle。`decline` 或 `cancel` 结束
本次尝试且不打开项目；再次尝试应发起不带旧输入状态的新请求。

[rmcp 输入模型](https://docs.rs/rmcp/3.5.1/rmcp/model/struct.InputRequiredResult.html) 与
[客户端实现](https://docs.rs/crate/rmcp/3.5.1/source/src/service/client.rs) 提供原生支持：
`RunningService::call_tool` 自动调用本地 `ClientHandler::list_roots` /
`create_elicitation` 并重试，默认最多十轮；`call_tool_once` 直接暴露每轮结果。
声明 Tasks 的客户端应使用 `call_tool_once` 自行完成输入轮次，再轮询任务；高层
`call_tool` 不会自动轮询 Tasks。

输入 handle 只属于当前授权 server、一次性使用、五分钟过期。每个 store 最多保留
64 个 continuation，每个状态上限 256 KiB，客户端 Roots 最多 64 个。重试核对原始
工具、参数、capabilities、输入 kind/id 与实际请求内容，包括脚本路径及摘要。
客户端 Roots 不进入共享 workspace 根目录注册表。等待输入不代表业务执行成功或
任务已接受；全部必要输入完成前不会启动 worker。

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

Streamable HTTP 无 session；每次请求独立授权，并限制 Host、Origin、body 大小、
速率、并发与保留的授权主体数量。请求携带 `MCP-Protocol-Version: 2026-07-28`、
协议与客户端能力元数据，以及适用的 `Mcp-Method` / `Mcp-Name` 路由头。
MCP session ID 不是授权凭据。Sora 不保留旧式独立 HTTP+SSE transport。

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

### 长操作与 Tasks

Sora 实现 [`io.modelcontextprotocol/tasks` 扩展](https://modelcontextprotocol.io/extensions/tasks/overview)。
客户端在每次请求的 `_meta.io.modelcontextprotocol/clientCapabilities.extensions` 中声明支持。
对 build、codegen、export、schema-lock、Excel-template 与两个 Excel sync tool，服务端在
客户端支持扩展时返回 `resultType: "task"` 句柄；其他 tool 直接返回完成结果。
不支持扩展时同步执行相同操作，保留普通请求的 progress 与 cancellation。
不提供旧任务 wire 适配器、tool `execution.taskSupport` 或客户端 `task` 元数据。

客户端按 `pollIntervalMs`（250 ms）轮询 `tasks/get`，传入 `taskId`。
返回的新 `DetailedTask` 在 `completed` 时内嵌原始 tool `result`，包括 `isError: true`
的业务失败；`failed` 内嵌 JSON-RPC `error`。`tasks/cancel` 只确认取消意图，
不代表 worker 已停止；越过提交点的操作仍可能成功完成。task resource 使用相同的新任务结构，
继续绑定授权主体与 project。

执行 `ttlMs` 从任务创建起算，默认五分钟。deadline 发出真正的协作取消，不中断原子提交，
也不伪造终态。完成记录最多另保留一分钟，并更新 `ttlMs`。每个 store 按 owner 限制
八个活跃任务，所有 owner 合计最多 64 条记录；容量不足时可淘汰最旧的终态记录。
任务只保存在内存中，不跨服务端重启存活。

Sora 任务不要求客户端输入，`tasks/update` 明确返回错误。任务通过轮询观察，
不提供任务 push notification 或任务 progress notification；不支持 `tasks/list`、
`tasks/result` 与客户端指定的 task TTL。Roots、项目选择和脚本信任通过
`input_required` / `inputResponses` 重试，在任务接受前完成；worker 不索取输入，
也不通过任务输入交互实现这些步骤。

资源变更通知使用 `subscriptions/listen`，客户端选择资源 URI，并通过取消 listen 请求退订。

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

1. 确认协议为 `2026-07-28`，查看 Tools、Resources 与 Prompts。
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
