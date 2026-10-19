use sqlx::PgPool;

use crate::models::issue::IssueAnalysis;
use crate::notifications::NotificationProvider;

pub struct NotificationService {
    provider: Box<dyn NotificationProvider>,
    pool: PgPool,
}

impl NotificationService {
    pub fn new(provider: Box<dyn NotificationProvider>, pool: PgPool) -> Self {
        Self { provider, pool }
    }

    pub async fn notify(
        &self,
        analysis: &IssueAnalysis,
        repo_full_name: &str,
        issue_title: &str,
        issue_url: &str,
        issue_labels: &[String],
        recipient: &str,
    ) -> anyhow::Result<()> {
        let notification = crate::notifications::email::build_notification(
            analysis,
            repo_full_name,
            issue_title,
            issue_url,
            issue_labels,
        );

        self.provider.send(&notification).await?;

        crate::database::notifications::create_notification(
            &self.pool,
            0,
            "email",
            recipient,
            &format!("[{}] {}", repo_full_name, issue_title),
            &notification.summary,
        )
        .await?;

        Ok(())
    }
}
