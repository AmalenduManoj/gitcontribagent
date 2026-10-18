use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NotificationLog {
    pub id: i64,
    pub analysis_id: i64,
    pub provider: String,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub status: String,
}

pub async fn create_notification(
    pool: &PgPool,
    analysis_id: i64,
    provider: &str,
    recipient: &str,
    subject: &str,
    body: &str,
) -> anyhow::Result<NotificationLog> {
    let log = sqlx::query_as::<_, NotificationLog>(
        r#"
        INSERT INTO notifications (analysis_id, provider, recipient, subject, body, status)
        VALUES ($1, $2, $3, $4, $5, 'sent')
        RETURNING id, analysis_id, provider, recipient, subject, body, status
        "#,
    )
    .bind(analysis_id)
    .bind(provider)
    .bind(recipient)
    .bind(subject)
    .bind(body)
    .fetch_one(pool)
    .await?;
    Ok(log)
}
