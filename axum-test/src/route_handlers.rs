use axum::{response::IntoResponse, Extension};

use crate::application::application::Application;

use super::types::Result;
use super::types::{json, ApiResponse};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResponse {
    done: bool,
}

impl ApiResponse for HealthCheckResponse {}

pub async fn health(Extension(_app): Extension<Application>) -> Result<impl IntoResponse> {
    Ok(json(HealthCheckResponse { done: true }))
}

pub async fn version(Extension(_app): Extension<Application>) -> Result<impl IntoResponse> {
    Ok(json(HealthCheckResponse { done: true }))
}

pub async fn handle_poster(Extension(_app): Extension<Application>) -> Result<impl IntoResponse> {
    Ok(json(HealthCheckResponse { done: true }))
}
