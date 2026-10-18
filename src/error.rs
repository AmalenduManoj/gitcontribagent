use thiserror::Error;

/// Application-wide error type.
///
/// Each variant maps to a layer (database, GitHub API, LLM, email, etc.)
/// so callers can pattern-match on the source of failure.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("GitHub webhook validation failed: {0}")]
    WebhookValidation(String),

    #[error("LLM provider error: {0}")]
    LlmProvider(String),

    #[error("email error: {0}")]
    Email(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("approval state error: {0}")]
    ApprovalState(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Map each error variant to an HTTP status code.
    pub fn status_code(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::WebhookValidation(_) => StatusCode::UNAUTHORIZED,
            AppError::ApprovalState(_) => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = serde_json::json!({ "error": self.to_string() });
        (status, axum::Json(body)).into_response()
    }
}
