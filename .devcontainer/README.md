# Development Container for elmo-api-rs

This directory contains the configuration for a development container that can be used with GitHub Codespaces or locally with VS Code Dev Containers.

## What's Included

- **Rust toolchain**: Latest stable Rust with cargo and rustfmt
- **VS Code extensions**:
  - rust-analyzer: Rust language server for code intelligence
  - vscode-lldb: Debugger for Rust
  - crates: Dependency management helper
  - even-better-toml: Enhanced TOML support

- **Git & GitHub CLI**: For version control and GitHub integration
- **Pre-configured settings**: 
  - Format on save enabled
  - Clippy linting on save
  - RUST_LOG environment variable set for debugging

## Using with GitHub Codespaces

1. Go to the repository on GitHub
2. Click the green "Code" button
3. Select the "Codespaces" tab
4. Click "Create codespace on main" (or your branch)

GitHub will automatically build the container and set up your development environment.

## Using Locally with VS Code

### Prerequisites
- [Docker](https://www.docker.com/products/docker-desktop)
- [Visual Studio Code](https://code.visualstudio.com/)
- [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers)

### Steps
1. Clone this repository
2. Open the repository in VS Code
3. When prompted, click "Reopen in Container" (or press F1 and select "Dev Containers: Reopen in Container")
4. Wait for the container to build (first time may take a few minutes)

## Post-Create Setup

The container automatically runs `cargo build` after creation to download dependencies and verify the setup.

## Running the Application

Note: The application requires a PostgreSQL database to run. The devcontainer does not include PostgreSQL by default to keep it lightweight. For testing, the test suite uses SQLite in-memory databases and does not require PostgreSQL.

### Running Tests
```bash
cargo test
```

### Running the Server (requires PostgreSQL)
If you have PostgreSQL available:

1. Copy `.env.example` to `.env` and configure your database credentials
2. Run the server:
```bash
cargo run
```

The API will be available at `http://localhost:3000`

## Available Commands

- `cargo check` - Quick syntax and type checking
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo clippy` - Run linter
- `cargo fmt` - Format code
- `cargo run` - Run the application

## Ports

The container forwards port 3000, which is the default port for the elmo-api server.
