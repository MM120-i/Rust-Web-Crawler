# Use PowerShell to run recipes on Windows instead of hunting for a `sh` on PATH
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# List available commands (this is what runs if you just type `just`)
default:
    @just --list

# Static checks that mirror CI's fmt + clippy jobs
check:
    cargo fmt --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Auto-fix formatting
fmt:
    cargo fmt

# Run the full test suite
test:
    cargo test --workspace

# Start local Postgres (docker compose, with the health check)
db:
    docker compose up -d

# Stop local Postgres
db-down:
    docker compose down

# Run the CLI's current example crawl
run-example:
    cargo run -p crawler-cli

# Release build across the workspace
build:
    cargo build --workspace --release
