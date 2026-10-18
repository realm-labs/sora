# Model Context Protocol

Sora supports only MCP `2026-07-28` for AI clients. It exposes the same
project, schema, data, Excel, and build services used by the CLI and Studio.
Generated files are outputs, never mutation inputs. Clients use `server/discover`
and self-contained per-request protocol metadata, not the legacy initialize handshake.

## Install and connect

Build or install the `sora` binary from this repository:

```bash
cargo install --path crates/sora-cli
```

For a local client, prefer stdio and use absolute paths:

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

The server writes only JSON-RPC to stdout. Audit logs go to stderr. A project
that declares Lua parser, type-mapping, or source-loader scripts remains untrusted
unless the client completes the form Elicitation flow or the local command explicitly
includes `--trust-project-scripts`.

Script digests and execution use the same immutable source snapshot. The manifest's
script declarations are captured with it; opening an authorized snapshot never
reloads executable scripts from disk. Changes before approval require a new trust
round, and changes after authorization cannot replace the code being opened.
An existing session is reused only when its loaded manifest, scripts, and runtime
options match the authorized snapshot; otherwise opening fails explicitly.

Clients that provide MCP Roots can omit `--project`. Use `sora_project_list`
and `sora_project_open`; clients without form Elicitation must pass both
`root_id` and `relative_manifest` explicitly.

## Client input and retries

Roots, project selection, and script trust use MCP `2026-07-28` multi-round-trip
input, on both stdio and HTTP. Sora returns `resultType: "input_required"`; it
never waits for a reverse `roots/list` or `elicitation/create` JSON-RPC request.
The client fulfils the embedded `inputRequests` locally, then sends a **new**
`tools/call` with the same tool, arguments, and capability declarations, the
server's opaque `requestState`, and only the responses requested in that round.
Project opening may require three rounds: roots, selection, then script trust.
Without Roots support, only configured server roots are available. Without form
Elicitation, selection must be explicit and untrusted scripts remain refused.

For example, a Roots-capable client's initial request is:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"sora_project_list","arguments":{},"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{"roots":{}}}}}
```

Sora returns the following shape (`requestState` is an illustrative opaque handle):

```json
{"jsonrpc":"2.0","id":1,"result":{"resultType":"input_required","inputRequests":{"roots":{"method":"roots/list"}},"requestState":"7b5779e8-fddd-4d34-8582-415ab87baf3e"}}
```

The client retries, rather than POSTing a standalone JSON-RPC response:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"sora_project_list","arguments":{},"requestState":"7b5779e8-fddd-4d34-8582-415ab87baf3e","inputResponses":{"roots":{"roots":[{"uri":"file:///work/project","name":"project"}]}},"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{"roots":{}}}}}
```

Selection and trust inputs have method `elicitation/create`, form `params`, and
response values such as `{"action":"accept","content":{"root_id":"root-…","relative_manifest":"project.toml"}}`
and `{"action":"accept","content":{"trust_project_scripts":true}}`. Each uses
its own server-issued input key (`project_selection` or `script_trust`) and the
latest handle. `decline` or `cancel` completes that attempt without opening the
project; trying again starts a new request without previous input state.

