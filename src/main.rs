use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use elmo_api::{create_app, get_db_connection};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up tracing for logging and request/response tracking
    // This configuration can be overridden by setting the RUST_LOG environment variable
    // For Docker deployments, ensure RUST_LOG is set in the Dockerfile or docker-compose.yml
    // Example: ENV RUST_LOG=tower_http=trace,axum=trace,elmo_api=trace
    tracing_subscriber::registry()
        // Configure log levels for different components
        // tower_http: HTTP middleware logging
        // axum: Web framework logging
        // elmo_api: Our application-specific logs
        .with(EnvFilter::new("tower_http=trace,axum=trace,elmo_api=trace"))
        // Add formatting layer with targets enabled for better log readability
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    let pool = get_db_connection().await?;

    let app = create_app(pool).await;

    // run our app with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
