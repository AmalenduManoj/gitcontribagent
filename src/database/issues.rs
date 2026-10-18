use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Issue {
    pub id: i64,
    pub repository_id: i64,
    pub github_issue_id: i64,
    pub issue_number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub author: Option<String>,
    pub labels: Value,
    pub html_url: Option<String>,
}

pub async fn upsert_issue(
    pool: &PgPool,
    repository_id: i64,
    github_issue_id: i64,
    issue_number: i32,
    title: &str,
    body: Option<&str>,
    state: &str,
    author: Option<&str>,
    labels: &Value,
    html_url: Option<&str>,
) -> anyhow::Result<Issue> {
    let issue = sqlx::query_as::<_, Issue>(
        r#"
        INSERT INTO issues (repository_id, github_issue_id, issue_number, title, body, state, author, labels, html_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (repository_id, issue_number) DO UPDATE SET
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            state = EXCLUDED.state,
            author = EXCLUDED.author,
            labels = EXCLUDED.labels,
            html_url = EXCLUDED.html_url,
            updated_at = NOW()
        RETURNING id, repository_id, github_issue_id, issue_number, title, body, state, author, labels, html_url
        "#,
    )
    .bind(repository_id)
    .bind(github_issue_id)
    .bind(issue_number)
    .bind(title)
    .bind(body)
    .bind(state)
    .bind(author)
    .bind(labels)
    .bind(html_url)
    .fetch_one(pool)
    .await?;
    Ok(issue)
}

pub async fn get_issue_by_number(
    pool: &PgPool,
    repository_id: i64,
    issue_number: i32,
) -> anyhow::Result<Option<Issue>> {
    let issue = sqlx::query_as::<_, Issue>(
        r#"SELECT id, repository_id, github_issue_id, issue_number, title, body, state, author, labels, html_url
           FROM issues WHERE repository_id = $1 AND issue_number = $2"#,
    )
    .bind(repository_id)
    .bind(issue_number)
    .fetch_optional(pool)
    .await?;
    Ok(issue)
}

pub async fn list_issues(
    pool: &PgPool,
    repository_id: Option<i64>,
) -> anyhow::Result<Vec<Issue>> {
    let issues = match repository_id {
        Some(rid) => {
            sqlx::query_as::<_, Issue>(
                r#"SELECT id, repository_id, github_issue_id, issue_number, title, body, state, author, labels, html_url
                   FROM issues WHERE repository_id = $1 ORDER BY issue_number DESC"#,
            )
            .bind(rid)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, Issue>(
                r#"SELECT id, repository_id, github_issue_id, issue_number, title, body, state, author, labels, html_url
                   FROM issues ORDER BY issue_number DESC"#,
            )
            .fetch_all(pool)
            .await?
        }
    };
    Ok(issues)
}

pub async fn record_webhook_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    payload: &Value,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        r#"INSERT INTO github_webhook_events (event_id, event_type, payload) VALUES ($1, $2, $3) ON CONFLICT (event_id) DO NOTHING"#,
    )
    .bind(event_id)
    .bind(event_type)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