The [rmcp input model](https://docs.rs/rmcp/3.5.1/rmcp/model/struct.InputRequiredResult.html)
and [client implementation](https://docs.rs/crate/rmcp/3.5.1/source/src/service/client.rs)
provide native support: `RunningService::call_tool` automatically invokes local
`ClientHandler::list_roots` / `create_elicitation` handlers and retries (default
maximum ten rounds). `call_tool_once` exposes each intermediate response. Clients
advertising Tasks must drive these input rounds with `call_tool_once`, then poll
the returned task; the high-level `call_tool` helper does not poll Tasks.

Input handles are authorization-server-local, single-use, and expire after five
minutes. Sora retains at most 64 pending continuations, each with a 256 KiB state
limit; client roots are limited to 64. Retries are checked against the original
tool, arguments, capabilities, input kind/id, and exact requested content,
including script paths and digests. Client roots never enter the shared workspace
root registry. A pending input is not a successful tool execution or an accepted
task; no worker starts until all required input is complete.

## Streamable HTTP

Local HTTP:

```bash
sora mcp --transport http --host 127.0.0.1 --port 8000
```

Connect to `http://127.0.0.1:8000/mcp`. The default allowed browser Origin is
the public URL's origin. Add each other exact browser origin explicitly:

```bash
sora mcp --transport http \
  --host 127.0.0.1 \
  --port 8000 \
  --allowed-origin http://localhost:6274
```

Non-loopback listeners require HTTPS as the public URL and an OAuth resource
server configuration:

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

Sora discovers the issuer's JWKS through RFC 8414. Use `--oauth-jwks-uri` only
when the deployment requires an explicit endpoint. Access tokens must be
asymmetrically signed JWTs with matching `iss` and `aud`, a subject, an
unexpired `exp`, a valid optional `nbf`, and every configured scope. Protected
resource metadata is available at
`/.well-known/oauth-protected-resource`.

Streamable HTTP is sessionless. It validates Host and Origin and bounds request
size, request rate, concurrency, and retained authorization contexts. Requests
carry `MCP-Protocol-Version: 2026-07-28`, protocol/client capability metadata,
and the standard `Mcp-Method` / `Mcp-Name` routing headers where applicable.
Every request is authorized independently; no MCP session ID grants access.
Sora does not implement the obsolete standalone HTTP+SSE transport.

## Recommended workflow

1. List and open a project.
2. Read its summary, revision, diagnostics, and normalized schema resources.
3. Validate schema and data.
4. For schema, row, project, or Excel changes, call the matching preview tool.
5. Review diagnostics, changes, affected files, and input revisions.
6. Apply the returned `plan_id` with a new stable `idempotency_key`.
7. Build declared outputs and read returned immutable artifact resources.

Plans expire, are authorization-bound, and fail after their input revision
changes. Cancellation before commit leaves no persistent changes. Once atomic
commit begins, Sora completes it or rolls it back.

## Tool reference

Every tool rejects unknown input fields and returns structured output. Business
failures set MCP `isError: true` and preserve diagnostics in the same envelope.
All tools declare read-only, destructive, idempotent, and open-world hints.

| Tool | Main input | Effect |
|---|---|---|
| `sora_server_info` | `{}` | Protocol and workspace summary; read-only. |
| `sora_project_list` | `{}` | Discover manifests inside allowed Roots. |
| `sora_project_open` | `root_id?`, `relative_manifest?` | Open a project; missing selection may use Elicitation. |
| `sora_project_inspect` | `project_id` | Schema identity, groups, views, sources, targets, and formats. |
| `sora_project_init` | `root_id`, `relative_directory`, `schema_id` | Preview a new scaffold. |
| `sora_project_init_apply` | `plan_id`, `idempotency_key` | Atomically create the previewed scaffold. |
| `sora_schema_validate` | `project_id` | Normalize and validate schema. |
| `sora_schema_search` | `project_id` plus kind/name/field/type/reference filters | Search normalized entities. |
| `sora_schema_preview` | `project_id`, expected schema/manifest revisions, ordered `operations` | Produce an immutable schema plan and text diffs. |
| `sora_schema_apply` | `project_id`, `plan_id`, `idempotency_key` | Atomically apply a schema plan. |
| `sora_data_validate` | `project_id`, optional `view`, `tables` | Validate selected or all project data. |
| `sora_table_query` | `project_id`, `table`, optional filters/key/index/select/order/cursor/limit/locale | Query validated typed rows; limit is 1–500. |
| `sora_data_diff` | `project_id`, `other_data_root` | Compare a project-relative baseline root. |
| `sora_data_preview` | `project_id`, expected schema/data revisions, ordered `operations` | Preview typed row and localization changes. |
| `sora_data_apply` | `project_id`, `plan_id`, `idempotency_key` | Atomically apply a data plan. |
| `sora_excel_sync_preview` | `project_id`, expected schema/data revisions | Preview workbook synchronization. |
| `sora_excel_sync_apply` | `project_id`, `plan_id`, `idempotency_key` | Atomically synchronize XLSX workbooks. |
| `sora_build` | `project_id`, expected project revision, optional view/output groups/targets/formats/clean | Run the declared build graph. |
| `sora_codegen` | `project_id`, revision, `target`, optional view/clean | Run one declared generator target. |
| `sora_export` | `project_id`, revision, `format`, optional view/clean | Run one declared exporter. |
| `sora_schema_lock` | `project_id`, revision, optional view/clean | Build the declared schema lock. |
| `sora_excel_template` | `project_id`, revision, optional view/clean | Build declared Excel templates. |

### Long-running operations and Tasks

Sora implements the [`io.modelcontextprotocol/tasks` extension](https://modelcontextprotocol.io/extensions/tasks/overview).
Clients declare it in each request's `_meta.io.modelcontextprotocol/clientCapabilities.extensions`.
For build, codegen, export, schema-lock, Excel-template, and both Excel synchronization
tools, Sora returns a `resultType: "task"` handle when the client supports the extension.
Other tools return their completed result directly. Clients without the extension
execute the same operation synchronously, with normal request progress and cancellation.
There is no legacy task wire adapter, per-tool `execution.taskSupport`, or client `task` metadata.

Poll `tasks/get` with `taskId`, respecting `pollIntervalMs` (250 ms). The response is
a `DetailedTask`: terminal `completed` includes the original tool `result`, including
business failures with `isError: true`; `failed` includes a JSON-RPC `error`.
`tasks/cancel` acknowledges cancellation intent, not completion. A worker that passed
its commit point may still complete successfully. The task resource exposes the same
new task shape and remains authorization- and project-bound.

Execution `ttlMs` starts at task creation (five minutes). The deadline requests real
cooperative cancellation; it does not abort an atomic commit or fabricate a terminal
status. Completed records are retained for up to another minute, with an updated
`ttlMs`. Each store permits eight active tasks per authorization owner and at most
64 records across all owners; older terminal records may be evicted to admit work.
Tasks are in-memory and do not survive a server restart.

Sora tasks do not request client input: `tasks/update` returns an explicit error.
Observe tasks by polling; task push notifications and task progress notifications
are not provided. `tasks/list`, `tasks/result`, and client-side task TTL requests
are not supported. Roots, project selection, and script trust complete through
`input_required` / `inputResponses` retries before task acceptance, never from a
worker or through task input requests.

Resource change notifications use `subscriptions/listen`; clients select resource
URIs and cancel that listen request to unsubscribe.

## Resource reference

Fixed resources:

- `sora://server/info`
- `sora://workspace/projects`
- `sora://project/{project_id}/summary`
- `sora://project/{project_id}/manifest`
- `sora://project/{project_id}/capabilities`
- `sora://project/{project_id}/schema`
- `sora://project/{project_id}/diagnostics`
- `sora://project/{project_id}/revision`

Templates:

- `sora://project/{project_id}/schema/{kind}/{name}`
- `sora://project/{project_id}/table/{table}/schema`
- `sora://project/{project_id}/table/{table}/rows?cursor=&limit=&select=`
- `sora://project/{project_id}/artifact/{artifact_id}`
- `sora://project/{project_id}/task/{task_id}`
- `sora://docs/overview` and `sora://docs/safety`

Row cursors are revision-bound. Artifacts and tasks have TTLs and are readable
only by their authorization context and project.

## Prompt and Completion reference

The seven guided prompts are:

- `sora_create_table`
- `sora_add_field_with_migration`
- `sora_rename_entity_safely`
- `sora_fix_validation_errors`
- `sora_add_codegen_target`
- `sora_prepare_config_release`
- `sora_review_schema`

Every prompt requires `project_id`, embeds a bounded project summary, links the
schema and diagnostics resources, and preserves preview/apply safety. Completion
supports project, entity kind/name, table, field, enum, group, view, source, codegen
target, runtime format, export format, locale, artifact, and mode arguments.

## Security and trust

- Roots are explicit capabilities. Paths are root-relative, canonicalized, and
  checked against traversal and external symlinks.
- Project cells, comments, schema text, prompt content, and documentation are
  untrusted data and never commands.
- Lua trust is recorded for the exact script paths and SHA-256 digests in one
  authorization context. Another subject must make an independent decision.
- Formatters use a built-in executable allowlist and argument arrays, never a
  shell command string.
- `clean` deletes only manifest-declared generated outputs inside validated
  output roots.
- Mutation plans bind authorization, project, revision, TTL, and idempotency.
- Audit events contain tool, project, authorization fingerprint, revisions,
  outcome, duration, and bounded change counts; bearer tokens and bodies are
  never logged.

## MCP Inspector walkthrough

[MCP Inspector](https://github.com/modelcontextprotocol/inspector) requires a
current Node.js release.

Build Sora, then inspect stdio:

```bash
cargo build -p sora-cli
npx @modelcontextprotocol/inspector \
  target/debug/sora mcp \
  --project "$(pwd)/examples/showcase/project.toml"
```

In the UI:

1. Confirm protocol `2026-07-28` and inspect the Tools, Resources, and Prompts
   tabs.
2. Call `sora_project_inspect` for the opened project.
3. Read its `revision`, `schema`, and `diagnostics` resources.
4. Call `sora_table_query` for table `Item` with `limit: 2`.
5. Run a preview tool and verify that no files change.
6. Apply only a disposable plan, then repeat the same idempotency key to verify
   the stored result.
7. Start a build and observe progress, Tasks when available, and artifact links.

CLI smoke checks are also useful:

```bash
npx @modelcontextprotocol/inspector --cli \
  target/debug/sora mcp --project "$(pwd)/examples/showcase/project.toml" \
  --method tools/list

npx @modelcontextprotocol/inspector --cli \
  target/debug/sora mcp --project "$(pwd)/examples/showcase/project.toml" \
  --method resources/list
```

For local Streamable HTTP, start Sora with Inspector's Origin:

```bash
sora mcp --transport http \
  --allowed-origin http://localhost:6274

npx @modelcontextprotocol/inspector
```

Select `Streamable HTTP` and enter `http://127.0.0.1:8000/mcp`.

## Showcase verification

The checked-in showcase is the recommended read-only end-to-end project:

```bash
cargo test -p sora-mcp --test read_only_workflow
cargo test -p sora-mcp --test build_workflow
```

The first test opens the showcase, inspects it, queries validated Excel-backed
data, and verifies that revisions do not change. The second runs a transactional
build, observes progress, and reads the immutable artifact through MCP.

Transactional writes back up existing targets under `.sora/backups/<transaction-id>/` before replacement or deletion. The directory ignores itself from Git and retains the 20 most recent completed backup sets; backups belonging to active or recoverable transactions are never pruned.
