//! Network API handlers.

use axum::{Json, extract::Path};
use serde_json::json;

use crate::error::{ApiResult, ApiError};

/// GET /networks - List networks.
pub async fn list() -> ApiResult<Json<serde_json::Value>> {
    // TODO: Integrate with network store
    Ok(Json(json!([])))
}

/// POST /networks/create - Create a network.
pub async fn create() -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Network create not yet implemented".to_string()))
}

/// GET /networks/:id - Inspect a network.
pub async fn inspect(Path(_id): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Network inspect not yet implemented".to_string()))
}

/// DELETE /networks/:id - Remove a network.
pub async fn remove(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Network remove not yet implemented".to_string()))
}

/// POST /networks/:id/connect - Connect a container to a network.
pub async fn connect(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Network connect not yet implemented".to_string()))
}

/// POST /networks/:id/disconnect - Disconnect a container from a network.
pub async fn disconnect(Path(_id): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Network disconnect not yet implemented".to_string()))
}
