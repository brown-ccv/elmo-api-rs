use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use elmo_api::{create_app, routes::Utilization};
use http_body_util::BodyExt;
use sqlx::sqlite::SqlitePool;
use tower::ServiceExt;

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
    let pool = setup_test_db().await;
    create_app(pool).await
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
    assert_eq!(cpu_data[0].allocated, 75);
    assert_eq!(cpu_data[0].total, 100);
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
    assert_eq!(hourly_data[0].allocated, 83); // ROUND((75 + 80 + 85 + 90) / 4)
    assert_eq!(hourly_data[0].total, 100);
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
    assert_eq!(daily_data[0].allocated, 83); // ROUND((75 + 80 + 85 + 90) / 4)
    assert_eq!(daily_data[0].total, 100);
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
    assert_eq!(gpu_data[0].allocated, 60);
    assert_eq!(gpu_data[0].total, 100);
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
    assert_eq!(hourly_data[0].allocated, 68); // ROUND((60 + 65 + 70 + 75) / 4)
    assert_eq!(hourly_data[0].total, 100);
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
    assert_eq!(daily_data[0].allocated, 68); // ROUND((60 + 65 + 70 + 75) / 4)
    assert_eq!(daily_data[0].total, 100);
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
