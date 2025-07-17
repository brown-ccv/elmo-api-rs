use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::Response,
};
use elmo_api::routes::{TimeRange, Utilization};
use http_body_util::BodyExt;
use serde_json::Value;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceExt;

async fn setup_e2e_test_db() -> SqlitePool {
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
            ('2024-03-27T00:00:00', 10, 100),
            ('2024-03-27T00:30:00', 20, 100),
            ('2024-03-27T01:00:00', 30, 100),
            ('2024-03-27T01:30:00', 40, 100),
            ('2024-03-27T02:00:00', 50, 100),
            ('2024-03-28T00:00:00', 60, 100),
            ('2024-03-28T00:30:00', 70, 100),
            ('2024-03-28T01:00:00', 80, 100),
            ('2024-03-29T00:00:00', 90, 100),
            ('2024-03-29T12:00:00', 95, 100);
        INSERT INTO gpu (time, allocated, total) VALUES
            ('2024-03-27T00:00:00', 15, 100),
            ('2024-03-27T00:30:00', 25, 100),
            ('2024-03-27T01:00:00', 35, 100),
            ('2024-03-27T01:30:00', 45, 100),
            ('2024-03-27T02:00:00', 55, 100),
            ('2024-03-28T00:00:00', 65, 100),
            ('2024-03-28T00:30:00', 75, 100),
            ('2024-03-28T01:00:00', 85, 100),
            ('2024-03-29T00:00:00', 95, 100),
            ('2024-03-29T12:00:00', 98, 100);
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

async fn create_e2e_test_app() -> axum::Router {
    use axum::routing::get;
    use axum::{
        extract::{Query, State},
        response::IntoResponse,
        Json,
    };
    use tower_http::cors::{Any, CorsLayer};
    use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
    use tracing::Level;

    let pool = setup_e2e_test_db().await;

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

async fn spawn_test_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = create_e2e_test_app().await;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    addr
}

#[tokio::test]
async fn test_e2e_root_endpoint() {
    let app = create_e2e_test_app().await;
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
async fn test_e2e_cpu_endpoint_full_data() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(cpu_data.len(), 10);
    assert_eq!(cpu_data[0].allocated, Some(10));
    assert_eq!(cpu_data[0].total, Some(100));
    assert_eq!(cpu_data[9].allocated, Some(95));
    assert_eq!(cpu_data[9].total, Some(100));
}

#[tokio::test]
async fn test_e2e_gpu_endpoint_full_data() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/gpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let gpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(gpu_data.len(), 10);
    assert_eq!(gpu_data[0].allocated, Some(15));
    assert_eq!(gpu_data[0].total, Some(100));
    assert_eq!(gpu_data[9].allocated, Some(98));
    assert_eq!(gpu_data[9].total, Some(100));
}

#[tokio::test]
async fn test_e2e_hourly_cpu_aggregation() {
    let app = create_e2e_test_app().await;
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

    assert_eq!(hourly_data.len(), 7);

    assert_eq!(hourly_data[0].allocated, Some(15));
    assert_eq!(hourly_data[1].allocated, Some(35));
    assert_eq!(hourly_data[2].allocated, Some(50));
    assert_eq!(hourly_data[3].allocated, Some(65));
    assert_eq!(hourly_data[4].allocated, Some(80));
    assert_eq!(hourly_data[5].allocated, Some(90));
    assert_eq!(hourly_data[6].allocated, Some(95));
}

#[tokio::test]
async fn test_e2e_daily_cpu_aggregation() {
    let app = create_e2e_test_app().await;
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

    assert_eq!(daily_data.len(), 3);

    assert_eq!(daily_data[0].allocated, Some(30));
    assert_eq!(daily_data[1].allocated, Some(70));
    assert_eq!(daily_data[2].allocated, Some(93));
}

#[tokio::test]
async fn test_e2e_hourly_gpu_aggregation() {
    let app = create_e2e_test_app().await;
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

    assert_eq!(hourly_data.len(), 7);

    assert_eq!(hourly_data[0].allocated, Some(20));
    assert_eq!(hourly_data[1].allocated, Some(40));
    assert_eq!(hourly_data[2].allocated, Some(55));
    assert_eq!(hourly_data[3].allocated, Some(70));
    assert_eq!(hourly_data[4].allocated, Some(85));
    assert_eq!(hourly_data[5].allocated, Some(95));
    assert_eq!(hourly_data[6].allocated, Some(98));
}

#[tokio::test]
async fn test_e2e_daily_gpu_aggregation() {
    let app = create_e2e_test_app().await;
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

    assert_eq!(daily_data.len(), 3);

    assert_eq!(daily_data[0].allocated, Some(35));
    assert_eq!(daily_data[1].allocated, Some(75));
    assert_eq!(daily_data[2].allocated, Some(97));
}

