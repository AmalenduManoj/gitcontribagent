use sqlx::PgPool;
use std::time::Duration;
use crate::config::Config;
use crate::database::repositories;
use crate::github::client::GitHubClient;
use crate::services::issue_service::IssueFilter;
use std::sync::Arc;

pub struct PollingService {
    pool: PgPool,
    github: Arc<dyn GitHubClient>,
    filter: IssueFilter,
    interval: Duration,
}

impl PollingService {
    pub fn new(pool: PgPool, github: Arc<dyn GitHubClient>, config: &Config) -> Self {
        Self {
            pool,
            github,
            filter: IssueFilter::from_config(config),
            interval: Duration::from_secs(config.poll_interval_secs),
        }
    }

    pub async fn run(&self) {
        tracing::info!("Polling service started (interval: {:?})", self.interval);
        loop {
            if let Err(e) = self.poll_all_repos().await {
                tracing::error!("Polling error: {}", e);
            }
            tokio::time::sleep(self.interval).await;
        }
    }

    async fn poll_all_repos(&self) -> anyhow::Result<()> {
        let repos = repositories::list_repositories(&self.pool).await?;

        for repo in repos {
            tracing::info!(owner = %repo.owner, name = %repo.name, "Polling repository");

            let since = repo.last_polled_at.map(|dt| dt.to_rfc3339());

            match self.github.list_issues(&repo.owner, &repo.name, since.as_deref()).await {
                Ok(issues) => {
                    for issue in issues {
                        let labels: serde_json::Value = serde_json::to_value(&issue.labels)?;
                        let result = crate::services::issue_service::process_polled_issue(
                            &self.pool,
                            &self.filter,
                            &repo,
                            &issue,
                            &labels,
                        ).await;

                        match result {
                            Ok(crate::services::issue_service::FilterResult::Accept) => {
                                tracing::info!(
                                    repository = format!("{}/{}", repo.owner, repo.name),
                                    issue_number = issue.number,
                                    "Issue accepted for analysis"
                                );
                            }
                            Ok(crate::services::issue_service::FilterResult::Reject(reason)) => {
                                tracing::debug!(
                                    repository = format!("{}/{}", repo.owner, repo.name),
                                    issue_number = issue.number,
                                    reason = %reason,
                                    "Issue rejected"
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    repository = format!("{}/{}", repo.owner, repo.name),
                                    issue_number = issue.number,
                                    error = %e,
                                    "Error processing polled issue"
                                );
                            }
                        }
                    }
                    repositories::update_last_polled_at(&self.pool, repo.id).await?;
                }
                Err(e) => {
                    tracing::warn!(
                        repository = format!("{}/{}", repo.owner, repo.name),
                        error = %e,
                        "Failed to fetch issues from GitHub"
                    );
                }
            }
        }

        Ok(())
    }
}
