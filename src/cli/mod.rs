use clap::{Parser, Subcommand};

/// Top-level CLI structure.
///
/// Uses clap's derive API so subcommands are auto-documented in `--help`.
#[derive(Parser)]
#[command(name = "github-agent")]
#[command(about = "Private AI assistant for GitHub good first issues")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the Axum server
    Serve,
    /// Database commands
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
    /// Repository management
    Repository {
        #[command(subcommand)]
        command: RepositoryCommands,
    },
    /// Issue workflow
    Issue {
        #[command(subcommand)]
        command: IssueCommands,
    },
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Run database migrations
    Migrate,
}

#[derive(Subcommand)]
pub enum RepositoryCommands {
    /// Add a repository to watch (owner/repo)
    Add { repo: String },
    /// List watched repositories
    List,
    /// Remove a repository from watching
    Remove { repo: String },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    /// List tracked issues
    List,
    /// Analyze an issue (owner/repo#123)
    Analyze { issue: String },
    /// Generate a comment for an issue (does NOT post)
    Comment { issue: String },
    /// Approve a pending comment
    CommentApprove { issue: String },
    /// Post an approved comment to GitHub
    CommentPost { issue: String },
}
