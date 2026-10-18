use std::collections::HashSet;
use std::env;

/// Application configuration loaded from environment variables.
///
/// All secrets (GitHub token, OpenAI key, SMTP password) are read from the
/// environment and are never logged or passed to the LLM.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub github_token: String,
    pub github_webhook_secret: String,
    pub openai_api_key: String,
    pub openai_model: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub notification_email: String,
    /// Contribution labels to match (case-insensitive).
    pub contribution_labels: HashSet<String>,
    /// GitHub usernames whose issues should be ignored.
    pub ignored_users: HashSet<String>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Loads `.env` via dotenvy first, then reads each variable.
    /// Required variables cause an error; optional ones have defaults.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let contribution_labels = env::var("CONTRIBUTION_LABELS")
            .unwrap_or_else(|_| {
                "good first issue,good-first-issue,first issue,beginner,help wanted".to_string()
            })
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .collect();

        let ignored_users = env::var("IGNORED_USERS")
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
            database_url: env::var("DATABASE_URL")?,
            github_token: env::var("GITHUB_TOKEN")?,
            github_webhook_secret: env::var("GITHUB_WEBHOOK_SECRET")?,
            openai_api_key: env::var("OPENAI_API_KEY")?,
            openai_model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".into()),
            smtp_host: env::var("SMTP_HOST")?,
            smtp_port: env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(587),
            smtp_user: env::var("SMTP_USER")?,
            smtp_password: env::var("SMTP_PASSWORD")?,
            smtp_from: env::var("SMTP_FROM")?,
            notification_email: env::var("NOTIFICATION_EMAIL")?,
            contribution_labels,
            ignored_users,
        })
    }
}
