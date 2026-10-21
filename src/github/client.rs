use async_trait::async_trait;
use base64::Engine;
use crate::error::AppError;
use crate::github::models::{
    CreateCommentRequest, CreateCommentResponse, GitHubComment, GitHubContentResponse,
    GitHubIssue, GitHubRepo,
};

#[async_trait]
pub trait GitHubClient: Send + Sync {
    async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> anyhow::Result<GitHubIssue>;

    async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<GitHubIssue>>;

    async fn get_issue_comments(
        &self,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> anyhow::Result<Vec<GitHubComment>>;

    async fn get_repo_metadata(
        &self,
        owner: &str,
        repo: &str,
    ) -> anyhow::Result<GitHubRepo>;

    async fn get_readme(&self, owner: &str, repo: &str) -> anyhow::Result<Option<String>>;

    async fn get_contributing(&self, owner: &str, repo: &str) -> anyhow::Result<Option<String>>;

    async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> anyhow::Result<Option<String>>;

    async fn create_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        number: i32,
        body: &str,
    ) -> anyhow::Result<i64>;
}

pub struct GitHubClientImpl {
    client: reqwest::Client,
    token: String,
}

impl GitHubClientImpl {
    pub fn new(token: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("github-contribution-agent")
            .build()
            .expect("Failed to build reqwest client");
        Self { client, token }
    }

    fn base_url(&self, owner: &str, repo: &str) -> String {
        format!("https://api.github.com/repos/{}/{}", owner, repo)
    }

    async fn fetch_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> anyhow::Result<Option<String>> {
        let url = format!("{}/contents/{}", self.base_url(owner, repo), path);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!(
                "GitHub API error {}: {}",
                status, text
            ))
            .into());
        }

        let content_resp: GitHubContentResponse = response
            .json()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        if content_resp.encoding == "base64" {
            let decoded = base64::engine::general_purpose::STANDARD
                .decode(content_resp.content.replace('\n', ""))
                .map_err(|e| AppError::GitHubApi(format!("Base64 decode error: {}", e)))?;
            let text = String::from_utf8(decoded)
                .map_err(|e| AppError::GitHubApi(format!("UTF-8 decode error: {}", e)))?;
            Ok(Some(text))
        } else {
            Ok(Some(content_resp.content))
        }
    }
}

#[async_trait]
impl GitHubClient for GitHubClientImpl {
    async fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        since: Option<&str>,
    ) -> anyhow::Result<Vec<GitHubIssue>> {
        let mut url = format!("{}/issues?state=open&per_page=100", self.base_url(owner, repo));
        if let Some(s) = since {
            url.push_str(&format!("&since={}", s));
        }

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| crate::error::AppError::GitHubApi(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::error::AppError::GitHubApi(format!(
                "GitHub API error {}: {}",
                status, text
            ))
            .into());
        }

        response
            .json::<Vec<GitHubIssue>>()
            .await
            .map_err(|e| crate::error::AppError::GitHubApi(e.to_string()).into())
    }

    async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> anyhow::Result<GitHubIssue> {
        let url = format!("{}/issues/{}", self.base_url(owner, repo), number);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!(
                "GitHub API error {}: {}",
                status, text
            ))
            .into());
        }

        response
            .json::<GitHubIssue>()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()).into())
    }

    async fn get_issue_comments(
        &self,
        owner: &str,
        repo: &str,
        number: i32,
    ) -> anyhow::Result<Vec<GitHubComment>> {
        let url = format!("{}/issues/{}/comments", self.base_url(owner, repo), number);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!(
                "GitHub API error {}: {}",
                status, text
            ))
            .into());
        }

        response
            .json::<Vec<GitHubComment>>()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()).into())
    }

    async fn get_repo_metadata(&self, owner: &str, repo: &str) -> anyhow::Result<GitHubRepo> {
        let url = self.base_url(owner, repo);
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!(
                "GitHub API error {}: {}",
                status, text
            ))
            .into());
        }

        response
            .json::<GitHubRepo>()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()).into())
    }

    async fn get_readme(&self, owner: &str, repo: &str) -> anyhow::Result<Option<String>> {
        self.fetch_content(owner, repo, "README.md").await
    }

    async fn get_contributing(&self, owner: &str, repo: &str) -> anyhow::Result<Option<String>> {
        self.fetch_content(owner, repo, "CONTRIBUTING.md").await
    }

    async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> anyhow::Result<Option<String>> {
        self.fetch_content(owner, repo, path).await
    }

    async fn create_issue_comment(
        &self,
        owner: &str,
        repo: &str,
        number: i32,
        body: &str,
    ) -> anyhow::Result<i64> {
        let url = format!("{}/issues/{}/comments", self.base_url(owner, repo), number);
        let request = CreateCommentRequest {
            body: body.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::GitHubApi(format!(
                "GitHub API error {}: {}",
                status, text
            ))
            .into());
        }

        let comment: CreateCommentResponse = response
            .json()
            .await
            .map_err(|e| AppError::GitHubApi(e.to_string()))?;

        Ok(comment.id)
    }
}
