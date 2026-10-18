use github_contribution_agent::github::webhooks::{parse_webhook_event, validate_signature};
use serde_json::json;

const SECRET: &str = "test_webhook_secret";

fn compute_signature(payload: &[u8], secret: &str) -> String {
    use hmac::{Hmac, KeyInit, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

#[test]
fn valid_signature_passes() {
    let payload = b"{\"action\":\"opened\",\"issue\":{\"number\":1}}";
    let sig = compute_signature(payload, SECRET);
    assert!(validate_signature(payload, &sig, SECRET));
}

#[test]
fn invalid_signature_fails() {
    let payload = b"{\"action\":\"opened\"}";
    assert!(!validate_signature(payload, "sha256=invalid", SECRET));
}

#[test]
fn tampered_payload_fails() {
    let payload = b"{\"action\":\"opened\"}";
    let sig = compute_signature(payload, SECRET);

    let tampered = b"{\"action\":\"closed\"}";
    assert!(!validate_signature(tampered, &sig, SECRET));
}

#[test]
fn missing_prefix_fails() {
    let payload = b"{\"action\":\"opened\"}";
    let sig = compute_signature(payload, SECRET);
    let without_prefix = sig.strip_prefix("sha256=").unwrap();
    assert!(!validate_signature(payload, without_prefix, SECRET));
}

#[test]
fn wrong_secret_fails() {
    let payload = b"{\"action\":\"opened\"}";
    let sig = compute_signature(payload, "wrong_secret");
    assert!(!validate_signature(payload, &sig, SECRET));
}

#[test]
fn parse_issues_event() {
    let payload = json!({
        "action": "opened",
        "repository": {
            "id": 123,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "description": "A test repo",
            "html_url": "https://github.com/owner/test-repo",
            "language": "Rust",
            "stargazers_count": 42,
            "open_issues_count": 5,
            "default_branch": "main",
            "owner": {
                "id": 456,
                "login": "owner",
                "html_url": "https://github.com/owner",
                "type": "User"
            }
        },
        "issue": {
            "id": 789,
            "number": 1,
            "title": "Bug in parser",
            "body": "Parser crashes on empty input",
            "state": "open",
            "html_url": "https://github.com/owner/test-repo/issues/1",
            "user": {
                "id": 111,
                "login": "contributor",
                "html_url": "https://github.com/contributor",
                "type": "User"
            },
            "labels": [
                {"id": 1, "name": "good first issue", "color": "7057ff", "description": "Good for newcomers"}
            ],
            "comments": 0,
            "created_at": "2026-10-18T00:00:00Z",
            "updated_at": "2026-10-18T00:00:00Z"
        },
        "sender": {
            "id": 222,
            "login": "maintainer",
            "html_url": "https://github.com/maintainer",
            "type": "User"
        }
    });

    let event = parse_webhook_event("issues", &payload).unwrap();
    assert_eq!(event.action.as_deref(), Some("opened"));
    assert!(matches!(
        event.event_type,
        github_contribution_agent::github::models::WebhookEventType::Issues
    ));
    assert_eq!(event.repository.full_name, "owner/test-repo");
    assert!(event.issue.is_some());
    let issue = event.issue.unwrap();
    assert_eq!(issue.title, "Bug in parser");
    assert_eq!(issue.labels.len(), 1);
    assert_eq!(issue.labels[0].name, "good first issue");
}

#[test]
fn parse_ping_event() {
    let payload = json!({
        "zen": "Keep it logically awesome.",
        "hook_id": 12345,
        "repository": {
            "id": 123,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "description": null,
            "html_url": "https://github.com/owner/test-repo",
            "language": null,
            "stargazers_count": 0,
            "open_issues_count": 0,
            "default_branch": "main",
            "owner": {
                "id": 456,
                "login": "owner",
                "html_url": "https://github.com/owner",
                "type": "User"
            }
        },
        "sender": {
            "id": 222,
            "login": "maintainer",
            "html_url": "https://github.com/maintainer",
            "type": "User"
        }
    });

    let event = parse_webhook_event("ping", &payload).unwrap();
    assert!(event.action.is_none());
    assert!(matches!(
        event.event_type,
        github_contribution_agent::github::models::WebhookEventType::Ping
    ));
    assert!(event.issue.is_none());
}

#[test]
fn parse_issue_comment_event() {
    let payload = json!({
        "action": "created",
        "repository": {
            "id": 123,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "description": null,
            "html_url": "https://github.com/owner/test-repo",
            "language": null,
            "stargazers_count": 0,
            "open_issues_count": 1,
            "default_branch": "main",
            "owner": {
                "id": 456,
                "login": "owner",
                "html_url": "https://github.com/owner",
                "type": "User"
            }
        },
        "issue": {
            "id": 789,
            "number": 1,
            "title": "Bug in parser",
            "body": null,
            "state": "open",
            "html_url": "https://github.com/owner/test-repo/issues/1",
            "user": {"id": 111, "login": "contributor", "html_url": "https://github.com/contributor", "type": "User"},
            "labels": [],
            "comments": 1,
            "created_at": "2026-10-18T00:00:00Z",
            "updated_at": "2026-10-18T00:00:00Z"
        },
        "sender": {
            "id": 333,
            "login": "commenter",
            "html_url": "https://github.com/commenter",
            "type": "User"
        }
    });

    let event = parse_webhook_event("issue_comment", &payload).unwrap();
    assert_eq!(event.action.as_deref(), Some("created"));
    assert!(matches!(
        event.event_type,
        github_contribution_agent::github::models::WebhookEventType::IssueComment
    ));
    assert!(event.issue.is_some());
}
