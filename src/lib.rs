pub mod routes;

pub use routes::{TimeRange, Utilization};

use anyhow::Result;
use dotenvy::dotenv;
use sqlx::postgres::{PgConnectOptions, PgPool, PgSslMode};
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

pub async fn get_db_connection() -> Result<PgPool> {
    dotenv().ok();

    let opts = PgConnectOptions::new()
        .host(&env::var("DB_HOST").unwrap())
        .port(5432)
        .database(&env::var("DB_NAME").unwrap())
        .username(&env::var("DB_USER").unwrap())
        .password(&env::var("DB_PASSWORD").unwrap())
        .ssl_mode(PgSslMode::Allow);

    println!("DB_HOST: {}", env::var("DB_HOST").unwrap());
    println!("DB_NAME: {}", env::var("DB_NAME").unwrap());
    println!("DB_USER: {}", env::var("DB_USER").unwrap());
    println!("DB_PASSWORD: {}", env::var("DB_PASSWORD").unwrap());

    let pool = PgPool::connect_with(opts)
        .await
        .expect("Failed to connect to database");

    Ok(pool)
}

pub async fn create_app(pool: PgPool) -> axum::Router {
    use axum::routing::get;
    use routes::{
        get_cpu_utilization, get_daily_cpu_utilization, get_daily_gpu_utilization,
        get_gpu_utilization, get_hourly_cpu_utilization, get_hourly_gpu_utilization, root,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Tower HTTP Tracing Middleware
    // This adds HTTP-specific request/response tracking on top of the global tracing setup in main.rs
    // It works in conjunction with the tracing_subscriber configuration and requires the RUST_LOG
    // environment variable to be properly set when running in Docker
    //
    // - make_span_with: Creates a span for each request with basic request information
    // - on_request: Logs when a request is received, includes method and URI
    // - on_response: Logs when a response is sent, includes status code and latency
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    axum::Router::new()
        .route("/", get(root))
        .route("/cpu", get(get_cpu_utilization))
        .route("/gpu", get(get_gpu_utilization))
        .route("/cpu/hourly", get(get_hourly_cpu_utilization))
        .route("/gpu/hourly", get(get_hourly_gpu_utilization))
        .route("/cpu/daily", get(get_daily_cpu_utilization))
        .route("/gpu/daily", get(get_daily_gpu_utilization))
        .layer(trace_layer) // Add trace layer before cors
        .layer(cors)
        .with_state(pool)
}
