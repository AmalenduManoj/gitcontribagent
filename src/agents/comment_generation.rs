use rig::agent::Agent;
use rig::prelude::*;
use rig::providers::openai;

use crate::error::AppError;
use crate::models::issue::IssueAnalysis;

pub struct CommentGenerationAgent {
    agent: Agent,
}

impl CommentGenerationAgent {
    pub fn new(client: &openai::Client, model: &str) -> Self {
        let agent = client
            .agent(model)
            .preamble(include_str!("../../prompts/comment_generation.txt"))
            .build();
        Self { agent }
    }

    pub async fn generate(
        &self,
        analysis: &IssueAnalysis,
        issue_title: &str,
        issue_body: Option<&str>,
    ) -> anyhow::Result<String> {
        let context = format!(
            "Issue title: {}\n\nIssue body:\n{}\n\nAnalysis summary:\n{}\n\nRequirements:\n{}\n\nImplementation approach:\n{}\n\nComplexity: {:?}",
            issue_title,
            issue_body.unwrap_or("(empty)"),
            analysis.summary,
            analysis.requirements.join("\n"),
            analysis.implementation_approach.join("\n"),
            analysis.complexity,
        );

        let comment = self
            .agent
            .prompt(&context)
            .await
            .map_err(|e| AppError::LlmProvider(format!("Failed to generate comment: {}", e)))?;

        Ok(comment)
    }
}
