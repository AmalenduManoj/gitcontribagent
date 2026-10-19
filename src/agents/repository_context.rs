use crate::error::AppError;
use crate::github::client::GitHubClient;
use crate::github::models::GitHubIssue;
use std::sync::Arc;

pub struct RepositoryContextAgent {
    github: Arc<dyn GitHubClient>,
}

impl RepositoryContextAgent {
    pub fn new(github: Arc<dyn GitHubClient>) -> Self {
        Self { github }
    }

    pub async fn gather_context(
        &self,
        owner: &str,
        repo: &str,
        issue: &GitHubIssue,
    ) -> anyhow::Result<String> {
        let mut context = String::new();

        if let Some(readme) = self.github.get_readme(owner, repo).await? {
            context.push_str("=== README ===\n");
            context.push_str(&readme);
            context.push_str("\n\n");
        }

        if let Some(contributing) = self.github.get_contributing(owner, repo).await? {
            context.push_str("=== CONTRIBUTING ===\n");
            context.push_str(&contributing);
            context.push_str("\n\n");
        }

        for label in &issue.labels {
            context.push_str(&format!("Label: {}\n", label.name));
        }
        context.push('\n');

        if issue.body.is_none() || issue.body.as_ref().map(|b| b.len()).unwrap_or(0) < 500 {
            context.push_str("=== ISSUE BODY ===\n");
            if let Some(ref body) = issue.body {
                context.push_str(body);
            }
            context.push_str("\n\n");
        }

        for path in ["Cargo.toml", "package.json", "pyproject.toml", "go.mod"] {
            if let Some(content) = self.github.get_file_content(owner, repo, path).await? {
                context.push_str(&format!("=== {} ===\n", path));
                context.push_str(&content);
                context.push_str("\n\n");
                break;
            }
        }

        if context.is_empty() {
            context.push_str("No additional repository context available.\n");
        }

        Ok(context)
    }
}

pub async fn load_prompt(path: &str) -> Result<String, AppError> {
    std::fs::read_to_string(path)
        .map_err(|e| AppError::Config(format!("Failed to load prompt '{}': {}", path, e)))
}
