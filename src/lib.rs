pub mod routes;

pub use routes::{TimeRange, Utilization};

use sqlx::sqlite::SqlitePool;
use tower_http::cors::{Any, CorsLayer};

pub async fn create_app(pool: SqlitePool) -> axum::Router {
    use axum::routing::get;
    use routes::{
        get_cpu_utilization, get_daily_cpu_utilization, get_daily_gpu_utilization,
        get_gpu_utilization, get_hourly_cpu_utilization, get_hourly_gpu_utilization, root,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    axum::Router::new()
        .route("/", get(root))
        .route("/cpu", get(get_cpu_utilization))
        .route("/gpu", get(get_gpu_utilization))
        .route("/cpu/hourly", get(get_hourly_cpu_utilization))
        .route("/gpu/hourly", get(get_hourly_gpu_utilization))
        .route("/cpu/daily", get(get_daily_cpu_utilization))
        .route("/gpu/daily", get(get_daily_gpu_utilization))
        .layer(cors)
        .with_state(pool)
}
