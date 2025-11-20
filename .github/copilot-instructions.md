# Copilot Instructions for elmo-api-rs

## Project Overview

This is a Rust-based REST API for **elmo** (elegant live monitoring of Oscar), a system that provides HTTP endpoints for querying CPU and GPU utilization data from a PostgreSQL database. The API supports real-time data retrieval and aggregated metrics (hourly and daily).

**Repository Stats:**
- **Language:** Rust (Edition 2021)
- **Lines of Code:** ~3,600 lines of Rust code
- **Project Type:** Web API (Axum framework)
- **Database:** PostgreSQL (production), SQLite (tests)
- **License:** MIT

## Build & Development Commands

### Prerequisites
- Rust toolchain (tested with rustc 1.91.1 and cargo 1.91.1)
- PostgreSQL database (for production; not required for tests)

### Command Reference

**ALWAYS run commands in this exact order to avoid failures:**

1. **Check code (fastest validation):**
   ```bash
   cargo check
   ```
   - Time: ~1-2 seconds (cached), ~40-60 seconds (first run)
   - Use this for quick syntax/type checking before building

2. **Run tests (ALWAYS before committing):**
   ```bash
   cargo test
   ```
   - Time: ~40 seconds (first run), ~3-5 seconds (cached)
   - All tests use SQLite in-memory databases (no PostgreSQL required)
   - Runs 58 tests across unit, integration, end-to-end, and performance suites
   - Tests MUST pass before any PR can be merged

3. **Lint with Clippy (required for CI):**
   ```bash
   cargo clippy -- -D warnings
   ```
   - Time: ~1-2 seconds (cached)
   - Treats all warnings as errors (CI requirement)
   - MUST pass with zero warnings before PR submission

4. **Format check (required for CI):**
   ```bash
   cargo fmt --all -- --check
   ```
   - Time: <1 second
   - Verifies code follows Rust formatting standards
   - MUST pass before PR submission

5. **Auto-format code:**
   ```bash
   cargo fmt --all
   ```
   - Time: <1 second
   - Apply this before committing code changes

6. **Build (development):**
   ```bash
   cargo build
   ```
   - Time: ~40 seconds (first run), ~1-5 seconds (incremental)
   - Creates unoptimized binary at `target/debug/elmo-api`

7. **Build (release/production):**
   ```bash
   cargo build --release
   ```
   - Time: ~100-120 seconds (first run)
   - Creates optimized binary at `target/release/elmo-api`
   - Only use for production deployments

8. **Run the server (requires PostgreSQL):**
   ```bash
   cargo run
   ```
   - Starts HTTP server on `0.0.0.0:3000`
   - Requires `.env` file with PostgreSQL credentials (see `.env.example`)
   - Use Ctrl+C to stop the server

### Environment Setup

**For running the server (not required for tests):**
1. Copy `.env.example` to `.env`
2. Set these environment variables:
   - `DB_HOST` - PostgreSQL host
   - `DB_NAME` - Database name
   - `DB_USER` - Database username  
   - `DB_PASSWORD` - Database password

**Tests do NOT require environment setup** - they use SQLite in-memory databases.

## CI/CD Requirements

The GitHub Actions workflow (`.github/workflows/test.yml`) runs on all pull requests to `main` and executes these checks **in order**:

