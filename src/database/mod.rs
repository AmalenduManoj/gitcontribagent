pub mod repositories;
pub mod issues;
pub mod analyses;
pub mod approvals;
pub mod notifications;

use crate::cli::DbCommands;

/// Run a database CLI command.
///
/// Implemented in Phase 2+.
pub async fn run_migration_command(_command: DbCommands) -> anyhow::Result<()> {
    todo!("Database migrations: Phase 2+")
}
