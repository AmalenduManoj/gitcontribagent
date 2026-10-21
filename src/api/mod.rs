pub mod routes;
pub mod webhooks;

use crate::config::Config;
use crate::database;
use crate::github::client::{GitHubClient, GitHubClientImpl};
use crate::services::polling_service::PollingService;
use std::sync::Arc;

pub async fn serve() -> anyhow::Result<()> {
    let config = Config::from_env()?;
    let pool = database::create_pool(&config.database_url).await?;
    database::run_migrations(&pool).await?;

    let github: Arc<dyn GitHubClient> = Arc::new(GitHubClientImpl::new(config.github_token.clone()));
    let state = routes::AppState { pool: pool.clone(), github: github.clone() };

    let app = routes::router(state);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    if config.poll_enabled {
        let polling = PollingService::new(pool.clone(), github.clone(), &config);
        tokio::spawn(async move {
            polling.run().await;
        });
    }

    tracing::info!("Server listening on {}:{}", config.host, config.port);
    axum::serve(listener, app).await?;

    Ok(())
}
