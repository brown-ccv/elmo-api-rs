use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct CpuUtilization {
    pub time: String,
    pub allocated: i32,
    pub total: i32,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct GpuUtilization {
    pub time: String,
    pub allocated: i32,
    pub total: i32,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct HourlyCpuUtilization {
    pub hour: String,
    pub avg_allocated: f64,
    pub avg_total: f64,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct HourlyGpuUtilization {
    pub hour: String,
    pub avg_allocated: f64,
    pub avg_total: f64,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct DailyCpuUtilization {
    pub day: String,
    pub avg_allocated: f64,
    pub avg_total: f64,
}

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct DailyGpuUtilization {
    pub day: String,
    pub avg_allocated: f64,
    pub avg_total: f64,
}

#[derive(Debug, Deserialize)]
pub struct TimeRange {
    start_time: Option<String>,
    end_time: Option<String>,
}

pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn get_cpu_utilization(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        SELECT 
            time, 
            allocated, 
            total 
        FROM  
            cpu 
        WHERE time BETWEEN ? AND ?
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
        LIMIT 100
        "#
    };

    let cpu_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, CpuUtilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, CpuUtilization>(query)
                .fetch_all(&pool)
                .await
        };

    let cpu_utilization = cpu_utilization
        .map_err(|e| {
            tracing::error!("Error: failed to get cpu utilization: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Error: Could not get CPU utilization info");

    Json(cpu_utilization)
}

pub async fn get_hourly_cpu_utilization(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        SELECT 
            strftime('%Y-%m-%d %H:00:00', time) as hour,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            cpu 
        WHERE time BETWEEN ? AND ?
        GROUP BY hour
        ORDER BY hour
        "#
    } else {
        r#"
        SELECT 
            strftime('%Y-%m-%d %H:00:00', time) as hour,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            cpu 
        GROUP BY hour
        ORDER BY hour
        LIMIT 100
        "#
    };

    let hourly_cpu_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, HourlyCpuUtilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, HourlyCpuUtilization>(query)
                .fetch_all(&pool)
                .await
        };

    let hourly_cpu_utilization = hourly_cpu_utilization
        .map_err(|e| {
            tracing::error!("Error: failed to get hourly cpu utilization: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Error: Could not get hourly CPU utilization info");

    Json(hourly_cpu_utilization)
}

pub async fn get_gpu_utilization(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        SELECT 
            time, 
            allocated, 
            total 
        FROM  
            gpu 
        WHERE time BETWEEN ? AND ?
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
        LIMIT 100
        "#
    };

    let utilization = if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time)
    {
        sqlx::query_as::<_, GpuUtilization>(query)
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as::<_, GpuUtilization>(query)
            .fetch_all(&pool)
            .await
    };

    let utilization = utilization
        .map_err(|e| {
            tracing::error!("Error: failed to get gpu utilization: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Error: Could not get GPU utilization info");

    Json(utilization)
}

pub async fn get_hourly_gpu_utilization(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        SELECT 
            strftime('%Y-%m-%d %H:00:00', time) as hour,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            gpu 
        WHERE time BETWEEN ? AND ?
        GROUP BY hour
        ORDER BY hour
        "#
    } else {
        r#"
        SELECT 
            strftime('%Y-%m-%d %H:00:00', time) as hour,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            gpu 
        GROUP BY hour
        ORDER BY hour
        LIMIT 100
        "#
    };

    let hourly_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, HourlyGpuUtilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, HourlyGpuUtilization>(query)
                .fetch_all(&pool)
                .await
        };

    let hourly_utilization = hourly_utilization
        .map_err(|e| {
            tracing::error!("Error: failed to get hourly gpu utilization: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Error: Could not get hourly GPU utilization info");

    Json(hourly_utilization)
}

pub async fn get_daily_cpu_utilization(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        SELECT 
            strftime('%Y-%m-%d', time) as day,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            cpu 
        WHERE time BETWEEN ? AND ?
        GROUP BY day
        ORDER BY day
        "#
    } else {
        r#"
        SELECT 
            strftime('%Y-%m-%d', time) as day,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            cpu 
        GROUP BY day
        ORDER BY day
        LIMIT 100
        "#
    };

    let daily_cpu_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, DailyCpuUtilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, DailyCpuUtilization>(query)
                .fetch_all(&pool)
                .await
        };

    let daily_cpu_utilization = daily_cpu_utilization
        .map_err(|e| {
            tracing::error!("Error: failed to get daily cpu utilization: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Error: Could not get daily CPU utilization info");

    Json(daily_cpu_utilization)
}

pub async fn get_daily_gpu_utilization(
    State(pool): State<SqlitePool>,
    Query(time_range): Query<TimeRange>,
) -> impl IntoResponse {
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        SELECT 
            strftime('%Y-%m-%d', time) as day,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            gpu 
        WHERE time BETWEEN ? AND ?
        GROUP BY day
        ORDER BY day
        "#
    } else {
        r#"
        SELECT 
            strftime('%Y-%m-%d', time) as day,
            AVG(allocated) as avg_allocated,
            AVG(total) as avg_total
        FROM  
            gpu 
        GROUP BY day
        ORDER BY day
        LIMIT 100
        "#
    };

    let daily_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, DailyGpuUtilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, DailyGpuUtilization>(query)
                .fetch_all(&pool)
                .await
        };

    let daily_utilization = daily_utilization
        .map_err(|e| {
            tracing::error!("Error: failed to get daily gpu utilization: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
        .expect("Error: Could not get daily GPU utilization info");

    Json(daily_utilization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::Response;
    use http_body_util::BodyExt;
    use sqlx::sqlite::SqlitePool;

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

    /// Converts an Axum Response into a Vec<u8> containing the response body bytes.
    ///
    /// This function is used in tests to extract the response body from an Axum Response
    /// so we can parse it back into our expected data types (e.g., CpuUtilization, GpuUtilization).
    async fn get_body_bytes(response: Response) -> Vec<u8> {
        let body = response.into_body();
        let bytes = body.collect().await.unwrap().to_bytes();
        bytes.to_vec()
    }

    #[tokio::test]
    async fn test_get_cpu_utilization_without_time_range() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: None,
            end_time: None,
        };

        let result = get_cpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let cpu_data: Vec<CpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(cpu_data.len(), 4);
        assert_eq!(cpu_data[0].allocated, 75);
        assert_eq!(cpu_data[0].total, 100);
    }

    #[tokio::test]
    async fn test_get_cpu_utilization_with_time_range() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: Some("2024-03-27T00:00:00".to_string()),
            end_time: Some("2024-03-27T00:30:00".to_string()),
        };

        let result = get_cpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let cpu_data: Vec<CpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(cpu_data.len(), 3); // Only first 3 entries within time range
        assert_eq!(cpu_data[0].allocated, 75);
        assert_eq!(cpu_data[0].total, 100);
    }

    #[tokio::test]
    async fn test_get_hourly_cpu_utilization() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: None,
            end_time: None,
        };

        let result = get_hourly_cpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let hourly_data: Vec<HourlyCpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(hourly_data.len(), 1); // All data is in one hour
        assert_eq!(hourly_data[0].avg_allocated, 82.5); // (75 + 80 + 85 + 90) / 4
        assert_eq!(hourly_data[0].avg_total, 100.0);
    }

    #[tokio::test]
    async fn test_get_daily_cpu_utilization() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: None,
            end_time: None,
        };

        let result = get_daily_cpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let daily_data: Vec<DailyCpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(daily_data.len(), 1); // All data is in one day
        assert_eq!(daily_data[0].avg_allocated, 82.5); // (75 + 80 + 85 + 90) / 4
        assert_eq!(daily_data[0].avg_total, 100.0);
    }

    #[tokio::test]
    async fn test_get_gpu_utilization_without_time_range() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: None,
            end_time: None,
        };

        let result = get_gpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let gpu_data: Vec<GpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(gpu_data.len(), 4);
        assert_eq!(gpu_data[0].allocated, 60);
        assert_eq!(gpu_data[0].total, 100);
    }

    #[tokio::test]
    async fn test_get_hourly_gpu_utilization() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: None,
            end_time: None,
        };

        let result = get_hourly_gpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let hourly_data: Vec<HourlyGpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(hourly_data.len(), 1); // All data is in one hour
        assert_eq!(hourly_data[0].avg_allocated, 67.5); // (60 + 65 + 70 + 75) / 4
        assert_eq!(hourly_data[0].avg_total, 100.0);
    }

    #[tokio::test]
    async fn test_get_daily_gpu_utilization() {
        let pool = setup_test_db().await;
        let time_range = TimeRange {
            start_time: None,
            end_time: None,
        };

        let result = get_daily_gpu_utilization(State(pool), Query(time_range)).await;
        let response = result.into_response();

        assert_eq!(response.status(), StatusCode::OK);

        let body = get_body_bytes(response).await;
        let daily_data: Vec<DailyGpuUtilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(daily_data.len(), 1); // All data is in one day
        assert_eq!(daily_data[0].avg_allocated, 67.5); // (60 + 65 + 70 + 75) / 4
        assert_eq!(daily_data[0].avg_total, 100.0);
    }
}
