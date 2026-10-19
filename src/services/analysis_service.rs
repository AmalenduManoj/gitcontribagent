use serde_json::{json, Value};
use sqlx::PgPool;

use crate::agents::comment_generation::CommentGenerationAgent;
use crate::database::approvals as approval_db;
use crate::database::issues as issue_db;
use crate::error::AppError;
use crate::models::issue::IssueAnalysis;

pub async fn generate_comment(
    pool: &PgPool,
    agent: &CommentGenerationAgent,
    issue_id: i64,
) -> anyhow::Result<Value> {
    let issue = issue_db::get_issue_by_id(pool, issue_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("issue {} not found", issue_id)))?;

    let analysis_row = crate::database::analyses::get_latest_analysis(pool, issue_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("no analysis for issue {}", issue_id)))?;

    let analysis = IssueAnalysis {
        summary: analysis_row.summary,
        requirements: serde_json::from_value(analysis_row.requirements)
            .unwrap_or_default(),
        affected_areas: serde_json::from_value(analysis_row.affected_areas)
            .unwrap_or_default(),
        possible_files: serde_json::from_value(analysis_row.possible_files)
            .unwrap_or_default(),
        implementation_approach: serde_json::from_value(analysis_row.implementation_approach)
            .unwrap_or_default(),
        testing_requirements: serde_json::from_value(analysis_row.testing_requirements)
            .unwrap_or_default(),
        complexity: match analysis_row.complexity.as_str() {
            "low" => crate::models::issue::Complexity::Low,
            "medium" => crate::models::issue::Complexity::Medium,
            _ => crate::models::issue::Complexity::High,
        },
        newcomer_suitability: analysis_row.newcomer_suitability,
        uncertainties: serde_json::from_value(analysis_row.uncertainties)
            .unwrap_or_default(),
    };

    let comment_text = agent
        .generate(&analysis, &issue.title, issue.body.as_deref())
        .await?;

    let payload = json!({ "body": comment_text });
    let action = approval_db::create_approval_action(pool, issue_id, "post_comment", Some(&payload)).await?;

    tracing::info!(action_id = action.id, issue_id = issue_id, "Comment generated (pending approval)");

    Ok(json!({
        "action_id": action.id,
        "comment_text": comment_text,
        "status": action.status,
    }))
}
