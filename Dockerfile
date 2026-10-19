FROM rust:1.95-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY prompts ./prompts

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/github-contribution-agent /usr/local/bin/github-agent
COPY migrations ./migrations
COPY prompts ./prompts

EXPOSE 8080

ENTRYPOINT ["github-agent"]
CMD ["serve"]
