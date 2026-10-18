pub mod repositories;
pub mod issues;
pub mod analyses;
pub mod approvals;
pub mod notifications;

use crate::cli::DbCommands;
use sqlx::postgres::PgPool;

pub async fn create_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

pub async fn run_migration_command(command: DbCommands) -> anyhow::Result<()> {
    match command {
        DbCommands::Migrate => {
            let config = crate::config::Config::from_env()?;
            let pool = create_pool(&config.database_url).await?;
            run_migrations(&pool).await?;
            tracing::info!("Database migrations completed successfully");
            pool.close().await;
            Ok(())
        }
    }
}
