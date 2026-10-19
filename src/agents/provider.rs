use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn analyze_issue(
        &self,
        prompt: &str,
        user_message: &str,
    ) -> anyhow::Result<String>;

    async fn generate_comment(
        &self,
        prompt: &str,
        user_message: &str,
    ) -> anyhow::Result<String>;
}
