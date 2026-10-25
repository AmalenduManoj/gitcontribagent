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

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.95+
- [Docker](https://docs.docker.com/get-docker/) (for PostgreSQL)
- [Ollama](https://ollama.com/) (for local LLM)

### 1. Install Ollama + Qwen 2.5

```bash
curl -fsSL https://ollama.com/install.sh | sh
ollama pull qwen2.5:14b
```

### 2. Configure

```bash
git clone https://github.com/your-username/github-contribution-agent.git
cd github-contribution-agent
cp .env.example .env
```

Edit `.env` with your settings:

```env
GITHUB_TOKEN=ghp_xxx              # Read-only PAT is fine for analysis
DATABASE_URL=postgres://agent:agent@localhost:5432/github_agent
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=qwen2.5:14b
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=you@gmail.com
SMTP_PASSWORD=app_password
SMTP_FROM=you@gmail.com
NOTIFICATION_EMAIL=you@gmail.com
```

### 3. Run

```bash
# Start PostgreSQL
docker compose up -d postgres

# Run migrations
cargo run -- db migrate

# Start the agent
cargo run -- serve
```

Server starts on `http://localhost:8080`. Polling begins automatically every 5 minutes.

### 4. Watch a repository

```bash
cargo run -- repository add rust-lang/rust
```

## Configuration

| Variable | Required | Default | Description |
|---|---|---|---|
| `GITHUB_TOKEN` | Yes | — | GitHub personal access token (read-only for analysis) |
| `DATABASE_URL` | Yes | — | PostgreSQL connection string |
| `OLLAMA_BASE_URL` | No | `http://localhost:11434` | Ollama server URL |
| `OLLAMA_MODEL` | No | `qwen2.5:14b` | Model to use for analysis |
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

## CLI Usage

```bash
# Start the server
github-agent serve

# Repository management
github-agent repository add owner/repo
github-agent repository list
github-agent repository remove owner/repo

# Issue workflow
github-agent issue list
github-agent issue analyze owner/repo#123
github-agent issue comment owner/repo#123          # generate (no post)
github-agent issue comment-approve owner/repo#123  # approve
github-agent issue comment-post owner/repo#123     # post to GitHub

# Database
github-agent db migrate
```

**The CLI never silently performs a public action.** `comment` generates, `comment-approve` approves, `comment-post` posts — three separate commands.

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

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo test` and `cargo check`
5. Submit a pull request

## License

MIT License. See [LICENSE](LICENSE) for details.