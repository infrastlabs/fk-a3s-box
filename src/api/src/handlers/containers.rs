//! Container API handlers.

use axum::{Json, extract::{Path, Query}, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::error::{ApiResult, ApiError};
use crate::models::ContainerCreateRequest;

/// Query parameters for container list.
#[derive(Debug, Deserialize)]
pub struct ListQuery {
    /// Show all containers (default shows just running)
    #[serde(default)]
    all: bool,

    /// Show n last created containers (includes all states)
    limit: Option<usize>,

    /// Show only containers with given size
    size: Option<bool>,

    /// Filter containers by status, name, etc.
    filters: Option<String>,
}

/// GET /containers/json - List containers.
pub async fn list(Query(query): Query<ListQuery>) -> ApiResult<Json<serde_json::Value>> {
    let state = a3s_box_cli::state::StateFile::load_default()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let mut containers: Vec<_> = state.list(query.all)
        .iter()
        .map(|record| {
            json!({
                "Id": record.id,
                "Names": [format!("/{}", record.name)],
                "Image": record.image,
                "ImageID": "",
                "Command": record.cmd.join(" "),
                "Created": record.created_at.timestamp(),
                "State": record.status,
                "Status": format!("{}", record.status),
                "Ports": [],
                "Labels": record.labels,
                "SizeRw": 0,
                "SizeRootFs": 0,
                "HostConfig": {
                    "NetworkMode": record.network_mode
                },
                "NetworkSettings": {
                    "Networks": {}
                },
                "Mounts": []
            })
        })
        .collect();

    // Apply limit if specified
    if let Some(limit) = query.limit {
        containers.truncate(limit);
    }

    Ok(Json(json!(containers)))
}

/// POST /containers/create - Create a container.
pub async fn create(
    Query(query): Query<HashMap<String, String>>,
    Json(req): Json<ContainerCreateRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    // Extract container name from query parameter
    let name = query.get("name").map(|s| s.as_str());

    // Build a3s-box run command arguments
    let mut run_args = a3s_box_cli::commands::run::RunArgs {
        image: req.image.clone(),
        name: name.map(|s| s.to_string()),
        detach: true, // Always create in detached mode
        rm: req.host_config.as_ref()
            .and_then(|hc| hc.auto_remove)
            .unwrap_or(false),
        interactive: false,
        tty: false,
        env: req.env.unwrap_or_default(),
        volume: vec![], // TODO: Parse from host_config.binds
        publish: vec![], // TODO: Parse from host_config.port_bindings
        network: req.host_config.as_ref()
            .and_then(|hc| hc.network_mode.clone()),
        hostname: req.hostname,
        user: req.user,
        workdir: req.working_dir,
        entrypoint: req.entrypoint,
        label: req.labels.map(|labels| {
            labels.into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect()
        }).unwrap_or_default(),
        restart: req.host_config.as_ref()
            .and_then(|hc| hc.restart_policy.as_ref())
            .map(|rp| rp.name.clone())
            .unwrap_or_else(|| "no".to_string()),
        memory: req.host_config.as_ref()
            .and_then(|hc| hc.memory)
            .map(|m| (m / 1024 / 1024) as u32), // Convert bytes to MB
        privileged: req.host_config.as_ref()
            .and_then(|hc| hc.privileged)
            .unwrap_or(false),
        read_only: req.host_config.as_ref()
            .and_then(|hc| hc.readonly_rootfs)
            .unwrap_or(false),
        cmd: req.cmd.unwrap_or_default(),
        ..Default::default()
    };

    // Execute create (which is run without starting)
    let result = a3s_box_cli::commands::run::execute(run_args).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "Id": result.box_id,
        "Warnings": []
    })))
}
