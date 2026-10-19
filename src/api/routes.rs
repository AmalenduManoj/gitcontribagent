use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::Json;
use serde_json::{json, Value};

use crate::error::AppError;

pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn generate_comment(
    Path(issue_id): Path<i64>,
    State(pool): State<sqlx::PgPool>,
) -> Result<Json<Value>, AppError> {
    todo!("Wire up CommentGenerationAgent")
}

pub fn router(pool: sqlx::PgPool) -> axum::Router {
    axum::Router::new()
        .route("/health", get(health_check))
        .route("/issues/:id/generate-comment", post(generate_comment))
        .with_state(pool)
}
