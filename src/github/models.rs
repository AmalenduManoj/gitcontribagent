use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: i32,
    pub open_issues_count: i32,
    pub default_branch: String,
    pub owner: GitHubUser,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub user: GitHubUser,
    pub labels: Vec<GitHubLabel>,
    pub comments: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubLabel {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub html_url: String,
    #[serde(default)]
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubComment {
    pub id: i64,
    pub body: String,
    pub user: GitHubUser,
    pub html_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub event_type: WebhookEventType,
    pub action: Option<String>,
    pub repository: GitHubRepo,
    pub issue: Option<GitHubIssue>,
    pub sender: GitHubUser,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebhookEventType {
    Issues,
    IssueComment,
    Installation,
    Ping,
    Unknown(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubContentResponse {
    pub content: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCommentResponse {
    pub id: i64,
}
