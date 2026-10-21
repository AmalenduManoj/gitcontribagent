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

pub async fn process_polled_issue(
    pool: &sqlx::PgPool,
    filter: &IssueFilter,
    repo: &crate::database::repositories::Repository,
    issue: &crate::github::models::GitHubIssue,
    labels: &serde_json::Value,
) -> anyhow::Result<FilterResult> {
    let event = crate::github::models::WebhookEvent {
        event_type: crate::github::models::WebhookEventType::Issues,
        action: Some("opened".into()),
        repository: crate::github::models::GitHubRepo {
            id: repo.github_repo_id.unwrap_or(0),
            name: repo.name.clone(),
            full_name: format!("{}/{}", repo.owner, repo.name),
            description: None,
            html_url: format!("https://github.com/{}/{}", repo.owner, repo.name),
            language: None,
            stargazers_count: 0,
            open_issues_count: 0,
            default_branch: "main".into(),
            owner: crate::github::models::GitHubUser {
                id: 0,
                login: repo.owner.clone(),
                html_url: format!("https://github.com/{}", repo.owner),
                r#type: "User".into(),
            },
        },
        issue: Some(issue.clone()),
        sender: crate::github::models::GitHubUser {
            id: issue.user.id,
            login: issue.user.login.clone(),
            html_url: issue.user.html_url.clone(),
            r#type: issue.user.r#type.clone(),
        },
    };

    let result = filter.should_process(&event);
    if result == FilterResult::Accept {
        crate::database::issues::upsert_issue(
            pool,
            repo.id,
            issue.id,
            issue.number,
            &issue.title,
            issue.body.as_deref(),
            &issue.state,
            Some(&issue.user.login),
            labels,
            Some(&issue.html_url),
        ).await?;
    }
    Ok(result)
}
