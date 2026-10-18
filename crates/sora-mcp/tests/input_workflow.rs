//! Raw MRTR rounds intentionally use a new transport for every request, like HTTP oneshot.
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::process::Command;
use std::{
    env::temp_dir,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{Ok as anyhow_ok, Result, anyhow};
use rmcp::ServiceExt;
use serde_json::{Value, from_str, json};
use sora_mcp::SoraMcpServer;
use sora_workspace::WorkspaceService;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, duplex},
    spawn,
    time::timeout,
};
use url::Url;
use uuid::Uuid;

async fn raw_request(
    server: &SoraMcpServer,
    method: &str,
    mut params: Value,
    caps: Value,
) -> Result<Value> {
    params["_meta"] = json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientCapabilities": caps
    });
    let id = Uuid::new_v4().to_string();
    let request = json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params});
    let (server_transport, mut client_transport) = duplex(128 * 1024);
    let server = server.clone();
    let handle = spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow_ok(())
    });
    client_transport
        .write_all(format!("{request}\n").as_bytes())
        .await?;
    let mut reader = BufReader::new(client_transport);
    let response: Value = timeout(Duration::from_secs(5), async {
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            let response: Value = from_str(&line)?;
            if response.get("id").is_some() {
                break anyhow_ok(response);
            }
            assert!(
                response["method"]
                    .as_str()
                    .is_some_and(|method| method.starts_with("notifications/")),
                "only ordinary notifications may precede the result: {response}"
            );
        }
    })
    .await??;
    assert_eq!(
        response["id"], id,
        "must receive a response, never a reverse request: {response}"
    );
    assert!(
        response.get("method").is_none(),
        "unexpected reverse request: {response}"
    );
    drop(reader);
    timeout(Duration::from_secs(5), handle).await???;
    Ok(response)
}

fn capabilities() -> Value {
    json!({"roots":{}, "elicitation":{"form":{}}})
}

fn retry(original: &Value, response: &Value, key: &str, value: Value) -> Value {
    assert_eq!(
        response["result"]["resultType"], "input_required",
        "{response}"
    );
    assert!(
        response["result"]["inputRequests"][key].is_object(),
        "{response}"
    );
    let mut request = original.clone();
    request["requestState"] = response["result"]["requestState"].clone();
    request["inputResponses"] = json!({key:value});
    request
}

fn project(scripted: bool) -> Result<PathBuf> {
    let root = temp_dir().join(format!("sora-input-{}", Uuid::new_v4()));
    fs::create_dir_all(&root)?;
    fs::write(
        root.join("project.toml"),
        format!(
            "project = {{ id = \"demo\" }}\ngroups = {{ common = {{ default = true }} }}\nviews = {{ default = {{ contract = \"demo/default\", groups = [\"common\"] }} }}\nincludes = [\"schema.toml\"]\n{}",
            if scripted {
                "[parsers]\nscripts = [\"parser.lua\"]\n"
            } else {
                ""
            }
        ),
    )?;
    fs::write(
        root.join("schema.toml"),
        "[tables.Settings]\nid = \"settings\"\nmode = \"singleton\"\n[tables.Settings.fields]\nname = \"string\"\n",
    )?;
    if scripted {
        fs::write(
            root.join("parser.lua"),
            "return { parsers = { identity = { parse = function(cell) return cell.text end } } }\n",
        )?;
    }
    Ok(root)
}

async fn roots_retry(server: &SoraMcpServer, original: &Value, root: &PathBuf) -> Result<Value> {
    let response = raw_request(server, "tools/call", original.clone(), capabilities()).await?;
    assert_eq!(
        response["result"]["inputRequests"]["roots"],
        json!({"method":"roots/list"})
    );
    Ok(retry(
        original,
        &response,
        "roots",
        json!({"roots":[{
            "uri":Url::from_directory_path(root).map_err(|()| anyhow!("root URL"))?.to_string(), "name":"test"
        }]}),
    ))
}

