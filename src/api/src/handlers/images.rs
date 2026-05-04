//! Image API handlers.

use axum::{Json, extract::Path};
use serde_json::json;

use crate::error::{ApiResult, ApiError};

/// GET /images/json - List images.
pub async fn list() -> ApiResult<Json<serde_json::Value>> {
    // TODO: Integrate with image store
    Ok(Json(json!([])))
}

/// POST /images/create - Pull an image.
pub async fn pull() -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Image pull not yet implemented".to_string()))
}

/// GET /images/:name/json - Inspect an image.
pub async fn inspect(Path(_name): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Image inspect not yet implemented".to_string()))
}

/// GET /images/:name/history - Get image history.
pub async fn history(Path(_name): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Image history not yet implemented".to_string()))
}

/// POST /images/:name/push - Push an image.
pub async fn push(Path(_name): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Image push not yet implemented".to_string()))
}

/// POST /images/:name/tag - Tag an image.
pub async fn tag(Path(_name): Path<String>) -> ApiResult<()> {
    Err(ApiError::NotImplemented("Image tag not yet implemented".to_string()))
}

/// DELETE /images/:name - Remove an image.
pub async fn remove(Path(_name): Path<String>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented("Image remove not yet implemented".to_string()))
}

/// POST /build - Build an image from a Dockerfile.
pub async fn build() -> ApiResult<()> {
    Err(ApiError::NotImplemented("Image build not yet implemented".to_string()))
}