1. `cargo test --verbose`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --all -- --check`

**All three checks MUST pass for PR approval.** The workflow uses Rust stable toolchain with caching enabled.

## Project Architecture

### Directory Structure

```
elmo-api-rs/
├── src/                    # Source code
│   ├── main.rs            # Entry point, tracing setup, server startup
│   ├── lib.rs             # Database connection, app creation, middleware
│   └── routes.rs          # API endpoints and request handlers
├── tests/                  # Test suites (all use SQLite)
│   ├── api_tests.rs       # API endpoint tests
│   ├── end_to_end_tests.rs # E2E scenarios  
│   ├── integration_tests.rs # Integration tests
│   └── performance_tests.rs # Performance benchmarks
├── sql/                    # SQL scripts
│   └── create_service_account.sql # PostgreSQL setup
├── scripts/gcp/           # Google Cloud Platform deployment scripts
├── data/                   # SQLite database (for local testing)
├── .github/workflows/     # CI/CD workflows
├── Cargo.toml             # Rust package manifest
├── Cargo.lock             # Dependency lock file
├── Dockerfile             # Container build configuration
└── .env.example           # Environment variable template
```

### Core Components

**`src/main.rs`** (35 lines)
- Initializes tracing/logging with `tracing-subscriber`
- Calls `get_db_connection()` to connect to PostgreSQL
- Calls `create_app()` to build the Axum router
- Starts HTTP server on port 3000

**`src/lib.rs`** (70 lines)
- `get_db_connection()`: Creates PostgreSQL connection pool using environment variables
- `create_app()`: Builds Axum router with routes, CORS, and tracing middleware
- Exports `TimeRange` and `Utilization` types from routes module

**`src/routes.rs`** (831 lines)
- Defines `Utilization` struct (time, allocated, total fields)
- Defines `TimeRange` struct for query parameters
- 7 route handlers:
  - `root()` - Health check endpoint
  - `get_cpu_utilization()` - Raw CPU data
  - `get_gpu_utilization()` - Raw GPU data
  - `get_hourly_cpu_utilization()` - Hourly CPU aggregates
  - `get_hourly_gpu_utilization()` - Hourly GPU aggregates
  - `get_daily_cpu_utilization()` - Daily CPU aggregates
  - `get_daily_gpu_utilization()` - Daily GPU aggregates
- Contains unit tests (lines 424-830) with SQLite-compatible implementations

### API Endpoints

All endpoints support optional `start` and `end` query parameters (ISO 8601 timestamps):

- `GET /` - Health check (returns "Hello, World!")
- `GET /cpu?start=<timestamp>&end=<timestamp>` - Raw CPU utilization
- `GET /gpu?start=<timestamp>&end=<timestamp>` - Raw GPU utilization
- `GET /cpu/hourly?start=<timestamp>&end=<timestamp>` - Hourly CPU averages
- `GET /gpu/hourly?start=<timestamp>&end=<timestamp>` - Hourly GPU averages
- `GET /cpu/daily?start=<timestamp>&end=<timestamp>` - Daily CPU averages
- `GET /gpu/daily?start=<timestamp>&end=<timestamp>` - Daily GPU averages

### Database Schema

**Production (PostgreSQL):**
- Schema: `oscar`
- Tables: `oscar.cpu`, `oscar.gpu`
- Columns: `time` (timestamp), `allocated` (int), `total` (int)

**Tests (SQLite):**
- No schema prefix
- Tables: `cpu`, `gpu`  
- Same columns but time stored as TEXT

### Key Dependencies

- **axum** (0.8) - Web framework
- **sqlx** (0.8) - SQL toolkit with PostgreSQL and SQLite support
- **tokio** (1.0) - Async runtime
- **tower-http** (0.5) - HTTP middleware (CORS, tracing)
- **tracing** / **tracing-subscriber** - Structured logging
- **serde** / **serde_json** - JSON serialization
- **chrono** (0.4) - Date/time handling
- **dotenvy** (0.15) - `.env` file loading

## Common Patterns & Conventions

### Code Style
- Use `cargo fmt` for all formatting (enforced by CI)
- Zero Clippy warnings allowed (enforced by CI)
- Prefer functional programming patterns
- Prioritize performance, readability, and maintainability

### Database Query Pattern
All production routes follow this pattern:
1. Accept `State(pool): State<PgPool>` and `Query(time_range): Query<TimeRange>`
2. Build SQL query with optional time filtering
3. Execute query with `sqlx::query_as::<_, Utilization>()`
4. Handle errors and log with `tracing::error!`
5. Return JSON response

### Testing Pattern
- Tests duplicate production routes as SQLite-compatible versions
- Use `setup_test_db()` to create in-memory SQLite database with test data
- Convert `Response` to bytes with `get_body_bytes()` helper
- Parse JSON and assert values match expected results

## Important Notes

### DO NOT:
- Add dependencies without checking for vulnerabilities
- Modify the database schema (it's managed externally)
- Change API response formats (breaking change for clients)
- Skip running tests before committing
- Commit code that doesn't pass `cargo clippy -- -D warnings`
- Commit unformatted code (run `cargo fmt --all` first)

### ALWAYS:
1. Run `cargo check` first for quick validation
2. Run `cargo test` before committing ANY code changes
3. Run `cargo clippy -- -D warnings` to verify linting
4. Run `cargo fmt --all` before committing to ensure formatting
5. Test time-range filtering when adding/modifying endpoints
6. Use SQLite syntax in test routes (different from PostgreSQL)
7. Include both filtered and unfiltered test cases

### Known Quirks
- **PostgreSQL vs SQLite:** Production uses PostgreSQL-specific functions (`date_trunc()`) which don't exist in SQLite. Tests use `strftime()` instead.
- **Time formatting:** PostgreSQL timestamps vs SQLite TEXT requires different query syntax
- **Schema prefix:** Production queries use `oscar.cpu`, tests use just `cpu`
- **Build time:** First build takes ~40s, incremental builds are much faster (~1-2s)
- **Release builds:** Take ~100-120 seconds due to optimizations

## Troubleshooting

### Build fails with missing dependencies
```bash
cargo clean
cargo update
cargo build
```

### Tests fail with database errors
Tests should NEVER require PostgreSQL. If you see PostgreSQL connection errors in tests, you're likely calling production route functions instead of the SQLite test versions.

### Clippy warnings
Fix all warnings - they are treated as errors in CI. Run:
```bash
cargo clippy --fix -- -D warnings
```

### Format check fails
Run auto-format:
```bash
cargo fmt --all
```

### Server won't start
1. Verify `.env` file exists with correct PostgreSQL credentials
2. Verify PostgreSQL database is accessible
3. Check that port 3000 is not already in use

## Quick Reference Files

- **README.md** - Basic project description and usage
- **CLAUDE.md** - Claude AI-specific project guidance  
- **Cargo.toml** - Dependencies and package metadata
- **.env.example** - Environment variable template (copy to `.env`)
- **.gitignore** - Excludes `/target`, `.env`, `*.db-shm`, `*.db-wal`, `secrets/`
- **Dockerfile** - Multi-stage container build (Rust 1.86)

## Trust These Instructions

These instructions have been validated by running all commands successfully. Only search for additional information if these instructions are incomplete or you encounter errors not documented here.
