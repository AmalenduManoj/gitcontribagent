use hmac::{Hmac, KeyInit, Mac};
use serde_json::Value;
use sha2::Sha256;

use crate::error::AppError;
use crate::github::models::{WebhookEvent, WebhookEventType};

type HmacSha256 = Hmac<Sha256>;

pub fn validate_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
    let Some(hex_signature) = signature.strip_prefix("sha256=") else {
        return false;
    };

    let Ok(expected) = hex::decode(hex_signature) else {
        return false;
    };

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };

    mac.update(payload);
    let result = mac.finalize();
    let computed = result.into_bytes();

    computed.as_slice() == expected.as_slice()
}

pub fn parse_webhook_event(event_type: &str, payload: &Value) -> anyhow::Result<WebhookEvent> {
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .map(String::from);

    let repository = serde_json::from_value(payload["repository"].clone())
        .map_err(|e| AppError::WebhookValidation(format!("failed to parse repository: {}", e)))?;

    let sender = serde_json::from_value(payload["sender"].clone())
        .map_err(|e| AppError::WebhookValidation(format!("failed to parse sender: {}", e)))?;

    let issue = payload
        .get("issue")
        .map(|v| serde_json::from_value(v.clone()))
        .transpose()
        .map_err(|e| AppError::WebhookValidation(format!("failed to parse issue: {}", e)))?;

    let event_type = match event_type {
        "issues" => WebhookEventType::Issues,
        "issue_comment" => WebhookEventType::IssueComment,
        "installation" => WebhookEventType::Installation,
        "ping" => WebhookEventType::Ping,
        other => WebhookEventType::Unknown(other.to_string()),
    };

    Ok(WebhookEvent {
        event_type,
        action,
        repository,
        issue,
        sender,
    })
}
