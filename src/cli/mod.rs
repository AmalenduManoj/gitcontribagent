use clap::{Parser, Subcommand};
use crate::github::client::GitHubClient;

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
    /// Interactive setup wizard
    Setup,
    /// Systemd service management
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },
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
pub enum ServiceCommands {
    /// Install as systemd service (auto-start on boot)
    Install,
    /// Start the service
    Start,
    /// Stop the service
    Stop,
    /// Show service status
    Status,
    /// Uninstall the service
    Uninstall,
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

pub async fn handle_repository(command: RepositoryCommands) -> anyhow::Result<()> {
    let config = crate::config::Config::from_env()?;
    let pool = crate::database::create_pool(&config.database_url).await?;

    match command {
        RepositoryCommands::Add { repo } => {
            let (owner, mut name) = parse_repo(&repo)?;
            name = name.trim_end_matches(".git").to_string();
            let repo = crate::database::repositories::upsert_repository(
                &pool, &owner, &name, None,
            ).await?;
            println!("Watching {}/{} (id: {})", repo.owner, repo.name, repo.id);
        }
        RepositoryCommands::List => {
            let repos = crate::database::repositories::list_repositories(&pool).await?;
            for r in repos {
                println!("{}/{} (id: {})", r.owner, r.name, r.id);
            }
        }
        RepositoryCommands::Remove { repo } => {
            let (owner, mut name) = parse_repo(&repo)?;
            name = name.trim_end_matches(".git").to_string();
            if let Some(r) = crate::database::repositories::get_repository_by_owner_name(&pool, &owner, &name).await? {
                crate::database::repositories::delete_repository(&pool, r.id).await?;
                println!("Stopped watching {}/{}", owner, name);
            } else {
                println!("{}/{} was not being watched", owner, name);
            }
        }
    }

    pool.close().await;
    Ok(())
}

pub async fn handle_issue(command: IssueCommands) -> anyhow::Result<()> {
    let config = crate::config::Config::from_env()?;
    let pool = crate::database::create_pool(&config.database_url).await?;

    match command {
        IssueCommands::List => {
            let issues = crate::database::issues::list_issues(&pool, None).await?;
            for i in issues {
                let labels: Vec<String> = serde_json::from_value::<Vec<serde_json::Value>>(i.labels.clone())
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
                    .collect();
                println!("#{} {} [{}] {}", i.issue_number, i.title, i.state, labels.join(", "));
            }
        }
        IssueCommands::Analyze { issue } => {
            let (owner, name, number) = parse_issue_ref(&issue)?;
            let repo = crate::database::repositories::get_repository_by_owner_name(&pool, &owner, &name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Repository {}/{} not found. Add it first with 'repository add'", owner, name))?;

            let github = crate::github::client::GitHubClientImpl::new(config.github_token.clone());
            let gh_issue = github.get_issue(&owner, &name, number).await?;

            let labels = serde_json::to_value(&gh_issue.labels)?;
            let db_issue = crate::database::issues::upsert_issue(
                &pool, repo.id, gh_issue.id, gh_issue.number, &gh_issue.title,
                gh_issue.body.as_deref(), &gh_issue.state, Some(&gh_issue.user.login),
                &labels, Some(&gh_issue.html_url),
            ).await?;

            println!("Issue #{} stored (id: {})", db_issue.issue_number, db_issue.id);
            println!("Title: {}", db_issue.title);
            println!("Run 'issue comment {}/{}#{}' to generate a comment", owner, name, number);
        }
        IssueCommands::Comment { issue } => {
            let (owner, name, number) = parse_issue_ref(&issue)?;
            let repo = crate::database::repositories::get_repository_by_owner_name(&pool, &owner, &name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;
            let db_issue = crate::database::issues::get_issue_by_number(&pool, repo.id, number)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Issue not found"))?;

            let _github = crate::github::client::GitHubClientImpl::new(config.github_token.clone());
            let github = std::sync::Arc::new(crate::github::client::GitHubClientImpl::new(config.github_token.clone()));
            let ollama_client = rig::providers::ollama::Client::builder()
                .api_key(rig::providers::ollama::OllamaApiKey::from(""))
                .base_url(&config.ollama_base_url)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to create Ollama client: {}", e))?;
            let analysis_agent = crate::agents::issue_analysis::IssueAnalysisAgent::new(
                &ollama_client,
                &config.ollama_model,
            );
            let context_agent = crate::agents::repository_context::RepositoryContextAgent::new(github.clone());
            let repo_context = context_agent.gather_context(&owner, &name, &github.get_issue(&owner, &name, number).await?).await?;

            let gh_issue = github.get_issue(&owner, &name, number).await?;
            let issue_context = format!("Title: {}\nBody: {}", gh_issue.title, gh_issue.body.as_deref().unwrap_or("(empty)"));

            let analysis = analysis_agent.analyze(&issue_context, &repo_context).await?;
            crate::database::analyses::save_analysis(&pool, db_issue.id, &analysis, None).await?;
            println!("Analysis saved (complexity: {:?})", analysis.complexity);

            let comment_agent = crate::agents::comment_generation::CommentGenerationAgent::new(
                &ollama_client,
                &config.ollama_model,
            );
            let result = crate::services::analysis_service::generate_comment(&pool, &comment_agent, db_issue.id).await?;

            println!("Comment generated (action_id: {})", result["action_id"]);
            println!("Status: {}", result["status"]);
            println!("---");
            println!("{}", result["comment_text"]);
        }
        IssueCommands::CommentApprove { issue } => {
            let (owner, name, number) = parse_issue_ref(&issue)?;
            let repo = crate::database::repositories::get_repository_by_owner_name(&pool, &owner, &name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;
            let db_issue = crate::database::issues::get_issue_by_number(&pool, repo.id, number)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Issue not found"))?;

            let result = crate::approval::service::approve_comment(&pool, db_issue.id, "cli-user").await?;
            println!("Action {} approved", result["action_id"]);
        }
        IssueCommands::CommentPost { issue } => {
            let (owner, name, number) = parse_issue_ref(&issue)?;
            let repo = crate::database::repositories::get_repository_by_owner_name(&pool, &owner, &name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;
            let db_issue = crate::database::issues::get_issue_by_number(&pool, repo.id, number)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Issue not found"))?;

            let github = std::sync::Arc::new(crate::github::client::GitHubClientImpl::new(config.github_token.clone()));
            let result = crate::approval::service::post_comment(&pool, github, db_issue.id).await?;
            println!("Comment posted! GitHub comment ID: {}", result["github_comment_id"]);
        }
    }

    pool.close().await;
    Ok(())
}

fn parse_repo(repo: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("Invalid repository format '{}'. Expected 'owner/repo'", repo);
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn parse_issue_ref(issue: &str) -> anyhow::Result<(String, String, i32)> {
    let parts: Vec<&str> = issue.split('#').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid issue format '{}'. Expected 'owner/repo#123'", issue);
    }
    let (owner, name) = parse_repo(parts[0])?;
    let number: i32 = parts[1].parse().map_err(|_| anyhow::anyhow!("Invalid issue number '{}'", parts[1]))?;
    Ok((owner, name, number))
}

pub async fn handle_setup() -> anyhow::Result<()> {
    println!("=== GitHub Contribution Agent Setup ===\n");

    // Check Ollama
    if crate::agents::ollama_setup::is_ollama_installed() {
        println!("✅ Ollama is installed");
    } else {
        println!("❌ Ollama is not installed.");
        println!("   Install with: curl -fsSL https://ollama.com/install.sh | sh");
        println!("   Then re-run: github-agent setup");
        return Ok(());
    }

    let base_url = "http://localhost:11434";
    if crate::agents::ollama_setup::is_ollama_running(base_url).await {
        println!("✅ Ollama server is running");
    } else {
        println!("Starting Ollama server...");
        crate::agents::ollama_setup::start_ollama_server()?;
        for _ in 0..10 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if crate::agents::ollama_setup::is_ollama_running(base_url).await {
                break;
            }
        }
        if crate::agents::ollama_setup::is_ollama_running(base_url).await {
            println!("✅ Ollama server started");
        } else {
            println!("❌ Ollama server failed to start");
            return Ok(());
        }
    }

    let model = "qwen2.5:7b";
    if crate::agents::ollama_setup::is_model_available(base_url, model).await {
        println!("✅ Model {} is available", model);
    } else {
        println!("Pulling model {}...", model);
        crate::agents::ollama_setup::pull_model(base_url, model).await?;
    }

    // Create config directory
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("github-agent");
    std::fs::create_dir_all(&config_dir)?;

    let env_path = config_dir.join(".env");
    if env_path.exists() {
        println!("\nConfig already exists at {}", env_path.display());
    } else {
        println!("\nCreating config at {}", env_path.display());
        let template = include_str!("../../.env.example");
        std::fs::write(&env_path, template)?;
    }

    println!("\n✅ Setup complete!");
    println!("Next steps:");
    println!("  1. Edit {} with your tokens", env_path.display());
    println!("  2. Run: github-agent db migrate");
    println!("  3. Run: github-agent serve");
    println!("  4. Run: github-agent service install (for auto-start on boot)");

    Ok(())
}

pub async fn handle_service(command: ServiceCommands) -> anyhow::Result<()> {
    match command {
        ServiceCommands::Install => {
            println!("Installing systemd service...");
            let service_content = r#"[Unit]
Description=GitHub Contribution Agent
After=docker.service ollama.service
Requires=docker.service

[Service]
Type=simple
ExecStartPre=/bin/bash -c 'until docker compose exec -T postgres pg_isready -U agent; do sleep 2; done'
ExecStartPre=/bin/bash -c 'until curl -s http://localhost:11434/api/tags > /dev/null; do sleep 2; done'
ExecStart=/usr/local/bin/github-agent serve
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#;
            let service_path = "/etc/systemd/system/github-agent.service";
            std::fs::write(service_path, service_content)?;
            std::process::Command::new("systemctl")
                .args(["daemon-reload"])
                .status()?;
            std::process::Command::new("systemctl")
                .args(["enable", "github-agent"])
                .status()?;
            std::process::Command::new("systemctl")
                .args(["enable", "ollama"])
                .status()
                .ok();
            println!("✅ Service installed and enabled (auto-start on boot)");
            println!("   Start with: github-agent service start");
        }
        ServiceCommands::Start => {
            std::process::Command::new("systemctl")
                .args(["start", "github-agent"])
                .status()?;
            println!("✅ Service started");
        }
        ServiceCommands::Stop => {
            std::process::Command::new("systemctl")
                .args(["stop", "github-agent"])
                .status()?;
            println!("✅ Service stopped");
        }
        ServiceCommands::Status => {
            std::process::Command::new("systemctl")
                .args(["status", "github-agent"])
                .status()?;
        }
        ServiceCommands::Uninstall => {
            std::process::Command::new("systemctl")
                .args(["stop", "github-agent"])
                .status()
                .ok();
            std::process::Command::new("systemctl")
                .args(["disable", "github-agent"])
                .status()
                .ok();
            std::fs::remove_file("/etc/systemd/system/github-agent.service").ok();
            std::process::Command::new("systemctl")
                .args(["daemon-reload"])
                .status()
                .ok();
            println!("✅ Service uninstalled");
        }
    }
    Ok(())
}
