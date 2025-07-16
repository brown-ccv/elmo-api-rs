use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use elmo_api::routes::{TimeRange, Utilization};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use std::time::{Duration, Instant};
use tower::ServiceExt;

async fn setup_performance_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

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

    let mut cpu_inserts = Vec::new();
    let mut gpu_inserts = Vec::new();
    
    for i in 0..1000 {
        let hour = i / 4;
        let minute = (i % 4) * 15;
        let day = hour / 24;
        let hour_of_day = hour % 24;
        
        let time = format!("2024-03-{:02}T{:02}:{:02}:00", day + 1, hour_of_day, minute);
        
        cpu_inserts.push(format!("('{}', {}, 100)", time, 50 + (i % 50)));
        gpu_inserts.push(format!("('{}', {}, 100)", time, 40 + (i % 60)));
    }
    
    let cpu_query = format!(
        "INSERT INTO cpu (time, allocated, total) VALUES {}",
        cpu_inserts.join(", ")
    );
    
    let gpu_query = format!(
        "INSERT INTO gpu (time, allocated, total) VALUES {}",
        gpu_inserts.join(", ")
    );
    
    sqlx::query(&cpu_query).execute(&pool).await.unwrap();
    sqlx::query(&gpu_query).execute(&pool).await.unwrap();

    pool
}

