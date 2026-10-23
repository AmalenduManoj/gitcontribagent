use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Repository {
    pub id: i64,
    pub owner: String,
    pub name: String,
    pub github_repo_id: Option<i64>,
    pub last_polled_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn upsert_repository(
    pool: &PgPool,
    owner: &str,
    name: &str,
    github_repo_id: Option<i64>,
) -> anyhow::Result<Repository> {
    let repo = sqlx::query_as::<_, Repository>(
        r#"
        INSERT INTO repositories (owner, name, github_repo_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (owner, name) DO UPDATE SET github_repo_id = EXCLUDED.github_repo_id, updated_at = NOW()
        RETURNING id, owner, name, github_repo_id, last_polled_at
        "#,
    )
    .bind(owner)
    .bind(name)
    .bind(github_repo_id)
    .fetch_one(pool)
    .await?;
    Ok(repo)
}

pub async fn get_repository_by_owner_name(
    pool: &PgPool,
    owner: &str,
    name: &str,
) -> anyhow::Result<Option<Repository>> {
    let repo = sqlx::query_as::<_, Repository>(
        r#"SELECT id, owner, name, github_repo_id, last_polled_at FROM repositories WHERE owner = $1 AND name = $2"#,
    )
    .bind(owner)
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(repo)
}

pub async fn get_repository_by_id(pool: &PgPool, id: i64) -> anyhow::Result<Option<Repository>> {
    let repo = sqlx::query_as::<_, Repository>(
        r#"SELECT id, owner, name, github_repo_id, last_polled_at FROM repositories WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(repo)
}

pub async fn update_last_polled_at(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE repositories SET last_polled_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_repositories(pool: &PgPool) -> anyhow::Result<Vec<Repository>> {
    let repos = sqlx::query_as::<_, Repository>(
        r#"SELECT id, owner, name, github_repo_id, last_polled_at FROM repositories ORDER BY owner, name"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(repos)
}

pub async fn delete_repository(pool: &PgPool, id: i64) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM repositories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
