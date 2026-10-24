use github_contribution_agent::config::Config;
use github_contribution_agent::github::models::{
    GitHubIssue, GitHubLabel, GitHubRepo, GitHubUser, WebhookEvent, WebhookEventType,
};
use github_contribution_agent::services::issue_service::{FilterResult, IssueFilter};

fn test_config() -> Config {
    Config {
        host: "0.0.0.0".into(),
        port: 8080,
        database_url: "postgres://localhost/test".into(),
        github_token: "token".into(),
        github_webhook_secret: "secret".into(),
        openai_api_key: "key".into(),
        openai_model: "gpt-4o".into(),
        smtp_host: "smtp.example.com".into(),
        smtp_port: 587,
        smtp_user: "user".into(),
        smtp_password: "pass".into(),
        smtp_from: "from@example.com".into(),
        notification_email: "to@example.com".into(),
        contribution_labels: [
            "good first issue",
            "good-first-issue",
            "first issue",
            "beginner",
            "help wanted",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        ignored_users: ["bot-user"].iter().map(|s| s.to_string()).collect(),
        poll_interval_secs: 300,
        poll_enabled: true,
        ollama_base_url: "http://localhost:11434".into(),
        ollama_model: "qwen2.5:14b".into(),
    }
}

fn test_repo() -> GitHubRepo {
    GitHubRepo {
        id: 1,
        name: "test-repo".into(),
        full_name: "owner/test-repo".into(),
        description: Some("A test repo".into()),
        html_url: "https://github.com/owner/test-repo".into(),
        language: Some("Rust".into()),
        stargazers_count: 10,
        open_issues_count: 1,
        default_branch: "main".into(),
        owner: GitHubUser {
            id: 2,
            login: "owner".into(),
            html_url: "https://github.com/owner".into(),
            r#type: "User".into(),
        },
    }
}

fn test_issue(labels: Vec<GitHubLabel>, state: &str, author: &str) -> GitHubIssue {
    GitHubIssue {
        id: 100,
        number: 1,
        title: "Test issue".into(),
        body: Some("Issue body".into()),
        state: state.into(),
        html_url: "https://github.com/owner/test-repo/issues/1".into(),
        user: GitHubUser {
            id: 3,
            login: author.into(),
            html_url: format!("https://github.com/{}", author),
            r#type: "User".into(),
        },
        labels,
        comments: 0,
        created_at: "2026-10-18T00:00:00Z".into(),
        updated_at: "2026-10-18T00:00:00Z".into(),
    }
}

fn label(name: &str) -> GitHubLabel {
    GitHubLabel {
        id: 1,
        name: name.into(),
        color: "7057ff".into(),
        description: None,
    }
}

fn issues_event(action: &str, issue: GitHubIssue) -> WebhookEvent {
    WebhookEvent {
        event_type: WebhookEventType::Issues,
        action: Some(action.into()),
        repository: test_repo(),
        issue: Some(issue),
        sender: GitHubUser {
            id: 4,
            login: "sender".into(),
            html_url: "https://github.com/sender".into(),
            r#type: "User".into(),
        },
    }
}

#[test]
fn accept_issue_with_matching_label() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("good first issue")], "open", "contributor"));
    assert_eq!(filter.should_process(&event), FilterResult::Accept);
}

#[test]
fn reject_issue_without_matching_label() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("bug")], "open", "contributor"));
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("label")),
        FilterResult::Accept => panic!("should reject"),
    }
}

#[test]
fn reject_closed_issue() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("good first issue")], "closed", "contributor"));
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("state")),
        FilterResult::Accept => panic!("should reject"),
    }
}

#[test]
fn reject_ignored_user() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("good first issue")], "open", "bot-user"));
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("ignored")),
        FilterResult::Accept => panic!("should reject"),
    }
}

#[test]
fn reject_irrelevant_action() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("closed", test_issue(vec![label("good first issue")], "open", "contributor"));
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("action")),
        FilterResult::Accept => panic!("should reject"),
    }
}

#[test]
fn reject_ping_event() {
    let filter = IssueFilter::from_config(&test_config());
    let event = WebhookEvent {
        event_type: WebhookEventType::Ping,
        action: None,
        repository: test_repo(),
        issue: None,
        sender: GitHubUser {
            id: 4,
            login: "sender".into(),
            html_url: "https://github.com/sender".into(),
            r#type: "User".into(),
        },
    };
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("ping")),
        FilterResult::Accept => panic!("should reject"),
    }
}

#[test]
fn accept_reopened_issue() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("reopened", test_issue(vec![label("good first issue")], "open", "contributor"));
    assert_eq!(filter.should_process(&event), FilterResult::Accept);
}

#[test]
fn accept_labeled_event() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("labeled", test_issue(vec![label("good first issue")], "open", "contributor"));
    assert_eq!(filter.should_process(&event), FilterResult::Accept);
}

#[test]
fn label_matching_is_case_insensitive() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("Good First Issue")], "open", "contributor"));
    assert_eq!(filter.should_process(&event), FilterResult::Accept);
}

#[test]
fn label_matching_hyphens() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("good-first-issue")], "open", "contributor"));
    assert_eq!(filter.should_process(&event), FilterResult::Accept);
}

#[test]
fn label_matching_partial_no_match() {
    let filter = IssueFilter::from_config(&test_config());
    let event = issues_event("opened", test_issue(vec![label("good")], "open", "contributor"));
    match filter.should_process(&event) {
        FilterResult::Reject(_) => {}
        FilterResult::Accept => panic!("partial label should not match"),
    }
}

#[test]
fn reject_unknown_event_type() {
    let filter = IssueFilter::from_config(&test_config());
    let event = WebhookEvent {
        event_type: WebhookEventType::Unknown("star".into()),
        action: Some("created".into()),
        repository: test_repo(),
        issue: None,
        sender: GitHubUser {
            id: 4,
            login: "sender".into(),
            html_url: "https://github.com/sender".into(),
            r#type: "User".into(),
        },
    };
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("unknown")),
        FilterResult::Accept => panic!("should reject"),
    }
}

#[test]
fn reject_no_issue_in_payload() {
    let filter = IssueFilter::from_config(&test_config());
    let event = WebhookEvent {
        event_type: WebhookEventType::Issues,
        action: Some("opened".into()),
        repository: test_repo(),
        issue: None,
        sender: GitHubUser {
            id: 4,
            login: "sender".into(),
            html_url: "https://github.com/sender".into(),
            r#type: "User".into(),
        },
    };
    match filter.should_process(&event) {
        FilterResult::Reject(reason) => assert!(reason.contains("no issue")),
        FilterResult::Accept => panic!("should reject"),
    }
}
