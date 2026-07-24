# Model Context Protocol

Sora includes an MCP `2025-11-25` server for AI clients. It exposes the same
project, schema, data, Excel, and build services used by the CLI and Studio.
Generated files are outputs, never mutation inputs.

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
that declares Lua parser or type-mapping scripts remains untrusted unless the
client completes the form Elicitation flow or the local command explicitly
includes `--trust-project-scripts`.

Clients that provide MCP Roots can omit `--project`. Use `sora_project_list`
and `sora_project_open`; clients without form Elicitation must pass both
`root_id` and `relative_manifest` explicitly.

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

Streamable HTTP validates Host and Origin, bounds request size, request rate,
concurrency, authorization contexts, sessions, idle time, and resumable event
retention. Sessions and opaque event cursors are authorization-bound. Sora does
not implement the obsolete standalone HTTP+SSE transport.

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
| `sora_project_inspect` | `project_id` | Sources, scopes, targets, formats, and package summary. |
| `sora_project_init` | `root_id`, `relative_directory`, `package` | Preview a new scaffold. |
| `sora_project_init_apply` | `plan_id`, `idempotency_key` | Atomically create the previewed scaffold. |
| `sora_schema_validate` | `project_id` | Normalize and validate schema. |
| `sora_schema_search` | `project_id` plus kind/name/field/type/reference filters | Search normalized entities. |
| `sora_schema_preview` | `project_id`, expected schema/manifest revisions, ordered `operations` | Produce an immutable schema plan and text diffs. |
| `sora_schema_apply` | `project_id`, `plan_id`, `idempotency_key` | Atomically apply a schema plan. |
| `sora_data_validate` | `project_id`, optional `scope`, `tables` | Validate selected or all project data. |
| `sora_table_query` | `project_id`, `table`, optional filters/key/index/select/order/cursor/limit/locale | Query validated typed rows; limit is 1–500. |
| `sora_data_diff` | `project_id`, `other_data_root` | Compare a project-relative baseline root. |
| `sora_data_preview` | `project_id`, expected schema/data revisions, ordered `operations` | Preview typed row and localization changes. |
| `sora_data_apply` | `project_id`, `plan_id`, `idempotency_key` | Atomically apply a data plan. |
| `sora_excel_sync_preview` | `project_id`, expected schema/data revisions | Preview workbook synchronization. |
| `sora_excel_sync_apply` | `project_id`, `plan_id`, `idempotency_key` | Atomically synchronize XLSX workbooks. |
| `sora_build` | `project_id`, expected project revision, optional scope/output groups/targets/formats/clean | Run the declared build graph. |
| `sora_codegen` | `project_id`, revision, `target`, optional scope/clean | Run one declared generator target. |
| `sora_export` | `project_id`, revision, `format`, optional scope/clean | Run one declared exporter. |
| `sora_schema_lock` | `project_id`, revision, optional scope/clean | Build the declared schema lock. |
| `sora_excel_template` | `project_id`, revision, optional scope/clean | Build declared Excel templates. |

Build, codegen, export, schema-lock, Excel-template, and Excel synchronization
advertise optional MCP Tasks. Clients without Tasks receive the same operation
as a normal tool call with progress and cancellation.

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
supports project, entity kind/name, table, field, enum, scope, source, codegen
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

1. Confirm protocol `2025-11-25` and inspect the Tools, Resources, and Prompts
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
