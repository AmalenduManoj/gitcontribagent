use serde_json::Value;
use sqlx::PgPool;

use crate::database::approvals as approval_db;
use crate::database::issues as issue_db;
use crate::error::AppError;
use crate::github::client::GitHubClient;
use std::sync::Arc;

pub async fn approve_comment(
    pool: &PgPool,
    issue_id: i64,
    approved_by: &str,
) -> anyhow::Result<Value> {
    issue_db::get_issue_by_id(pool, issue_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("issue {} not found", issue_id)))?;

    let action = approval_db::get_pending_approval(pool, issue_id, "post_comment")
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "no pending comment action for issue {}",
                issue_id
            ))
        })?;

    approval_db::approve_action(pool, action.id, approved_by).await?;

    tracing::info!(
        action_id = action.id,
        issue_id = issue_id,
        approved_by = approved_by,
        "Comment approved"
    );

    Ok(serde_json::json!({
        "action_id": action.id,
        "issue_id": issue_id,
        "status": "approved",
    }))
}

pub async fn post_comment(
    pool: &PgPool,
    github: Arc<dyn GitHubClient>,
    issue_id: i64,
) -> anyhow::Result<Value> {
    let issue = issue_db::get_issue_by_id(pool, issue_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("issue {} not found", issue_id)))?;

    let repo = crate::database::repositories::get_repository_by_id(pool, issue.repository_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("repository {} not found", issue.repository_id)))?;

    let action = approval_db::get_approved_approval(pool, issue_id, "post_comment")
        .await?
        .ok_or_else(|| {
            AppError::ApprovalState(format!(
                "no approved comment action for issue {} — must approve before posting",
                issue_id
            ))
        })?;

    let comment_text = action
        .generated_payload
        .as_ref()
        .and_then(|p| p.get("body"))
        .and_then(|b| b.as_str())
        .ok_or_else(|| AppError::Validation("comment has no body".into()))?;

    let comment_id = github
        .create_issue_comment(&repo.owner, &repo.name, issue.issue_number, comment_text)
        .await
        .map_err(AppError::from)?;

    approval_db::mark_action_executed(pool, action.id, comment_id).await?;

    tracing::info!(
        action_id = action.id,
        issue_id = issue_id,
        github_comment_id = comment_id,
        "Comment posted to GitHub"
    );

    Ok(serde_json::json!({
        "action_id": action.id,
        "issue_id": issue_id,
        "github_comment_id": comment_id,
        "status": "executed",
    }))
}
