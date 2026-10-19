use github_contribution_agent::models::issue::{Complexity, IssueAnalysis};
use github_contribution_agent::models::notification::Notification;
use github_contribution_agent::notifications::email::build_notification;
use github_contribution_agent::notifications::NotificationProvider;

use async_trait::async_trait;
use std::sync::Mutex;

struct MockProvider {
    sent: Mutex<Vec<Notification>>,
}

#[async_trait]
impl NotificationProvider for MockProvider {
    async fn send(&self, notification: &Notification) -> anyhow::Result<()> {
        self.sent.lock().unwrap().push(notification.clone());
        Ok(())
    }
}

fn test_analysis() -> IssueAnalysis {
    IssueAnalysis {
        summary: "Parser crashes on empty input".into(),
        requirements: vec!["Handle empty input".into()],
        affected_areas: vec!["parser".into()],
        possible_files: vec!["src/parser.rs".into()],
        implementation_approach: vec!["Add null check".into()],
        testing_requirements: vec!["Unit test".into()],
        complexity: Complexity::Low,
        newcomer_suitability: "Suitable".into(),
        uncertainties: vec!["Need maintainer input".into()],
    }
}

#[test]
fn notification_builds_correct_subject() {
    let analysis = test_analysis();
    let notification = build_notification(
        &analysis,
        "owner/repo",
        "Bug in parser",
        "https://github.com/owner/repo/issues/1",
        &["bug".to_string(), "good first issue".to_string()],
    );

    assert_eq!(notification.repository, "owner/repo");
    assert_eq!(notification.issue_title, "Bug in parser");
    assert_eq!(notification.issue_url, "https://github.com/owner/repo/issues/1");
    assert_eq!(notification.issue_labels.len(), 2);
    assert_eq!(notification.complexity, "low");
}

#[test]
fn notification_builds_html_body() {
    let analysis = test_analysis();
    let notification = build_notification(
        &analysis,
        "owner/repo",
        "Bug in parser",
        "https://github.com/owner/repo/issues/1",
        &["good first issue".to_string()],
    );

    let html = github_contribution_agent::notifications::email::build_html_body(&notification);
    assert!(html.contains("https://github.com/owner/repo/issues/1"));
    assert!(html.contains("Parser crashes on empty input"));
    assert!(html.contains("low"));
    assert!(html.contains("good first issue"));
}

#[test]
fn notification_builds_plain_text() {
    let analysis = test_analysis();
    let notification = build_notification(
        &analysis,
        "owner/repo",
        "Bug in parser",
        "https://github.com/owner/repo/issues/1",
        &["bug".to_string()],
    );

    let text = github_contribution_agent::notifications::email::build_plain_text(&notification);
    assert!(text.contains("Bug in parser"));
    assert!(text.contains("Parser crashes on empty input"));
    assert!(text.contains("https://github.com/owner/repo/issues/1"));
    assert!(!text.contains("<html>"));
}

#[tokio::test]
async fn mock_provider_receives_notification() {
    let provider = MockProvider {
        sent: Mutex::new(Vec::new()),
    };

    let analysis = test_analysis();
    let notification = build_notification(
        &analysis,
        "owner/repo",
        "Bug in parser",
        "https://github.com/owner/repo/issues/1",
        &["bug".to_string()],
    );

    provider.send(&notification).await.unwrap();

    let sent = provider.sent.lock().unwrap();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].issue_title, "Bug in parser");
}
