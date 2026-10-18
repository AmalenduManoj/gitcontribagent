use serde_json::Value;
use sqlx::PgPool;

use crate::config::Config;
use crate::database::issues as issue_db;
use crate::database::repositories;
use crate::github::models::{WebhookEvent, WebhookEventType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterResult {
    Accept,
    Reject(String),
}

pub struct IssueFilter {
    contribution_labels: std::collections::HashSet<String>,
    ignored_users: std::collections::HashSet<String>,
}

impl IssueFilter {
    pub fn from_config(config: &Config) -> Self {
        Self {
            contribution_labels: config.contribution_labels.clone(),
            ignored_users: config.ignored_users.clone(),
        }
    }

    pub fn should_process(&self, event: &WebhookEvent) -> FilterResult {
        match &event.event_type {
            WebhookEventType::Issues | WebhookEventType::IssueComment => {}
            WebhookEventType::Ping => return FilterResult::Reject("ping event".into()),
            WebhookEventType::Installation => {
                return FilterResult::Reject("installation event".into())
            }
            WebhookEventType::Unknown(ref t) => {
                return FilterResult::Reject(format!("unknown event type: {}", t))
            }
        }

        let action = match &event.action {
            Some(a) => a.as_str(),
            None => return FilterResult::Reject("no action".into()),
        };
        match action {
            "opened" | "labeled" | "unlabeled" | "reopened" => {}
            other => return FilterResult::Reject(format!("action '{}' not relevant", other)),
        }

        let issue = match &event.issue {
            Some(i) => i,
            None => return FilterResult::Reject("no issue in payload".into()),
        };

        if issue.state != "open" {
            return FilterResult::Reject(format!("issue state is '{}'", issue.state));
        }

        let has_matching_label = issue
            .labels
            .iter()
            .any(|label| self.contribution_labels.contains(&label.name.to_lowercase()));
        if !has_matching_label {
            return FilterResult::Reject("no matching contribution label".into());
        }

        if self.ignored_users.contains(&issue.user.login) {
            return FilterResult::Reject(format!("user '{}' is ignored", issue.user.login));
        }

        FilterResult::Accept
    }
}

pub async fn process_webhook_event(
    pool: &PgPool,
    filter: &IssueFilter,
    event: &WebhookEvent,
    event_id: &str,
    event_type: &str,
    payload: &Value,
) -> anyhow::Result<FilterResult> {
    let is_new = issue_db::record_webhook_event(pool, event_id, event_type, payload).await?;
    if !is_new {
        return Ok(FilterResult::Reject("duplicate event".into()));
    }

    let result = filter.should_process(event);

    if result == FilterResult::Accept {
        let repo = repositories::upsert_repository(
            pool,
            &event.repository.owner.login,
            &event.repository.name,
            Some(event.repository.id),
        )
        .await?;

        if let Some(ref issue) = event.issue {
            let labels: Value = serde_json::to_value(&issue.labels)?;
            issue_db::upsert_issue(
                pool,
                repo.id,
                issue.id,
                issue.number,
                &issue.title,
                issue.body.as_deref(),
                &issue.state,
                Some(&issue.user.login),
                &labels,
                Some(&issue.html_url),
            )
            .await?;

            tracing::info!(
                repository = format!("{}/{}", repo.owner, repo.name),
                issue_number = issue.number,
                "Issue accepted for analysis"
            );
        }
    }

    Ok(result)
}