async fn create_performance_test_app() -> axum::Router {
    use axum::routing::get;
    use axum::{
        extract::{Query, State},
        response::IntoResponse,
        Json,
    };
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
    use tracing::Level;

    let pool = setup_performance_test_db().await;

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
        Query(time_range): Query<TimeRange>,
    ) -> impl IntoResponse {
        let query = if time_range.start.is_some() && time_range.end.is_some() {
            r#"
            SELECT 
                strftime('%Y-%m-%dT%H:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM cpu
            WHERE time >= ?1 AND time <= ?2
            GROUP BY strftime('%Y-%m-%dT%H:00:00', time)
            ORDER BY time
            "#
        } else {
            r#"
            SELECT 
                strftime('%Y-%m-%dT%H:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM cpu
            GROUP BY strftime('%Y-%m-%dT%H:00:00', time)
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

    async fn get_daily_cpu_utilization_sqlite(
        State(pool): State<SqlitePool>,
        Query(time_range): Query<TimeRange>,
    ) -> impl IntoResponse {
        let query = if time_range.start.is_some() && time_range.end.is_some() {
            r#"
            SELECT 
                strftime('%Y-%m-%dT00:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM cpu
            WHERE time >= ?1 AND time <= ?2
            GROUP BY strftime('%Y-%m-%d', time)
            ORDER BY time
            "#
        } else {
            r#"
            SELECT 
                strftime('%Y-%m-%dT00:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM cpu
            GROUP BY strftime('%Y-%m-%d', time)
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

    async fn get_hourly_gpu_utilization_sqlite(
        State(pool): State<SqlitePool>,
        Query(time_range): Query<TimeRange>,
    ) -> impl IntoResponse {
        let query = if time_range.start.is_some() && time_range.end.is_some() {
            r#"
            SELECT 
                strftime('%Y-%m-%dT%H:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM gpu
            WHERE time >= ?1 AND time <= ?2
            GROUP BY strftime('%Y-%m-%dT%H:00:00', time)
            ORDER BY time
            "#
        } else {
            r#"
            SELECT 
                strftime('%Y-%m-%dT%H:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM gpu
            GROUP BY strftime('%Y-%m-%dT%H:00:00', time)
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

    async fn get_daily_gpu_utilization_sqlite(
        State(pool): State<SqlitePool>,
        Query(time_range): Query<TimeRange>,
    ) -> impl IntoResponse {
        let query = if time_range.start.is_some() && time_range.end.is_some() {
            r#"
            SELECT 
                strftime('%Y-%m-%dT00:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM gpu
            WHERE time >= ?1 AND time <= ?2
            GROUP BY strftime('%Y-%m-%d', time)
            ORDER BY time
            "#
        } else {
            r#"
            SELECT 
                strftime('%Y-%m-%dT00:00:00', time) as time,
                CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
                CAST(ROUND(AVG(total)) AS INTEGER) as total
            FROM gpu
            GROUP BY strftime('%Y-%m-%d', time)
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

    async fn root_test() -> &'static str {
        "Hello, World!"
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    axum::Router::new()
        .route("/", get(root_test))
        .route("/cpu", get(get_cpu_utilization_sqlite))
        .route("/gpu", get(get_gpu_utilization_sqlite))
        .route("/cpu/hourly", get(get_hourly_cpu_utilization_sqlite))
        .route("/gpu/hourly", get(get_hourly_gpu_utilization_sqlite))
        .route("/cpu/daily", get(get_daily_cpu_utilization_sqlite))
        .route("/gpu/daily", get(get_daily_gpu_utilization_sqlite))
        .layer(trace_layer)
        .layer(cors)
        .with_state(pool)
}

async fn get_body_bytes(response: Response) -> Vec<u8> {
    let body = response.into_body();
    let bytes = body.collect().await.unwrap().to_bytes();
    bytes.to_vec()
}

#[tokio::test]
async fn test_large_dataset_query_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration < Duration::from_millis(500), "Query took too long: {:?}", duration);
    
    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert_eq!(cpu_data.len(), 1000);
}

#[tokio::test]
async fn test_hourly_aggregation_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/hourly")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration < Duration::from_millis(500), "Hourly aggregation took too long: {:?}", duration);
    
    let body = get_body_bytes(response).await;
    let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert!(hourly_data.len() > 0);
}

#[tokio::test]
async fn test_daily_aggregation_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/daily")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration < Duration::from_millis(500), "Daily aggregation took too long: {:?}", duration);
    
    let body = get_body_bytes(response).await;
    let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert!(daily_data.len() > 0);
}

#[tokio::test]
async fn test_time_range_query_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?start=2024-03-01T00:00:00&end=2024-03-15T23:59:59")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let duration = start.elapsed();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration < Duration::from_millis(500), "Time range query took too long: {:?}", duration);
    
    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert!(cpu_data.len() > 0);
}

#[tokio::test]
async fn test_concurrent_requests_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    
    let tasks = (0..50).map(|i| {
        let app = app.clone();
        let endpoint = match i % 6 {
            0 => "/cpu",
            1 => "/gpu",
            2 => "/cpu/hourly",
            3 => "/gpu/hourly",
            4 => "/cpu/daily",
            _ => "/gpu/daily",
        }.to_string();
        
        tokio::spawn(async move {
            app.oneshot(Request::builder().uri(&endpoint).body(Body::empty()).unwrap())
                .await
                .unwrap()
        })
    });
    
    let responses = futures::future::join_all(tasks).await;
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_secs(2), "Concurrent requests took too long: {:?}", duration);
    
    for response in responses {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_memory_usage_large_dataset() {
    let app = create_performance_test_app().await;
    
    let mut total_memory_size = 0;
    
    for _ in 0..10 {
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = get_body_bytes(response).await;
        total_memory_size += body.len();
        
        let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
        assert_eq!(cpu_data.len(), 1000);
    }
    
    assert!(total_memory_size > 0);
}

#[tokio::test]
async fn test_sequential_requests_performance() {
    let app = create_performance_test_app().await;
    
    let endpoints = [
        "/cpu",
        "/gpu",
        "/cpu/hourly",
        "/gpu/hourly",
        "/cpu/daily",
        "/gpu/daily",
    ];
    
    let start = Instant::now();
    
    for endpoint in &endpoints {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(*endpoint).body(Body::empty()).unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    let duration = start.elapsed();
    assert!(duration < Duration::from_secs(1), "Sequential requests took too long: {:?}", duration);
}

#[tokio::test]
async fn test_response_size_limits() {
    let app = create_performance_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    
    assert!(body.len() < 1024 * 1024);
    
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert_eq!(cpu_data.len(), 1000);
}

#[tokio::test]
async fn test_stress_test_multiple_endpoints() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    
    let tasks = (0..100).map(|i| {
        let app = app.clone();
        let endpoint = match i % 6 {
            0 => "/cpu",
            1 => "/gpu",
            2 => "/cpu/hourly",
            3 => "/gpu/hourly", 
            4 => "/cpu/daily",
            _ => "/gpu/daily",
        }.to_string();
        
        tokio::spawn(async move {
            let response = app
                .oneshot(Request::builder().uri(&endpoint).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            response
        })
    });
    
    let responses = futures::future::join_all(tasks).await;
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_secs(5), "Stress test took too long: {:?}", duration);
    
    for response in responses {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

#[tokio::test]
async fn test_database_connection_pool_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    
    let tasks = (0..20).map(|_| {
        let app = app.clone();
        tokio::spawn(async move {
            let response = app
                .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        })
    });
    
    futures::future::join_all(tasks).await;
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_secs(1), "Database connection pool test took too long: {:?}", duration);
}

#[tokio::test]
async fn test_json_serialization_performance() {
    let app = create_performance_test_app().await;
    
    let start = Instant::now();
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let serialization_time = start.elapsed();
    
    assert!(serialization_time < Duration::from_millis(100), "JSON serialization took too long: {:?}", serialization_time);
    
    let start = Instant::now();
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    let deserialization_time = start.elapsed();
    
    assert!(deserialization_time < Duration::from_millis(100), "JSON deserialization took too long: {:?}", deserialization_time);
    assert_eq!(cpu_data.len(), 1000);
}

#[tokio::test]
async fn test_endpoint_response_time_consistency() {
    let app = create_performance_test_app().await;
    
    let mut times = Vec::new();
    
    for _ in 0..10 {
        let start = Instant::now();
        let response = app
            .clone()
            .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let duration = start.elapsed();
        
        assert_eq!(response.status(), StatusCode::OK);
        times.push(duration);
    }
    
    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    let max_time = times.iter().max().unwrap();
    let min_time = times.iter().min().unwrap();
    
    assert!(avg_time < Duration::from_millis(100), "Average response time too high: {:?}", avg_time);
    assert!(max_time.as_millis() - min_time.as_millis() < 50, "Response time variance too high");
}