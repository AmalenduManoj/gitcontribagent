.PHONY: build release test check install service-setup clean

# Build debug
build:
	cargo build

# Build release
release:
	cargo build --release

# Run tests
test:
	cargo test

# Check compilation
check:
	cargo check

# Install binary to /usr/local/bin
install: release
	sudo cp target/release/github-contribution-agent /usr/local/bin/github-agent
	sudo chmod +x /usr/local/bin/github-agent
	@echo "Installed to /usr/local/bin/github-agent"

# Run setup wizard
setup: install
	github-agent setup

# Install as systemd service (auto-start on boot)
service-install: install
	sudo github-agent service install
	sudo github-agent service start

# Run migrations
migrate:
	github-agent db migrate

# Start server
serve:
	github-agent serve

# Clean build artifacts
clean:
	cargo clean

# Build for all targets
build-all:
	cargo build --release --target x86_64-unknown-linux-gnu
	cargo build --release --target aarch64-unknown-linux-gnu

# Package release tarball
package: release
	mkdir -p dist
	tar -czf dist/github-agent-$(shell git describe --tags --always).tar.gz \
		-C target/release github-contribution-agent \
		migrations prompts
	@echo "Package created in dist/"