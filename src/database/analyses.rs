use serde_json::{json, Value};
use sqlx::PgPool;
use crate::models::issue::{Complexity, IssueAnalysis};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IssueAnalysisRow {
    pub id: i64,
    pub issue_id: i64,
    pub summary: String,
    pub requirements: Value,
    pub affected_areas: Value,
    pub possible_files: Value,
    pub implementation_approach: Value,
    pub testing_requirements: Value,
    pub complexity: String,
    pub newcomer_suitability: String,
    pub uncertainties: Value,
    pub raw_response: Option<String>,
}

pub async fn save_analysis(
    pool: &PgPool,
    issue_id: i64,
    analysis: &IssueAnalysis,
    raw_response: Option<&str>,
) -> anyhow::Result<IssueAnalysisRow> {
    let complexity_str = match analysis.complexity {
        Complexity::Low => "low",
        Complexity::Medium => "medium",
        Complexity::High => "high",
    };

    let row = sqlx::query_as::<_, IssueAnalysisRow>(
        r#"
        INSERT INTO issue_analyses (issue_id, summary, requirements, affected_areas, possible_files,
            implementation_approach, testing_requirements, complexity, newcomer_suitability, uncertainties, raw_response)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, issue_id, summary, requirements, affected_areas, possible_files,
            implementation_approach, testing_requirements, complexity, newcomer_suitability, uncertainties, raw_response
        "#,
    )
    .bind(issue_id)
    .bind(&analysis.summary)
    .bind(json!(analysis.requirements))
    .bind(json!(analysis.affected_areas))
    .bind(json!(analysis.possible_files))
    .bind(json!(analysis.implementation_approach))
    .bind(json!(analysis.testing_requirements))
    .bind(complexity_str)
    .bind(&analysis.newcomer_suitability)
    .bind(json!(analysis.uncertainties))
    .bind(raw_response)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_latest_analysis(
    pool: &PgPool,
    issue_id: i64,
) -> anyhow::Result<Option<IssueAnalysisRow>> {
    let row = sqlx::query_as::<_, IssueAnalysisRow>(
        r#"SELECT id, issue_id, summary, requirements, affected_areas, possible_files,
            implementation_approach, testing_requirements, complexity, newcomer_suitability, uncertainties, raw_response
           FROM issue_analyses WHERE issue_id = $1 ORDER BY created_at DESC LIMIT 1"#,
    )
    .bind(issue_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
