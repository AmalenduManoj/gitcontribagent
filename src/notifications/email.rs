use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::error::AppError;
use crate::models::notification::Notification;
use crate::notifications::NotificationProvider;

pub fn build_html_body(notification: &Notification) -> String {
    let labels_html = notification
        .issue_labels
        .iter()
        .map(|l| format!("<span style=\"background:#7057ff;color:#fff;padding:2px 8px;border-radius:4px;margin-right:4px;font-size:12px;\">{}</span>", l))
        .collect::<String>();

    let files_html = if notification.possible_files.is_empty() {
        "<em>Not identified</em>".to_string()
    } else {
        notification
            .possible_files
            .iter()
            .map(|f| format!("<li><code>{}</code></li>", f))
            .collect::<String>()
    };

    let approach_html = notification
        .implementation_approach
        .iter()
        .map(|s| format!("<li>{}</li>", s))
        .collect::<String>();

    let uncertainties_html = if notification.uncertainties.is_empty() {
        "<em>None</em>".to_string()
    } else {
        notification
            .uncertainties
            .iter()
            .map(|s| format!("<li>{}</li>", s))
            .collect::<String>()
    };

    let complexity_color = match notification.complexity.as_str() {
        "low" => "#28a745",
        "medium" => "#ffc107",
        "high" => "#dc3545",
        _ => "#6c757d",
    };

    format!(
        r#"<!DOCTYPE html>
<html><body style="font-family:system-ui,sans-serif;max-width:640px;margin:0 auto;padding:24px;color:#24292f;">
<h2 style="margin-bottom:4px;"><a href="{issue_url}" style="color:#0969da;text-decoration:none;">{issue_title}</a></h2>
<p style="color:#57606a;margin-top:0;">{repository} | {labels}</p>

<h3>Summary</h3>
<p>{summary}</p>

<h3>Technical Interpretation</h3>
<p>{interpretation}</p>

<h3>Possible Files</h3>
<ul>{files}</ul>

<h3>Implementation Approach</h3>
<ol>{approach}</ol>

<h3>Complexity</h3>
<p><span style="background:{complexity_color};color:#fff;padding:4px 12px;border-radius:12px;font-weight:bold;text-transform:uppercase;font-size:13px;">{complexity}</span></p>

<h3>Uncertainties</h3>
<ul>{uncertainties}</ul>

<hr style="border:none;border-top:1px solid #d0d7de;margin:24px 0;">
<p style="color:#57606a;font-size:12px;">This analysis was generated privately by github-contribution-agent. No action has been taken on GitHub.</p>
</body></html>"#,
        issue_url = notification.issue_url,
        issue_title = notification.issue_title,
        repository = notification.repository,
        labels = labels_html,
        summary = notification.summary,
        interpretation = notification.technical_interpretation,
        files = files_html,
        approach = approach_html,
        complexity_color = complexity_color,
        complexity = notification.complexity,
        uncertainties = uncertainties_html,
    )
}

pub fn build_plain_text(notification: &Notification) -> String {
    let labels = notification.issue_labels.join(", ");
    let files = if notification.possible_files.is_empty() {
        "Not identified".to_string()
    } else {
        notification.possible_files.join(", ")
    };
    let approach = notification.implementation_approach.join("; ");
    let uncertainties = if notification.uncertainties.is_empty() {
        "None".to_string()
    } else {
        notification.uncertainties.join("; ")
    };

    format!(
        "{} ({})\n{}\n\nSummary:\n{}\n\nTechnical Interpretation:\n{}\n\nPossible Files:\n{}\n\nImplementation Approach:\n{}\n\nComplexity: {}\n\nUncertainties:\n{}\n\n---\nThis analysis was generated privately by github-contribution-agent.\nNo action has been taken on GitHub.",
        notification.issue_title,
        notification.issue_url,
        labels,
        notification.summary,
        notification.technical_interpretation,
        files,
        approach,
        notification.complexity.to_uppercase(),
        uncertainties,
    )
}

pub struct EmailNotificationProvider {
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_password: String,
    from_address: String,
}

impl EmailNotificationProvider {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_user: String,
        smtp_password: String,
        from_address: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            from_address,
        }
    }
}

#[async_trait]
impl NotificationProvider for EmailNotificationProvider {
    async fn send(&self, notification: &Notification) -> anyhow::Result<()> {
        let subject = format!("[{}] {}", notification.repository, notification.issue_title);
        let html_body = build_html_body(notification);
        let plain_body = build_plain_text(notification);

        let email = Message::builder()
            .from(self.from_address.parse().map_err(|e| {
                AppError::Email(format!("Invalid from address: {}", e))
            })?)
            .to(notification.repository.parse().map_err(|e| {
                AppError::Email(format!("Invalid to address: {}", e))
            })?)
            .subject(&subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(plain_body),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )
            .map_err(|e| AppError::Email(format!("Failed to build email: {}", e)))?;

        let creds = Credentials::new(self.smtp_user.clone(), self.smtp_password.clone());

        let mailer = SmtpTransport::relay(&self.smtp_host)
            .map_err(|e| AppError::Email(format!("SMTP relay error: {}", e)))?
            .port(self.smtp_port)
            .credentials(creds)
            .build();

        mailer
            .send(&email)
            .map_err(|e| AppError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!(
            repository = %notification.repository,
            issue = %notification.issue_title,
            "Email notification sent"
        );

        Ok(())
    }
}

pub fn build_notification(
    analysis: &crate::models::issue::IssueAnalysis,
    repo_full_name: &str,
    issue_title: &str,
    issue_url: &str,
    issue_labels: &[String],
) -> Notification {
    let interpretation = format!(
        "{}\n\nAffected areas: {}\n\nNewcomer suitability: {}",
        analysis.summary,
        analysis.affected_areas.join(", "),
        analysis.newcomer_suitability,
    );

    Notification {
        repository: repo_full_name.to_string(),
        issue_title: issue_title.to_string(),
        issue_url: issue_url.to_string(),
        issue_labels: issue_labels.to_vec(),
        summary: analysis.summary.clone(),
        technical_interpretation: interpretation,
        possible_files: analysis.possible_files.clone(),
        implementation_approach: analysis.implementation_approach.clone(),
        complexity: format!("{:?}", analysis.complexity).to_lowercase(),
        uncertainties: analysis.uncertainties.clone(),
    }
}
