# github-contribution-agent

A private AI assistant that helps developers discover and understand `good first issue` opportunities in GitHub repositories they are interested in.

> **Core principle: "AI analyzes privately. The human decides whether anything becomes public."**

This tool is designed for contributing to repositories owned by **other people or organizations**. It does **not** automatically announce itself as an AI agent, does **not** post comments without approval, and treats all public GitHub interaction as requiring explicit user action.

---

## What It Does

- **Polls** GitHub repositories you're interested in for new/updated issues (no admin access required)
- **Filters** issues by configurable labels (`good first issue`, `good-first-issue`, `beginner`, `help wanted`, etc.)
- **Analyzes** matching issues privately using a local LLM (Qwen 2.5 via Ollama — free, no API key)
- **Notifies** you by email with a structured analysis: summary, requirements, affected areas, possible files, implementation approach, testing requirements, complexity, and uncertainties
- **Generates** suggested comments for issues — stored locally, never posted
- **Requires explicit approval** before any GitHub write action (posting a comment)

## Architecture

```
GitHub Polling (every 5 min)
      │
      v
  IssueFilter            (configurable labels, ignored users, idempotency)
      │
      v
  RepositoryContextAgent  (README, CONTRIBUTING, relevant files)
      │
      v
  IssueAnalysisAgent     (Rig Extractor → structured IssueAnalysis)
      │
      v
  NotificationService    (email via SMTP)
      │
      v
  UserApproval           (explicit approval state machine)
      │
      v
  Optional GitHubAction  (post comment ONLY after approval)
```

---

## Installation

### One-Line Install

```bash
curl -fsSL https://raw.githubusercontent.com/AmalenduManoj/github-contribution-agent/main/install.sh | bash
```

This installs Ollama, pulls Qwen 2.5 7B, builds the binary, and runs the setup wizard.

### Manual Install

```bash
# 1. Install Ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama pull qwen2.5:7b

# 2. Clone and build
git clone https://github.com/AmalenduManoj/github-contribution-agent.git
cd github-contribution-agent
cargo build --release

# 3. Run setup wizard (auto-detects Ollama, creates config)
./target/release/github-agent setup

# 4. Edit config with your tokens
nano ~/.config/github-agent/.env

# 5. Run migrations and start
./target/release/github-agent db migrate
./target/release/github-agent serve
```

### Auto-Start on Boot

```bash
# Install as systemd service (auto-starts on every boot)
sudo cp target/release/github-agent /usr/local/bin/
github-agent service install
github-agent service start

# Check status
github-agent service status

# Stop/uninstall
github-agent service stop
github-agent service uninstall
```

### Configuration

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_TOKEN` | Yes | — | GitHub personal access token (read-only for analysis, write for posting comments) |
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `OLLAMA_BASE_URL` | No | `http://localhost:11434` | Ollama server URL |
| `OLLAMA_MODEL` | No | `qwen2.5:7b` | Model to use for analysis |
| `SMTP_HOST` | No | — | SMTP server (for email notifications) |
| `SMTP_PORT` | No | `587` | SMTP port |
| `SMTP_USER` | No | — | SMTP username |
| `SMTP_PASSWORD` | No | — | SMTP password (use app password) |
| `SMTP_FROM` | No | — | Sender email address |
| `NOTIFICATION_EMAIL` | No | — | Recipient email address |
| `CONTRIBUTION_LABELS` | No | `good first issue,good-first-issue,first issue,beginner,help wanted` | Comma-separated labels to match |
| `IGNORED_USERS` | No | — | Comma-separated GitHub usernames to skip |
| `POLL_INTERVAL_SECS` | No | `300` | Polling interval in seconds |
| `POLL_ENABLED` | No | `true` | Enable/disable polling |
| `HOST` | No | `0.0.0.0` | Server host |
| `PORT` | No | `8080` | Server port |
| `RUST_LOG` | No | `info` | Log level |

---

## Quick Start

### 1. Add a repository

```bash
github-agent repository add owner/repo
```

### 2. Start the agent

```bash
github-agent serve
```

The agent polls GitHub every 5 minutes (configurable via `POLL_INTERVAL_SECS`).

### 3. Analyze an issue

```bash
github-agent issue analyze owner/repo#123
```

### 4. Generate and post a comment

```bash
github-agent issue comment owner/repo#123          # generate (stored as pending)
github-agent issue comment-approve owner/repo#123  # approve
github-agent issue comment-post owner/repo#123     # post to GitHub
```

## CLI Usage

```bash
# Setup and service
github-agent setup
github-agent serve
github-agent service install
github-agent service start
github-agent service stop
github-agent service status
github-agent service uninstall

# Repository management
github-agent repository add owner/repo
github-agent repository list
github-agent repository remove owner/repo

# Issue workflow
github-agent issue list
github-agent issue analyze owner/repo#123
github-agent issue comment owner/repo#123
github-agent issue comment-approve owner/repo#123
github-agent issue comment-post owner/repo#123

# Database
github-agent db migrate
```

