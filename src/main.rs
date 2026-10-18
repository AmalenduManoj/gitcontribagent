use clap::Parser;
use github_contribution_agent::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve => {
            github_contribution_agent::api::serve().await?;
        }
        Commands::Db { command } => {
            github_contribution_agent::database::run_migration_command(command).await?;
        }
        Commands::Repository { .. } => {
            todo!("Repository commands: Phase 2+")
        }
        Commands::Issue { .. } => {
            todo!("Issue commands: Phase 2+")
        }
    }

    Ok(())
}
