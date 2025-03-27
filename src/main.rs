use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::env;

use elmo_api::create_app;

fn get_database_url() -> Result<String> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    Ok(database_url)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv()?;

    let database_url = get_database_url()?;
    println!("database_url: {}", database_url);

    let pool = SqlitePool::connect(&database_url).await.unwrap();

    let app = create_app(pool).await;

    // run our app with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
