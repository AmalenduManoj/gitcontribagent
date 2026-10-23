use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::Json;
use rig::providers::openai;
use serde_json::{json, Value};
use std::sync::Arc;

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
    let github_client = openai::Client::new(std::env::var("OPENAI_API_KEY").unwrap_or_default())
        .map_err(|e| AppError::LlmProvider(format!("Failed to create OpenAI client: {}", e)))?;
    let agent = crate::agents::comment_generation::CommentGenerationAgent::new(&github_client, "gpt-4o");

    let result = crate::services::analysis_service::generate_comment(&state.pool, &agent, issue_id).await?;
    Ok(Json(result))
}

pub async fn approve_comment(
    State(state): State<AppState>,
    Path(issue_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let result = crate::approval::service::approve_comment(&state.pool, issue_id, "api-user").await?;
    Ok(Json(result))
}

pub async fn post_comment(
    State(state): State<AppState>,
    Path(issue_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let result = crate::approval::service::post_comment(&state.pool, state.github.clone(), issue_id).await?;
    Ok(Json(result))
}

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/health", get(health_check))
        .route("/issues/{id}/generate-comment", post(generate_comment))
        .route("/issues/{id}/approve-comment", post(approve_comment))
        .route("/issues/{id}/post-comment", post(post_comment))
        .with_state(state)
}