#[tokio::test]
async fn roots_selection_and_trust_complete_across_independent_raw_requests() -> Result<()> {
    let root = project(true)?;
    let shared = Arc::new(WorkspaceService::new());
    let server = SoraMcpServer::new(shared.clone());
    let other = SoraMcpServer::new(shared.clone());
    let original = json!({"name":"sora_project_open", "arguments":{}});
    let request = roots_retry(&server, &original, &root).await?;
    let denied = raw_request(&other, "tools/call", request.clone(), capabilities()).await?;
    assert_eq!(denied["error"]["code"], -32602);
    let mut tampered = request.clone();
    tampered["arguments"] = json!({"root_id":"forged"});
    assert_eq!(
        raw_request(&server, "tools/call", tampered, capabilities()).await?["error"]["code"],
        -32602
    );
    assert_eq!(
        raw_request(&server, "tools/call", request.clone(), json!({})).await?["error"]["code"],
        -32602
    );
    let response = raw_request(&server, "tools/call", request.clone(), capabilities()).await?;
    assert_eq!(
        response["result"]["inputRequests"]["project_selection"]["method"],
        "elicitation/create"
    );
    // Single-use continuation: the accepted roots response cannot be replayed.
    assert_eq!(
        raw_request(&server, "tools/call", request, capabilities()).await?["error"]["code"],
        -32602
    );
    let lookup = WorkspaceService::new();
    let root_id = lookup.add_root("test", &root)?.id().to_owned();
    let request = retry(
        &original,
        &response,
        "project_selection",
        json!({
            "action":"accept", "content":{"root_id":root_id, "relative_manifest":"project.toml"}
        }),
    );
    let response = raw_request(&server, "tools/call", request, capabilities()).await?;
    let message = response["result"]["inputRequests"]["script_trust"]["params"]["message"]
        .as_str()
        .unwrap();
    assert!(message.contains("SHA-256") && message.contains("parser.lua"));
    let mut forged = retry(
        &original,
        &response,
        "script_trust",
        json!({"action":"accept", "content":{"trust_project_scripts":true}}),
    );
    forged["inputResponses"] =
        json!({"project_selection":{"action":"accept", "content":{"trust_project_scripts":true}}});
    assert_eq!(
        raw_request(&server, "tools/call", forged, capabilities()).await?["error"]["code"],
        -32602
    );
    assert!(shared.project_ids()?.is_empty());
    let request = retry(
        &original,
        &response,
        "script_trust",
        json!({"action":"accept", "content":{"trust_project_scripts":true}}),
    );
    let response = raw_request(&server, "tools/call", request, capabilities()).await?;
    assert_eq!(response["result"]["isError"], false, "{response}");
    assert_eq!(shared.project_ids()?.len(), 1);
    assert!(
        shared.discover_projects()?.is_empty(),
        "client roots must not enter the shared root registry"
    );
    let listed = raw_request(
        &other,
        "tools/call",
        json!({"name":"sora_project_list", "arguments":{}}),
        json!({}),
    )
    .await?;
    assert_eq!(
        listed["result"]["structuredContent"]["data"]["projects"],
        json!([])
    );
    let resource = raw_request(
        &other,
        "resources/read",
        json!({"uri":"sora://workspace/projects"}),
        json!({}),
    )
    .await?;
    let projects: Value = from_str(resource["result"]["contents"][0]["text"].as_str().unwrap())?;
    assert_eq!(projects, json!([]));
    let denied = raw_request(&other, "tools/call", json!({"name":"sora_project_open", "arguments":{"root_id":root_id,"relative_manifest":"project.toml"}}), json!({})).await?;
    assert_eq!(denied["result"]["isError"], true);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[tokio::test]
async fn unsolicited_input_cannot_grant_trust_and_changed_script_input_is_rejected() -> Result<()> {
    let root = project(true)?;
    let shared = Arc::new(WorkspaceService::new());
    let root_id = shared.add_root("test", &root)?.id().to_owned();
    let server = SoraMcpServer::new(shared.clone());
    let original = json!({"name":"sora_project_open", "arguments":{"root_id":root_id,"relative_manifest":"project.toml"}});
    let mut forged = original.clone();
    forged["inputResponses"] =
        json!({"script_trust":{"action":"accept", "content":{"trust_project_scripts":true}}});
    let caps = json!({"elicitation":{"form":{}}});
    assert_eq!(
        raw_request(&server, "tools/call", forged, caps.clone()).await?["error"]["code"],
        -32602
    );
    let response = raw_request(&server, "tools/call", original.clone(), caps.clone()).await?;
    fs::write(
        root.join("parser.lua"),
        "-- changed after the prompt\nreturn { parsers = { identity = { parse = function(cell) return cell.text end } } }\n",
    )?;
    let request = retry(
        &original,
        &response,
        "script_trust",
        json!({"action":"accept", "content":{"trust_project_scripts":true}}),
    );
    let result = raw_request(&server, "tools/call", request, caps.clone()).await?;
    assert_eq!(result["result"]["isError"], true, "{result}");
    assert!(shared.project_ids()?.is_empty());
    // Cancellation is a completed input attempt, not a suspended old request.
    let response = raw_request(&server, "tools/call", original.clone(), caps.clone()).await?;
    let cancelled = retry(
        &original,
        &response,
        "script_trust",
        json!({"action":"cancel"}),
    );
    assert_eq!(
        raw_request(&server, "tools/call", cancelled.clone(), caps.clone()).await?["result"]["isError"],
        true
    );
    assert_eq!(
        raw_request(&server, "tools/call", cancelled, caps.clone()).await?["error"]["code"],
        -32602
    );
    let fresh = raw_request(&server, "tools/call", original.clone(), caps.clone()).await?;
    assert_ne!(
        fresh["result"]["requestState"],
        response["result"]["requestState"]
    );
    let request = retry(
        &original,
        &fresh,
        "script_trust",
        json!({"action":"accept", "content":{"trust_project_scripts":true}}),
    );
    assert_eq!(
        raw_request(&server, "tools/call", request, caps).await?["result"]["isError"],
        false
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[tokio::test]
async fn concurrent_trust_cannot_turn_a_cancelled_input_into_execution() -> Result<()> {
    let root = project(true)?;
    let shared = Arc::new(WorkspaceService::new());
    let root_id = shared.add_root("test", &root)?.id().to_owned();
    let server = SoraMcpServer::new(shared);
    let original = json!({"name":"sora_project_open", "arguments":{"root_id":root_id,"relative_manifest":"project.toml"}});
    let caps = json!({"elicitation":{"form":{}}});
    let accepted = raw_request(&server, "tools/call", original.clone(), caps.clone()).await?;
    let cancelled = raw_request(&server, "tools/call", original.clone(), caps.clone()).await?;
    let request = retry(
        &original,
        &accepted,
        "script_trust",
        json!({"action":"accept", "content":{"trust_project_scripts":true}}),
    );
    assert_eq!(
        raw_request(&server, "tools/call", request, caps.clone()).await?["result"]["isError"],
        false
    );
    let request = retry(
        &original,
        &cancelled,
        "script_trust",
        json!({"action":"cancel"}),
    );
    assert_eq!(
        raw_request(&server, "tools/call", request, caps).await?["result"]["isError"],
        true
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(any(unix, windows))]
fn link_directory(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    symlink(target, link)?;
    #[cfg(windows)]
    {
        // Directory junctions do not require the Windows symlink privilege.
        let output = Command::new("cmd.exe")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()?;
        assert!(
            output.status.success(),
            "failed to create junction: {output:?}"
        );
    }
    Ok(())
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn discovery_skips_an_escaping_directory_link_but_open_still_rejects_it() -> Result<()> {
    let root = project(false)?;
    let outside = project(false)?;
    let link = root.join("escape");
    link_directory(&outside, &link)?;
    assert!(link.join("project.toml").is_file());
    let shared = Arc::new(WorkspaceService::new());
    let server = SoraMcpServer::new(shared.clone());
    let list = json!({"name":"sora_project_list", "arguments":{}});
    let request = roots_retry(&server, &list, &root).await?;
    let listed = raw_request(&server, "tools/call", request, capabilities()).await?;
    assert_eq!(listed["result"]["isError"], false, "{listed}");
    let candidates = listed["result"]["structuredContent"]["data"]["projects"]
        .as_array()
        .unwrap();
    assert_eq!(candidates.len(), 1, "{listed}");
    assert_eq!(candidates[0]["relative_manifest"], "project.toml");
    let open = json!({"name":"sora_project_open", "arguments":{
        "root_id":candidates[0]["root_id"], "relative_manifest":"escape/project.toml"
    }});
    let request = roots_retry(&server, &open, &root).await?;
    let rejected = raw_request(&server, "tools/call", request, capabilities()).await?;
    assert_eq!(rejected["result"]["isError"], true, "{rejected}");
    assert!(shared.project_ids()?.is_empty());
    assert!(outside.join("project.toml").is_file());
    fs::remove_dir(&link)?;
    fs::remove_dir_all(root)?;
    fs::remove_dir_all(outside)?;
    Ok(())
}

#[tokio::test]
async fn client_root_init_keys_conflict_across_plans_and_root_responses() -> Result<()> {
    for different_roots in [false, true] {
        let root = project(false)?;
        let other_root = if different_roots {
            project(false)?
        } else {
            root.clone()
        };
        let shared = Arc::new(WorkspaceService::new());
        let server = SoraMcpServer::new(shared.clone());
        let mut plans = Vec::new();
        for (index, plan_root) in [&root, &other_root].into_iter().enumerate() {
            let root_id = WorkspaceService::new()
                .add_root("test", plan_root)?
                .id()
                .to_owned();
            let preview = json!({"name":"sora_project_init", "arguments":{
                "root_id":root_id, "relative_directory":format!("created-{index}"), "project_id":format!("created-{index}")
            }});
            let request = roots_retry(&server, &preview, plan_root).await?;
            let response = raw_request(&server, "tools/call", request, capabilities()).await?;
            assert_eq!(response["result"]["isError"], false, "{response}");
            plans.push(response["result"]["structuredContent"]["data"]["plan_id"].clone());
        }
        assert_ne!(plans[0], plans[1]);
        let apply = |plan: &Value, key: &str| {
            json!({"name":"sora_project_init_apply", "arguments":{
                "plan_id":plan, "idempotency_key":key
            }})
        };
        let first = raw_request(
            &server,
            "tools/call",
            apply(&plans[0], "one-owner-key"),
            json!({}),
        )
        .await?;
        assert_eq!(first["result"]["isError"], false, "{first}");
        let repeated = raw_request(
            &server,
            "tools/call",
            apply(&plans[0], "one-owner-key"),
            json!({}),
        )
        .await?;
        assert_eq!(
            first["result"]["structuredContent"],
            repeated["result"]["structuredContent"]
        );
        let conflict = raw_request(
            &server,
            "tools/call",
            apply(&plans[1], "one-owner-key"),
            json!({}),
        )
        .await?;
        assert_eq!(
            conflict["result"]["isError"], true,
            "same authorization must not create a second plan with a reused key (different roots: {different_roots}): {conflict}"
        );
        assert!(
            conflict["result"]["structuredContent"]["diagnostics"][0]["message"]
                .as_str()
                .unwrap()
                .contains("idempotency key was already used for another plan")
        );
        assert!(!other_root.join("created-1").exists());
        assert_eq!(shared.project_ids()?.len(), 1);
        let second = raw_request(
            &server,
            "tools/call",
            apply(&plans[1], "another-owner-key"),
            json!({}),
        )
        .await?;
        assert_eq!(second["result"]["isError"], false, "{second}");
        assert!(other_root.join("created-1").is_dir());
        fs::remove_dir_all(&root)?;
        if different_roots {
            fs::remove_dir_all(other_root)?;
        }
    }
    Ok(())
}

#[tokio::test]
async fn configured_and_client_root_plans_share_the_same_idempotency_namespace() -> Result<()> {
    let configured = project(false)?;
    let client_root = project(false)?;
    let shared = Arc::new(WorkspaceService::new());
    let configured_root = shared.add_root("configured", &configured)?;
    let authority = WorkspaceService::new().add_root("client", &client_root)?;
    let server = SoraMcpServer::new(shared.clone());
    let first = raw_request(
        &server,
        "tools/call",
        json!({"name":"sora_project_init", "arguments":{
            "root_id":configured_root.id(),"relative_directory":"first","project_id":"first"
        }}),
        json!({}),
    )
    .await?;
    assert_eq!(first["result"]["isError"], false, "{first}");
    let first_plan = first["result"]["structuredContent"]["data"]["plan_id"].clone();
    let preview = json!({"name":"sora_project_init", "arguments":{
        "root_id":authority.id(),"relative_directory":"second","project_id":"second"
    }});
    let request = roots_retry(&server, &preview, &client_root).await?;
    let second = raw_request(&server, "tools/call", request, capabilities()).await?;
    assert_eq!(second["result"]["isError"], false, "{second}");
    let second_plan = second["result"]["structuredContent"]["data"]["plan_id"].clone();
    let apply = |plan: &Value, key: &str| {
        json!({"name":"sora_project_init_apply", "arguments":{
            "plan_id":plan,"idempotency_key":key
        }})
    };
    assert_eq!(
        raw_request(
            &server,
            "tools/call",
            apply(&first_plan, "shared-key"),
            json!({})
        )
        .await?["result"]["isError"],
        false
    );
    let conflict = raw_request(
        &server,
        "tools/call",
        apply(&second_plan, "shared-key"),
        json!({}),
    )
    .await?;
    assert_eq!(conflict["result"]["isError"], true, "{conflict}");
    assert!(
        conflict["result"]["structuredContent"]["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("idempotency key was already used for another plan")
    );
    assert!(!client_root.join("second").exists());
    assert_eq!(shared.project_ids()?.len(), 1);
    assert_eq!(
        raw_request(
            &server,
            "tools/call",
            apply(&second_plan, "separate-key"),
            json!({})
        )
        .await?["result"]["isError"],
        false
    );
    assert_eq!(shared.project_ids()?.len(), 2);
    assert_eq!(
        shared.discover_projects()?.len(),
        2,
        "only the configured root is registered"
    );
    fs::remove_dir_all(configured)?;
    fs::remove_dir_all(client_root)?;
    Ok(())
}

#[tokio::test]
async fn client_root_scaffold_plans_remain_authorized_and_idempotent() -> Result<()> {
    let root = project(false)?;
    let shared = Arc::new(WorkspaceService::new());
    let server = SoraMcpServer::new(shared.clone());
    let root_id = WorkspaceService::new()
        .add_root("test", &root)?
        .id()
        .to_owned();
    let preview = json!({"name":"sora_project_init", "arguments":{
        "root_id":root_id,"relative_directory":"created","project_id":"created"
    }});
    let request = roots_retry(&server, &preview, &root).await?;
    let response = raw_request(&server, "tools/call", request, capabilities()).await?;
    assert_eq!(response["result"]["isError"], false, "{response}");
    let plan = &response["result"]["structuredContent"]["data"]["plan_id"];
    let apply = json!({"name":"sora_project_init_apply", "arguments":{"plan_id":plan,"idempotency_key":"create-once"}});
    let catalog = WorkspaceService::new();
    let authority = catalog.add_root("foreign-owner", &root)?;
    let foreign_plan =
        shared.preview_project_init_in_root("foreign-owner", authority, "denied", "demo")?;
    drop(catalog);
    let denied = json!({"name":"sora_project_init_apply", "arguments":{
        "plan_id":foreign_plan.plan_id,"idempotency_key":"create-once"
    }});
    assert_eq!(
        raw_request(&server, "tools/call", denied, json!({})).await?["result"]["isError"],
        true
    );
    assert!(!root.join("denied").exists());
    // Separate server instances with the same owner share one coordinator.
    // The plan snapshot, not the latest roots response, authorizes this apply.
    let same_owner = SoraMcpServer::new(shared.clone());
    let response = raw_request(&same_owner, "tools/call", apply.clone(), json!({})).await?;
    assert_eq!(response["result"]["isError"], false, "{response}");
    let repeated = raw_request(&server, "tools/call", apply, json!({})).await?;
    assert_eq!(
        response["result"]["structuredContent"],
        repeated["result"]["structuredContent"]
    );
    assert_eq!(shared.project_ids()?.len(), 1);
    fs::remove_dir_all(root)?;
    Ok(())
}