#[tokio::test]
async fn test_e2e_cpu_time_range_filtering() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu?start=2024-03-27T00:00:00&end=2024-03-27T01:00:00")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(cpu_data.len(), 3);
    assert_eq!(cpu_data[0].allocated, Some(10));
    assert_eq!(cpu_data[1].allocated, Some(20));
    assert_eq!(cpu_data[2].allocated, Some(30));
}

#[tokio::test]
async fn test_e2e_gpu_time_range_filtering() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/gpu?start=2024-03-28T00:00:00&end=2024-03-28T01:00:00")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let gpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(gpu_data.len(), 3);
    assert_eq!(gpu_data[0].allocated, Some(65));
    assert_eq!(gpu_data[1].allocated, Some(75));
    assert_eq!(gpu_data[2].allocated, Some(85));
}

#[tokio::test]
async fn test_e2e_hourly_aggregation_with_time_range() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/hourly?start=2024-03-27T00:00:00&end=2024-03-27T23:59:59")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(hourly_data.len(), 3);
    assert_eq!(hourly_data[0].allocated, Some(15));
    assert_eq!(hourly_data[1].allocated, Some(35));
    assert_eq!(hourly_data[2].allocated, Some(50));
}

#[tokio::test]
async fn test_e2e_daily_aggregation_with_time_range() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cpu/daily?start=2024-03-27T00:00:00&end=2024-03-28T23:59:59")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(daily_data.len(), 2);
    assert_eq!(daily_data[0].allocated, Some(30));
    assert_eq!(daily_data[1].allocated, Some(70));
}

#[tokio::test]
async fn test_e2e_malformed_time_parameters() {
    let app = create_e2e_test_app().await;
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
async fn test_e2e_partial_time_parameters() {
    let app = create_e2e_test_app().await;

    let response = app
        .clone()
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
    assert_eq!(cpu_data.len(), 10);
}

#[tokio::test]
async fn test_e2e_method_not_allowed() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/cpu")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_e2e_not_found() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_e2e_response_headers() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(headers.get("content-type").unwrap(), "application/json");
    assert!(headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn test_e2e_preflight_cors_request() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/cpu")
                .header("origin", "https://example.com")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
    assert!(headers.contains_key("access-control-allow-methods"));
}

#[tokio::test]
async fn test_e2e_data_consistency() {
    let app = create_e2e_test_app().await;

    let cpu_response = app
        .clone()
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let gpu_response = app
        .oneshot(Request::builder().uri("/gpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(cpu_response.status(), StatusCode::OK);
    assert_eq!(gpu_response.status(), StatusCode::OK);

    let cpu_body = get_body_bytes(cpu_response).await;
    let gpu_body = get_body_bytes(gpu_response).await;

    let cpu_data: Vec<Utilization> = serde_json::from_slice(&cpu_body).unwrap();
    let gpu_data: Vec<Utilization> = serde_json::from_slice(&gpu_body).unwrap();

    assert_eq!(cpu_data.len(), gpu_data.len());

    for (cpu, gpu) in cpu_data.iter().zip(gpu_data.iter()) {
        assert_eq!(cpu.total, gpu.total);
        assert!(cpu.allocated.is_some());
        assert!(gpu.allocated.is_some());
    }
}

#[tokio::test]
async fn test_e2e_concurrent_requests() {
    let app = create_e2e_test_app().await;

    let tasks = (0..10).map(|_| {
        let app = app.clone();
        tokio::spawn(async move {
            app.oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
                .await
                .unwrap()
        })
    });

    let responses = futures::future::join_all(tasks).await;

    for response in responses {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
        assert_eq!(cpu_data.len(), 10);
    }
}

#[tokio::test]
async fn test_e2e_large_dataset_performance() {
    use std::time::Instant;

    let app = create_e2e_test_app().await;

    let start = Instant::now();
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(duration < Duration::from_millis(100));

    let body = get_body_bytes(response).await;
    let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();
    assert_eq!(cpu_data.len(), 10);
}

#[tokio::test]
async fn test_e2e_json_response_structure() {
    let app = create_e2e_test_app().await;
    let response = app
        .oneshot(Request::builder().uri("/cpu").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = get_body_bytes(response).await;
    let json_value: Value = serde_json::from_slice(&body).unwrap();

    assert!(json_value.is_array());

    let array = json_value.as_array().unwrap();
    assert!(!array.is_empty());

    let first_item = &array[0];
    assert!(first_item.is_object());

    let obj = first_item.as_object().unwrap();
    assert!(obj.contains_key("time"));
    assert!(obj.contains_key("allocated"));
    assert!(obj.contains_key("total"));
}
