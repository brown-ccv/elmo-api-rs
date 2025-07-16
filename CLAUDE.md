# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust API for elmo (elegant live monitoring of Oscar) that provides HTTP endpoints for retrieving CPU and GPU utilization data from a PostgreSQL database. The API supports real-time, hourly, and daily aggregation of utilization metrics.

## Development Commands

- **Run the server**: `cargo run` (starts on localhost:3000)
- **Run tests**: `cargo test`
- **Build**: `cargo build`
- **Build release**: `cargo build --release`
- **Check for errors**: `cargo check`

## Architecture

### Core Components

- **main.rs**: Application entry point with tracing setup and server initialization
- **lib.rs**: Database connection setup and app creation with middleware layers
- **routes.rs**: API endpoint handlers for CPU/GPU utilization queries

### Database Architecture

- **Production**: PostgreSQL with `oscar.cpu` and `oscar.gpu` tables
- **Tests**: SQLite in-memory database with simplified table structure
- **Connection**: Uses sqlx with connection pooling via `PgPool`

### API Endpoints

All endpoints support optional `start` and `end` query parameters for time range filtering:

- `GET /` - Health check
- `GET /cpu` - Raw CPU utilization data
- `GET /gpu` - Raw GPU utilization data  
- `GET /cpu/hourly` - CPU data aggregated by hour
- `GET /gpu/hourly` - GPU data aggregated by hour
- `GET /cpu/daily` - CPU data aggregated by day
- `GET /gpu/daily` - GPU data aggregated by day

### Environment Configuration

The application requires these environment variables (see .env.example):
- `DB_HOST` - PostgreSQL host
- `DB_NAME` - Database name
- `DB_USER` - Database username
- `DB_PASSWORD` - Database password

Uses dotenvy for loading .env files in development.

### Testing Strategy

- Unit tests use SQLite with in-memory databases for speed
- Production routes are duplicated as SQLite-compatible versions in tests
- Tests include both time-filtered and unfiltered data queries
- All tests run with `cargo test`


## Code Style and Conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer functional programming patterns
- Always prioritize performance, readability, and maintainability
