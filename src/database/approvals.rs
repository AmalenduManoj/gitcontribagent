use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApprovalAction {
    pub id: i64,
    pub issue_id: i64,
    pub action_type: String,
    pub status: String,
    pub generated_payload: Option<Value>,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub github_comment_id: Option<i64>,
}

pub async fn create_approval_action(
    pool: &PgPool,
    issue_id: i64,
    action_type: &str,
    generated_payload: Option<&Value>,
) -> anyhow::Result<ApprovalAction> {
    let action = sqlx::query_as::<_, ApprovalAction>(
        r#"
        INSERT INTO approval_actions (issue_id, action_type, status, generated_payload)
        VALUES ($1, $2, 'pending', $3)
        RETURNING id, issue_id, action_type, status, generated_payload, approved_by, approved_at, executed_at, github_comment_id
        "#,
    )
    .bind(issue_id)
    .bind(action_type)
    .bind(generated_payload)
    .fetch_one(pool)
    .await?;
    Ok(action)
}

pub async fn get_pending_approval(
    pool: &PgPool,
    issue_id: i64,
    action_type: &str,
) -> anyhow::Result<Option<ApprovalAction>> {
    let action = sqlx::query_as::<_, ApprovalAction>(
        r#"SELECT id, issue_id, action_type, status, generated_payload, approved_by, approved_at, executed_at, github_comment_id
           FROM approval_actions WHERE issue_id = $1 AND action_type = $2 AND status = 'pending'
           ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(issue_id)
    .bind(action_type)
    .fetch_optional(pool)
    .await?;
    Ok(action)
}

pub async fn approve_action(pool: &PgPool, id: i64, approved_by: &str) -> anyhow::Result<()> {
    let result = sqlx::query(
        r#"UPDATE approval_actions SET status = 'approved', approved_by = $2, approved_at = NOW() WHERE id = $1 AND status = 'pending'"#,
    )
    .bind(id)
    .bind(approved_by)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        anyhow::bail!("Approval action {} is not in pending state", id);
    }
    Ok(())
}

pub async fn mark_action_executed(
    pool: &PgPool,
    id: i64,
    github_comment_id: i64,
) -> anyhow::Result<()> {
    let result = sqlx::query(
        r#"UPDATE approval_actions SET status = 'executed', github_comment_id = $2, executed_at = NOW() WHERE id = $1 AND status = 'approved'"#,
    )
    .bind(id)
    .bind(github_comment_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        anyhow::bail!("Approval action {} is not in approved state", id);
    }
    Ok(())
}

pub async fn reject_action(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    let result = sqlx::query(
        r#"UPDATE approval_actions SET status = 'rejected' WHERE id = $1 AND status = 'pending'"#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        anyhow::bail!("Approval action {} is not in pending state", id);
    }
    Ok(())
}
