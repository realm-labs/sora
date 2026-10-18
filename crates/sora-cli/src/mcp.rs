use std::{collections::BTreeSet, io::stderr, net::SocketAddr, path::Path, sync::Arc};

use anyhow::{Context, Result, anyhow, bail};
use sora_mcp::{HttpServerConfig, OAuthResourceServerConfig, serve_http, serve_stdio};
use sora_workspace::{RuntimeOptions, WorkspaceService};
use tracing_subscriber::fmt as tracing_fmt;
use url::Url;

use crate::args::{McpArgs, McpTransportArg};

pub fn run(args: McpArgs, runtime_options: RuntimeOptions) -> Result<()> {
    tracing_fmt()
        .json()
        .with_ansi(false)
        .with_writer(stderr)
        .try_init()
        .map_err(|error| anyhow!("failed to initialize MCP audit logging: {error}"))?;
    let workspace = Arc::new(WorkspaceService::new());
    if let Some(project) = &args.project {
        let root = project.parent().unwrap_or_else(|| Path::new("."));
        let root = workspace.add_root("explicit", root)?;
        let relative_manifest = project
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        workspace.open_discovered_project(
            root.id(),
            relative_manifest,
            runtime_options,
            args.trust_project_scripts,
        )?;
    }
    match args.transport {
        McpTransportArg::Stdio => {
            reject_http_only_arguments(&args)?;
            serve_stdio(workspace)
        }
        McpTransportArg::Http => {
            let config = http_config(&args)?;
            serve_http(workspace, config)
        }
    }
}

fn reject_http_only_arguments(args: &McpArgs) -> Result<()> {
    if args.public_url.is_some()
        || !args.allowed_origin.is_empty()
        || args.oauth_issuer.is_some()
        || args.oauth_audience.is_some()
        || args.oauth_jwks_uri.is_some()
        || !args.oauth_scope.is_empty()
    {
        bail!("HTTP-specific options require --transport http");
    }
    Ok(())
}

fn http_config(args: &McpArgs) -> Result<HttpServerConfig> {
    let bind = SocketAddr::new(args.host, args.port);
    let public_url = match &args.public_url {
        Some(url) => url.clone(),
        None if args.host.is_loopback() => Url::parse(&format!("http://{bind}/mcp"))
            .context("failed to derive local MCP public URL")?,
        None => bail!("--public-url is required for a non-loopback HTTP listener"),
    };
    let mut config = HttpServerConfig::new(bind, public_url);
    if !args.allowed_origin.is_empty() {
        config.allowed_origins.clone_from(&args.allowed_origin);
    }
    config.oauth = match (&args.oauth_issuer, &args.oauth_audience) {
        (Some(issuer), Some(audience)) => {
            let mut oauth = OAuthResourceServerConfig::new(issuer.clone(), audience);
            oauth.jwks_uri.clone_from(&args.oauth_jwks_uri);
            if !args.oauth_scope.is_empty() {
                oauth.required_scopes = args.oauth_scope.iter().cloned().collect::<BTreeSet<_>>();
            }
            Some(oauth)
        }
        (None, None) => None,
        _ => bail!("--oauth-issuer and --oauth-audience must be provided together"),
    };
    config
        .validate()
        .context("invalid HTTP MCP configuration")?;
    Ok(config)
}
