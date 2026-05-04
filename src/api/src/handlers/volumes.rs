//! Volume API handlers.

use axum::{Json, extract::Path};
use serde_json::json;

use crate::error::{ApiResult, ApiError};

/// GET /volumes - List volumes.
pub async fn list() -> ApiResult<Json<serde_json::Value>> {
    // TODO: Integrate with volume store
    Ok(Json(json!({
        "Volumes": [],
        "Warnings": null
    })))
}

/// POST /volumes/create - Create a volume.
pub async fn create() -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Volume create not yet implemented".to_string()))
}

/// GET /volumes/:name - Inspect a volume.
pub async fn inspect(Path(_name): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Volume inspect not yet implemented".to_string()))
}

/// DELETE /volumes/:name - Remove a volume.
pub async fn remove(Path(_name): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Volume remove not yet implemented".to_string()))
}
