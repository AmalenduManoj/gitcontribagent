pub mod routes;
pub mod webhooks;

/// Start the Axum HTTP server.
///
/// Implemented in Phase 2+. Currently a placeholder.
pub async fn serve() -> anyhow::Result<()> {
    tracing::info!("Starting github-contribution-agent server...");
    todo!("Axum server: Phase 2+")
}
