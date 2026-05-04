//! Container API handlers.

use axum::{Json, extract::Path};
use serde_json::json;

use crate::error::{ApiResult, ApiError};

/// GET /containers/json - List containers.
pub async fn list() -> ApiResult<Json<serde_json::Value>> {
    let state = a3s_box_cli::state::StateFile::load_default()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let containers: Vec<_> = state.list(true)
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
                "Status": format!("{} {}", record.status, ""),
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

    Ok(Json(json!(containers)))
}

/// POST /containers/create - Create a container.
pub async fn create() -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Container creation not yet implemented".to_string()))
}

/// GET /containers/:id/json - Inspect a container.
pub async fn inspect(Path(id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    let state = a3s_box_cli::state::StateFile::load_default()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Find container by ID or name
    let record = state.list(true)
        .into_iter()
        .find(|r| r.id.starts_with(&id) || r.name == id)
        .ok_or_else(|| ApiError::NotFound(format!("Container {} not found", id)))?;

    Ok(Json(json!({
        "Id": record.id,
        "Created": record.created_at.to_rfc3339(),
        "Path": record.cmd.first().unwrap_or(&String::new()),
        "Args": &record.cmd[1..],
        "State": {
            "Status": record.status,
            "Running": record.status == "running",
            "Paused": record.status == "paused",
            "Restarting": false,
            "OOMKilled": false,
            "Dead": record.status == "dead",
            "Pid": record.pid.unwrap_or(0),
            "ExitCode": record.exit_code.unwrap_or(0),
            "Error": "",
            "StartedAt": record.started_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
            "FinishedAt": ""
        },
        "Image": record.image,
        "ResolvConfPath": "",
        "HostnamePath": "",
        "HostsPath": "",
        "LogPath": record.console_log,
        "Name": format!("/{}", record.name),
        "RestartCount": record.restart_count,
        "Driver": "a3s-box",
        "Platform": "linux",
        "MountLabel": "",
        "ProcessLabel": "",
        "AppArmorProfile": "",
        "ExecIDs": null,
        "HostConfig": {
            "Binds": null,
            "ContainerIDFile": "",
            "LogConfig": {
                "Type": "json-file",
                "Config": {}
            },
            "NetworkMode": record.network_mode,
            "PortBindings": {},
            "RestartPolicy": {
                "Name": record.restart_policy,
                "MaximumRetryCount": record.max_restart_count
            },
            "AutoRemove": record.auto_remove,
            "VolumeDriver": "",
            "VolumesFrom": null,
            "CapAdd": null,
            "CapDrop": null,
            "CgroupnsMode": "host",
            "Dns": [],
            "DnsOptions": [],
            "DnsSearch": [],
            "ExtraHosts": null,
            "GroupAdd": null,
            "IpcMode": "private",
            "Cgroup": "",
            "Links": null,
            "OomScoreAdj": 0,
            "PidMode": "",
            "Privileged": record.privileged,
            "PublishAllPorts": false,
            "ReadonlyRootfs": record.read_only,
            "SecurityOpt": null,
            "UTSMode": "",
            "UsernsMode": "",
            "ShmSize": 67108864,
            "Runtime": "a3s-box",
            "ConsoleSize": [0, 0],
            "Isolation": "",
            "CpuShares": 0,
            "Memory": (record.memory_mb as i64) * 1024 * 1024,
            "NanoCpus": 0,
            "CgroupParent": "",
            "BlkioWeight": 0,
            "BlkioWeightDevice": [],
            "BlkioDeviceReadBps": [],
            "BlkioDeviceWriteBps": [],
            "BlkioDeviceReadIOps": [],
            "BlkioDeviceWriteIOps": [],
            "CpuPeriod": 0,
            "CpuQuota": 0,
            "CpuRealtimePeriod": 0,
            "CpuRealtimeRuntime": 0,
            "CpusetCpus": "",
            "CpusetMems": "",
            "Devices": [],
            "DeviceCgroupRules": null,
            "DeviceRequests": null,
            "KernelMemory": 0,
            "KernelMemoryTCP": 0,
            "MemoryReservation": 0,
            "MemorySwap": 0,
            "MemorySwappiness": null,
            "OomKillDisable": false,
            "PidsLimit": null,
            "Ulimits": null,
            "CpuCount": 0,
            "CpuPercent": 0,
            "IOMaximumIOps": 0,
            "IOMaximumBandwidth": 0,
            "MaskedPaths": null,
            "ReadonlyPaths": null
        },
        "GraphDriver": {
            "Data": {
                "Dir": record.box_dir
            },
            "Name": "a3s-box"
        },
        "Mounts": [],
        "Config": {
            "Hostname": record.hostname.unwrap_or_else(|| record.name.clone()),
            "Domainname": "",
            "User": record.user.unwrap_or_default(),
            "AttachStdin": false,
            "AttachStdout": true,
            "AttachStderr": true,
            "Tty": false,
            "OpenStdin": false,
            "StdinOnce": false,
            "Env": record.env.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>(),
            "Cmd": record.cmd,
            "Image": record.image,
            "Volumes": null,
            "WorkingDir": record.workdir.unwrap_or_else(|| "/".to_string()),
            "Entrypoint": record.entrypoint,
            "OnBuild": null,
            "Labels": record.labels
        },
        "NetworkSettings": {
            "Bridge": "",
            "SandboxID": "",
            "HairpinMode": false,
            "LinkLocalIPv6Address": "",
            "LinkLocalIPv6PrefixLen": 0,
            "Ports": {},
            "SandboxKey": "",
            "SecondaryIPAddresses": null,
            "SecondaryIPv6Addresses": null,
            "EndpointID": "",
            "Gateway": "",
            "GlobalIPv6Address": "",
            "GlobalIPv6PrefixLen": 0,
            "IPAddress": "",
            "IPPrefixLen": 0,
            "IPv6Gateway": "",
            "MacAddress": "",
            "Networks": {}
        }
    })))
}

/// POST /containers/:id/start - Start a container.
pub async fn start(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container start not yet implemented".to_string()))
}

/// POST /containers/:id/stop - Stop a container.
pub async fn stop(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container stop not yet implemented".to_string()))
}

/// POST /containers/:id/restart - Restart a container.
pub async fn restart(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container restart not yet implemented".to_string()))
}

/// POST /containers/:id/kill - Kill a container.
pub async fn kill(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container kill not yet implemented".to_string()))
}

/// POST /containers/:id/pause - Pause a container.
pub async fn pause(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container pause not yet implemented".to_string()))
}

/// POST /containers/:id/unpause - Unpause a container.
pub async fn unpause(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container unpause not yet implemented".to_string()))
}

/// POST /containers/:id/wait - Wait for a container to stop.
pub async fn wait(Path(_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Container wait not yet implemented".to_string()))
}

/// DELETE /containers/:id - Remove a container.
pub async fn remove(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Container remove not yet implemented".to_string()))
}

/// GET /containers/:id/logs - Get container logs.
pub async fn logs(Path(_id): Path<String>) -> ApiResult<String> {
    Err(ApiError::NotImplemented("Container logs not yet implemented".to_string()))
}

/// GET /containers/:id/stats - Get container stats.
pub async fn stats(Path(_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Container stats not yet implemented".to_string()))
}

/// GET /containers/:id/top - List processes in a container.
pub async fn top(Path(_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Container top not yet implemented".to_string()))
}

/// POST /containers/:id/exec - Create an exec instance.
pub async fn exec_create(Path(_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Exec create not yet implemented".to_string()))
}

/// POST /exec/:id/start - Start an exec instance.
pub async fn exec_start(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Exec start not yet implemented".to_string()))
}
