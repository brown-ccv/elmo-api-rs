use axum::{
    body::Body,
    extract::{Query, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use elmo_api::routes::{TimeRange, Utilization};
use http_body_util::BodyExt;
use sqlx::sqlite::SqlitePool;
use tower::ServiceExt;

// SQLite-compatible route functions for testing
async fn get_cpu_utilization_sqlite(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start.is_some() && time_range.end.is_some() {
        r#"
        SELECT 
            time, 
            allocated, 
            total 
        FROM  
            cpu 
        WHERE time >= ?1 AND time <= ?2
        ORDER BY time
        "#
    } else {
        r#"
        SELECT 
            time, 
            allocated, 
            total 
        FROM  
            cpu 
        ORDER BY time
        "#
    };

    let rows = if let (Some(start), Some(end)) = (time_range.start, time_range.end) {
        sqlx::query_as::<_, Utilization>(query)
            .bind(start.format("%Y-%m-%dT%H:%M:%S").to_string())
            .bind(end.format("%Y-%m-%dT%H:%M:%S").to_string())
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as::<_, Utilization>(query)
            .fetch_all(&pool)
            .await
    };

    match rows {
        Ok(utilizations) => Json(utilizations).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_gpu_utilization_sqlite(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start.is_some() && time_range.end.is_some() {
        r#"
        SELECT 
            time, 
            allocated, 
            total 
        FROM  
            gpu 
        WHERE time >= ?1 AND time <= ?2
        ORDER BY time
        "#
    } else {
        r#"
        SELECT 
            time, 
            allocated, 
            total 
        FROM  
            gpu 
        ORDER BY time
        "#
    };

    let rows = if let (Some(start), Some(end)) = (time_range.start, time_range.end) {
        sqlx::query_as::<_, Utilization>(query)
            .bind(start.format("%Y-%m-%dT%H:%M:%S").to_string())
            .bind(end.format("%Y-%m-%dT%H:%M:%S").to_string())
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as::<_, Utilization>(query)
            .fetch_all(&pool)
            .await
    };

    match rows {
        Ok(utilizations) => Json(utilizations).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_hourly_cpu_utilization_sqlite(
    State(pool): State<SqlitePool>,
    Query(_time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = r#"
        SELECT 
            strftime('%Y-%m-%dT%H:00:00', time) as time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM cpu
        GROUP BY strftime('%Y-%m-%dT%H:00:00', time)
        ORDER BY time
    "#;

    let rows = sqlx::query_as::<_, Utilization>(query)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(utilizations) => Json(utilizations).into_response(),
        Err(e) => {
            eprintln!("Error in get_hourly_cpu_utilization_sqlite: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_daily_cpu_utilization_sqlite(
    State(pool): State<SqlitePool>,
    Query(_time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = r#"
        SELECT 
            strftime('%Y-%m-%dT00:00:00', time) as time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM cpu
        GROUP BY strftime('%Y-%m-%d', time)
        ORDER BY time
    "#;

    let rows = sqlx::query_as::<_, Utilization>(query)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(utilizations) => Json(utilizations).into_response(),
        Err(e) => {
            eprintln!("Error in get_daily_cpu_utilization_sqlite: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_hourly_gpu_utilization_sqlite(
    State(pool): State<SqlitePool>,
    Query(_time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = r#"
        SELECT 
            strftime('%Y-%m-%dT%H:00:00', time) as time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM gpu
        GROUP BY strftime('%Y-%m-%dT%H:00:00', time)
        ORDER BY time
    "#;

    let rows = sqlx::query_as::<_, Utilization>(query)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(utilizations) => Json(utilizations).into_response(),
        Err(e) => {
            eprintln!("Error in get_hourly_gpu_utilization_sqlite: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn get_daily_gpu_utilization_sqlite(
    State(pool): State<SqlitePool>,
    Query(_time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = r#"
        SELECT 
            strftime('%Y-%m-%dT00:00:00', time) as time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM gpu
        GROUP BY strftime('%Y-%m-%d', time)
        ORDER BY time
    "#;

    let rows = sqlx::query_as::<_, Utilization>(query)
        .fetch_all(&pool)
        .await;

    match rows {
        Ok(utilizations) => Json(utilizations).into_response(),
        Err(e) => {
            eprintln!("Error in get_daily_gpu_utilization_sqlite: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn root_test() -> &'static str {
    "Hello, World!"
}

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Create test tables
    sqlx::query(
        r#"
        CREATE TABLE cpu (
            time TEXT NOT NULL,
            allocated INTEGER NOT NULL,
            total INTEGER NOT NULL
        );
        CREATE TABLE gpu (
            time TEXT NOT NULL,
            allocated INTEGER NOT NULL,
            total INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert test data
    sqlx::query(
        r#"
        INSERT INTO cpu (time, allocated, total) VALUES
            ('2024-03-27T00:00:00', 75, 100),
            ('2024-03-27T00:15:00', 80, 100),
            ('2024-03-27T00:30:00', 85, 100),
            ('2024-03-27T00:45:00', 90, 100);
        INSERT INTO gpu (time, allocated, total) VALUES
            ('2024-03-27T00:00:00', 60, 100),
            ('2024-03-27T00:15:00', 65, 100),
            ('2024-03-27T00:30:00', 70, 100),
            ('2024-03-27T00:45:00', 75, 100);
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn create_test_app() -> axum::Router {
    use axum::routing::get;
    use tower_http::cors::{Any, CorsLayer};

    let pool = setup_test_db().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    axum::Router::new()
        .route("/", get(root_test))
        .route("/cpu", get(get_cpu_utilization_sqlite))
        .route("/gpu", get(get_gpu_utilization_sqlite))
        .route("/cpu/hourly", get(get_hourly_cpu_utilization_sqlite))
        .route("/gpu/hourly", get(get_hourly_gpu_utilization_sqlite))
        .route("/cpu/daily", get(get_daily_cpu_utilization_sqlite))
        .route("/gpu/daily", get(get_daily_gpu_utilization_sqlite))
        .layer(cors)
        .with_state(pool)
}

async fn get_body_bytes(response: Response) -> Vec<u8> {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    bytes.to_vec()
}

#[tokio::test]
async fn test_get_cpu_utilization() {
    let app = create_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(cpu_data.len(), 4);
    assert_eq!(cpu_data[0].allocated, Some(75));
    assert_eq!(cpu_data[0].total, Some(100));
}

#[tokio::test]
async fn test_get_hourly_cpu_utilization() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/hourly")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(hourly_data.len(), 1); // All data is in one hour
    assert_eq!(hourly_data[0].allocated, Some(83)); // ROUND((75 + 80 + 85 + 90) / 4)
    assert_eq!(hourly_data[0].total, Some(100));
}

#[tokio::test]
async fn test_get_daily_cpu_utilization() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/daily")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(daily_data.len(), 1); // All data is in one day
    assert_eq!(daily_data[0].allocated, Some(83)); // ROUND((75 + 80 + 85 + 90) / 4)
    assert_eq!(daily_data[0].total, Some(100));
}

#[tokio::test]
async fn test_get_gpu_utilization() {
    let app = create_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/gpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let gpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(gpu_data.len(), 4);
    assert_eq!(gpu_data[0].allocated, Some(60));
    assert_eq!(gpu_data[0].total, Some(100));
}

#[tokio::test]
async fn test_get_hourly_gpu_utilization() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/gpu/hourly")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(hourly_data.len(), 1); // All data is in one hour
    assert_eq!(hourly_data[0].allocated, Some(68)); // ROUND((60 + 65 + 70 + 75) / 4)
    assert_eq!(hourly_data[0].total, Some(100));
}

#[tokio::test]
async fn test_get_daily_gpu_utilization() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/gpu/daily")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(daily_data.len(), 1); // All data is in one day
    assert_eq!(daily_data[0].allocated, Some(68)); // ROUND((60 + 65 + 70 + 75) / 4)
    assert_eq!(daily_data[0].total, Some(100));
}

#[tokio::test]
async fn test_time_range_filtering() {
    let app = create_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?start=2024-03-27T00:00:00&end=2024-03-27T00:30:00")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    println!("cpu_data: {:?}", cpu_data);

    assert_eq!(cpu_data.len(), 3); // Only first 3 entries within time range
}