## API Endpoints

| Method | Path | Description |
|---|---|---|
| GET | `/health` | Health check |
| GET | `/repositories` | List watched repositories |
| POST | `/repositories` | Add repository to watch |
| DELETE | `/repositories/:id` | Remove repository |
| GET | `/issues` | List issues (with filters) |
| GET | `/issues/:id` | Get issue detail |
| GET | `/issues/:id/analysis` | Get AI analysis |
| POST | `/issues/:id/generate-comment` | Generate comment (no posting) |
| POST | `/issues/:id/approve-comment` | Approve pending comment |
| POST | `/issues/:id/post-comment` | Post approved comment to GitHub |
| GET | `/openapi.json` | OpenAPI spec |

## How Approval Works

The system strictly separates **analysis** from **public action**:

```
generate_comment()  →  DB: status=pending     (no GitHub API call)
approve_comment()   →  DB: status=approved    (no GitHub API call)
post_comment()      →  GitHub API call        (only from approved state)
```

- `generate_comment()` **cannot** post anything — it is a pure function of the analysis
- `post_comment()` is **only valid from the `approved` state**; calling it on a `pending` action is a state-machine error
- Every public action is audited: who approved, when, the exact payload, and the resulting GitHub comment ID
- The agent never posts comments merely because an issue was detected

## Security

1. **No auto-posting** — The agent never posts comments or takes public actions without explicit approval
2. **Secrets in environment variables only** — never logged, never passed to the LLM
3. **LLM prompt safety** — all GitHub content (issue bodies, comments, README, source code) wrapped in `<untrusted_content>` tags; the agent is instructed to treat it as data, never instructions
4. **Least-privilege GitHub token** — read-only for analysis; write access only needed for posting comments
5. **Approval audit trail** — every public action recorded with approver, timestamp, payload, and resulting GitHub comment ID
6. **Idempotency** — duplicate issues are detected and skipped

## Example Workflow

1. **Configure**: `github-agent repository add owner/repo`
2. **Polling** detects a new issue with a matching label (e.g., `good first issue`)
3. **Analysis** runs privately — summary, requirements, affected areas, possible files, implementation approach, testing requirements, complexity, uncertainties, newcomer suitability
4. **Email** sent to developer with full analysis
5. **Developer decides**: "Generate a comment for this issue"
6. System generates comment text and stores it with `status=pending` — **GitHub is not called**
7. **Developer approves**: "Post this comment" → `status=approved`
8. **Only then** does the system call the GitHub API and record the resulting comment ID

## Docker Deployment

```bash
# Build and run
docker compose up --build -d

# Run migrations
docker compose run --rm app db migrate

# Check health
curl http://localhost:8080/health
```

`docker-compose.yml` defines two services:
- `postgres` — PostgreSQL 16 with persistent volume
- `app` — the agent, built from `Dockerfile` (multi-stage Rust build)

## Technology Stack

| Component | Technology |
|---|---|
| Language | Rust |
| Agent framework | [Rig](https://rig.rs) |
| Web server | [Axum](https://github.com/tokio-rs/axum) |
| Database | PostgreSQL + [SQLx](https://github.com/launchbadge/sqlx) |
| HTTP client | [Reqwest](https://github.com/seanmonstar/reqwest) |
| LLM | [Ollama](https://ollama.com) + Qwen 2.5 (free, local) |
| Email | [Lettre](https://lettre.rs) (SMTP) |
| CLI | [Clap](https://github.com/clap-rs/clap) |
| Logging | [Tracing](https://github.com/tokio-rs/tracing) |

## Testing

```bash
cargo test
```

29 tests covering:
- Webhook signature validation (HMAC-SHA256)
- Issue filtering (label matching, ignored users, state checks)
- Structured analysis serialization
- Email notification building
- Approval state machine transitions

No real LLM or GitHub API calls in tests.

---

## Contributing

This project welcomes contributions. To contribute:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run `cargo test` and `cargo check`
5. Submit a pull request

### Development Setup

```bash
# Clone
git clone https://github.com/AmalenduManoj/github-contribution-agent.git
cd github-contribution-agent

# Install dependencies
cargo build

# Run tests
cargo test

# Run the agent locally
cargo run -- serve
```

### Project Structure

```
src/
├── main.rs                  # Binary entry point
├── lib.rs                   # Library root
├── config.rs                # Environment configuration
├── error.rs                 # Error types
├── cli/                     # CLI commands (clap)
├── api/                     # Axum HTTP server + routes
├── github/                  # GitHub API client + webhook parsing
├── agents/                  # Rig-based AI agents
├── notifications/           # Email notification provider
├── approval/                # Approval state machine
├── database/                # PostgreSQL CRUD (SQLx)
├── models/                  # Data structures
└── services/                # Business logic (filtering, analysis, polling)
```

## License

MIT License. See [LICENSE](LICENSE) for details.
