use rig::extractor::Extractor;
use rig::prelude::*;
use rig::providers::openai;

use crate::error::AppError;
use crate::models::issue::IssueAnalysis;

pub struct IssueAnalysisAgent {
    extractor: Extractor<IssueAnalysis>,
}

impl IssueAnalysisAgent {
    pub fn new(client: &openai::Client, model: &str) -> Self {
        let extractor = client
            .extractor::<IssueAnalysis>(model)
            .preamble(include_str!("../../prompts/issue_analysis.txt"))
            .retries(2)
            .build();
        Self { extractor }
    }

    pub async fn analyze(
        &self,
        issue_context: &str,
        repo_context: &str,
    ) -> anyhow::Result<IssueAnalysis> {
        let user_message = format!(
            "<untrusted_content>\n<repository_context>\n{}\n</repository_context>\n\n<issue>\n{}\n</issue>\n</untrusted_content>",
            repo_context, issue_context
        );

        let analysis = self.extractor.extract(&user_message).await.map_err(|e| {
            AppError::LlmProvider(format!("Failed to extract issue analysis: {}", e))
        })?;

        Ok(analysis)
    }
}
