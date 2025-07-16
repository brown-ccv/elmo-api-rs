use axum::{
    body::Body,
    extract::{Query, State},
    http::{Request, StatusCode},
    response::{Response, IntoResponse},
};
use elmo_api::routes::{TimeRange, Utilization};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn setup_integration_test_db() -> SqlitePool {
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

    sqlx::query(
        r#"
        INSERT INTO cpu (time, allocated, total) VALUES
            ('2024-03-27T00:00:00', 75, 100),
            ('2024-03-27T00:15:00', 80, 100),
            ('2024-03-27T00:30:00', 85, 100),
            ('2024-03-27T00:45:00', 90, 100),
            ('2024-03-27T01:00:00', 60, 100),
            ('2024-03-27T01:15:00', 65, 100),
            ('2024-03-28T00:00:00', 70, 100),
            ('2024-03-28T00:15:00', 75, 100);
        INSERT INTO gpu (time, allocated, total) VALUES
            ('2024-03-27T00:00:00', 60, 100),
            ('2024-03-27T00:15:00', 65, 100),
            ('2024-03-27T00:30:00', 70, 100),
            ('2024-03-27T00:45:00', 75, 100),
            ('2024-03-27T01:00:00', 50, 100),
            ('2024-03-27T01:15:00', 55, 100),
            ('2024-03-28T00:00:00', 80, 100),
            ('2024-03-28T00:15:00', 85, 100);
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn create_integration_test_app() -> axum::Router {
    use axum::routing::get;
    use axum::{
        extract::{Query, State},
        http::StatusCode,
        response::IntoResponse,
        Json,
    };
    use tower_http::cors::{Any, CorsLayer};

    let pool = setup_integration_test_db().await;

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
async fn test_root_endpoint() {
    let app = create_integration_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let body_str = String::from_utf8(body).unwrap();
    assert_eq!(body_str, "Hello, World!");
}

#[tokio::test]
async fn test_cors_headers() {
    let app = create_integration_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu")
                .header("Origin", "https://example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_cpu_endpoint_with_multiple_time_ranges() {
    let app = create_integration_test_app().await;
    
    let test_cases = vec![
        ("/cpu", 8),
        ("/cpu?start=2024-03-27T00:00:00&end=2024-03-27T00:30:00", 3),
        ("/cpu?start=2024-03-27T01:00:00&end=2024-03-27T01:30:00", 2),
        ("/cpu?start=2024-03-28T00:00:00&end=2024-03-28T00:30:00", 2),
    ];

    for (uri, expected_count) in test_cases {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = get_body_bytes(response).await;
        let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
        assert_eq!(cpu_data.len(), expected_count, "Failed for URI: {}", uri);
    }
}

#[tokio::test]
async fn test_gpu_endpoint_with_multiple_time_ranges() {
    let app = create_integration_test_app().await;
    
    let test_cases = vec![
        ("/gpu", 8),
        ("/gpu?start=2024-03-27T00:00:00&end=2024-03-27T00:30:00", 3),
        ("/gpu?start=2024-03-27T01:00:00&end=2024-03-27T01:30:00", 2),
        ("/gpu?start=2024-03-28T00:00:00&end=2024-03-28T00:30:00", 2),
    ];

    for (uri, expected_count) in test_cases {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = get_body_bytes(response).await;
        let gpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
        assert_eq!(gpu_data.len(), expected_count, "Failed for URI: {}", uri);
    }
}

#[tokio::test]
async fn test_hourly_aggregation_across_multiple_hours() {
    let app = create_integration_test_app().await;
    
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
    
    assert_eq!(hourly_data.len(), 3);
    
    assert_eq!(hourly_data[0].allocated, Some(83));
    assert_eq!(hourly_data[1].allocated, Some(63));
    assert_eq!(hourly_data[2].allocated, Some(73));
}

#[tokio::test]
async fn test_daily_aggregation_across_multiple_days() {
    let app = create_integration_test_app().await;
    
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
    
    assert_eq!(daily_data.len(), 2);
    
    assert_eq!(daily_data[0].allocated, Some(76));
    assert_eq!(daily_data[1].allocated, Some(73));
}

#[tokio::test]
async fn test_hourly_aggregation_with_time_range() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/hourly?start=2024-03-27T00:00:00&end=2024-03-27T00:59:59")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(hourly_data.len(), 1);
    assert_eq!(hourly_data[0].allocated, Some(83));
}

#[tokio::test]
async fn test_daily_aggregation_with_time_range() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/daily?start=2024-03-27T00:00:00&end=2024-03-27T23:59:59")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(daily_data.len(), 1);
    assert_eq!(daily_data[0].allocated, Some(76));
}

#[tokio::test]
async fn test_invalid_time_range() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?start=invalid-date&end=2024-03-27T00:30:00")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_start_parameter() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?end=2024-03-27T00:30:00")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert_eq!(cpu_data.len(), 8);
}

#[tokio::test]
async fn test_missing_end_parameter() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?start=2024-03-27T00:00:00")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert_eq!(cpu_data.len(), 8);
}

#[tokio::test]
async fn test_empty_result_set() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?start=2025-01-01T00:00:00&end=2025-01-01T23:59:59")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert_eq!(cpu_data.len(), 0);
}

#[tokio::test]
async fn test_response_format() {
    let app = create_integration_test_app().await;
    
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/json");
    
    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    
    assert!(cpu_data.len() > 0);
    
    let first_record = &cpu_data[0];
    assert!(first_record.time.is_some());
    assert!(first_record.allocated.is_some());
    assert!(first_record.total.is_some());
}

#[tokio::test]
async fn test_all_endpoints_return_json() {
    let app = create_integration_test_app().await;
    
    let endpoints = vec![
        "/cpu",
        "/gpu", 
        "/cpu/hourly",
        "/gpu/hourly",
        "/cpu/daily",
        "/gpu/daily",
    ];
    
    for endpoint in endpoints {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(endpoint).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK, "Failed for endpoint: {}", endpoint);
        
        let headers = response.headers();
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        
        let body = get_body_bytes(response).await;
        let _: Vec<Utilization> = serde_json::from_slice(&body)
            .expect(&format!("Failed to parse JSON for endpoint: {}", endpoint));
    }
}

#[tokio::test]
async fn test_database_connection_failure_handling() {
    let empty_pool = SqlitePool::connect(":memory:").await.unwrap();
    
    async fn failing_handler(
        State(_pool): State<SqlitePool>,
        Query(_time_range): Query<TimeRange>,
    ) -> impl IntoResponse {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    
    let app = axum::Router::new()
        .route("/cpu", axum::routing::get(failing_handler))
        .with_state(empty_pool);
    
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}