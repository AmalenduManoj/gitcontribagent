use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::approval::service;
use crate::error::AppError;
use crate::github::client::GitHubClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub github: Arc<dyn GitHubClient>,
}

pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn generate_comment(
    State(state): State<AppState>,
    Path(issue_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    todo!("Wire up CommentGenerationAgent")
}

pub async fn approve_comment(
    State(state): State<AppState>,
    Path(issue_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let result = service::approve_comment(&state.pool, issue_id, "cli-user").await?;
    Ok(Json(result))
}

pub async fn post_comment(
    State(state): State<AppState>,
    Path(issue_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let result = service::post_comment(&state.pool, state.github.clone(), issue_id).await?;
    Ok(Json(result))
}

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/health", get(health_check))
        .route("/issues/:id/generate-comment", post(generate_comment))
        .route("/issues/:id/approve-comment", post(approve_comment))
        .route("/issues/:id/post-comment", post(post_comment))
        .with_state(state)
}
