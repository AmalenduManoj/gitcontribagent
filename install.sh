#!/usr/bin/env bash
set -euo pipefail

# GitHub Contribution Agent - One-line installer
# Usage: curl -fsSL https://raw.githubusercontent.com/your-username/github-contribution-agent/main/install.sh | bash

REPO="your-username/github-contribution-agent"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="github-agent"
CONFIG_DIR="${HOME}/.config/github-agent"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  ARTIFACT="github-agent-linux-amd64" ;;
    aarch64) ARTIFACT="github-agent-linux-arm64" ;;
    *)       error "Unsupported architecture: $ARCH" ;;
esac

# Check for sudo
if [ "$(id -u)" -ne 0 ]; then
    SUDO="sudo"
else
    SUDO=""
fi

echo "=== GitHub Contribution Agent Installer ==="
echo ""

# 1. Check/install Ollama
info "Checking for Ollama..."
if ! command -v ollama &> /dev/null; then
    info "Installing Ollama..."
    curl -fsSL https://ollama.com/install.sh | sh
fi
echo "✅ Ollama installed"

# 2. Start Ollama and pull model
info "Starting Ollama server..."
ollama serve > /tmp/ollama-install.log 2>&1 &
sleep 3

if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    warn "Ollama server not responding. Starting manually..."
    nohup ollama serve > /tmp/ollama.log 2>&1 &
    sleep 5
fi

MODEL="qwen2.5:7b"
info "Checking for model $MODEL..."
if ! ollama list 2>/dev/null | grep -q "$MODEL"; then
    info "Pulling model $MODEL (this may take a while)..."
    ollama pull "$MODEL"
fi
echo "✅ Model $MODEL available"

# 3. Download binary
info "Downloading github-agent binary..."
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep -o '"tag_name": "[^"]*"' | cut -d'"' -f4)
if [ -z "$LATEST_RELEASE" ]; then
    error "Could not fetch latest release. Building from source instead..."
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/$ARTIFACT"
info "Downloading from $DOWNLOAD_URL"

$SUDO curl -fsSL "$DOWNLOAD_URL" -o "$INSTALL_DIR/$BINARY_NAME"
$SUDO chmod +x "$INSTALL_DIR/$BINARY_NAME"
echo "✅ Binary installed to $INSTALL_DIR/$BINARY_NAME"

# 4. Create config directory
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/.env" ]; then
    cat > "$CONFIG_DIR/.env" << 'ENVEOF'
# Server
HOST=0.0.0.0
PORT=8080

# Database (update with your PostgreSQL URL)
DATABASE_URL=postgres://agent:agent@localhost:5432/github_agent

# GitHub
GITHUB_TOKEN=
GITHUB_WEBHOOK_SECRET=secret

# LLM (Ollama + Qwen 2.5)
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=qwen2.5:7b

# Email (SMTP) - optional
SMTP_HOST=
SMTP_PORT=587
SMTP_USER=
SMTP_PASSWORD=
SMTP_FROM=
NOTIFICATION_EMAIL=

# App
CONTRIBUTION_LABELS=good first issue,good-first-issue,first issue,beginner,help wanted
IGNORED_USERS=
POLL_INTERVAL_SECS=300
POLL_ENABLED=true
RUST_LOG=info
ENVEOF
    echo "✅ Config created at $CONFIG_DIR/.env"
fi

# 5. Run setup wizard
info "Running setup wizard..."
$BINARY_NAME setup

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "  1. Edit $CONFIG_DIR/.env with your GitHub token and SMTP credentials"
echo "  2. Run: github-agent db migrate"
echo "  3. Run: github-agent serve"
echo "  4. Run: github-agent service install (for auto-start on boot)"
echo ""