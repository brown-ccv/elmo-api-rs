use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::sqlite::SqlitePool;

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Utilization {
    pub time: String,
    pub allocated: i32,
    pub total: i32,
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
            sqlx::query_as::<_, Utilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Utilization>(query)
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
    // We use a Common Table Expression (CTE) to first format the timestamps into hourly buckets.
    // This ensures that all entries within the same hour are properly grouped together.
    // The CTE approach is cleaner than using strftime directly in the GROUP BY clause
    // and makes the query more maintainable.
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to hourly precision (YYYY-MM-DD HH:00:00)
            -- This ensures all entries within the same hour have the same timestamp
            SELECT 
                strftime('%Y-%m-%d %H:00:00', time) as time,
                allocated,
                total
            FROM cpu
            WHERE time BETWEEN ? AND ?
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        "#
    } else {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to hourly precision (YYYY-MM-DD HH:00:00)
            -- This ensures all entries within the same hour have the same timestamp
            SELECT 
                strftime('%Y-%m-%d %H:00:00', time) as time,
                allocated,
                total
            FROM cpu
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        LIMIT 100
        "#
    };

    let hourly_cpu_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, Utilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Utilization>(query)
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
        sqlx::query_as::<_, Utilization>(query)
            .bind(start)
            .bind(end)
            .fetch_all(&pool)
            .await
    } else {
        sqlx::query_as::<_, Utilization>(query)
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
    // We use a Common Table Expression (CTE) to first format the timestamps into hourly buckets.
    // This ensures that all entries within the same hour are properly grouped together.
    // The CTE approach is cleaner than using strftime directly in the GROUP BY clause
    // and makes the query more maintainable.
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to hourly precision (YYYY-MM-DD HH:00:00)
            -- This ensures all entries within the same hour have the same timestamp
            SELECT 
                strftime('%Y-%m-%d %H:00:00', time) as time,
                allocated,
                total
            FROM gpu
            WHERE time BETWEEN ? AND ?
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        "#
    } else {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to hourly precision (YYYY-MM-DD HH:00:00)
            -- This ensures all entries within the same hour have the same timestamp
            SELECT 
                strftime('%Y-%m-%d %H:00:00', time) as time,
                allocated,
                total
            FROM gpu
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        LIMIT 100
        "#
    };

    let hourly_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, Utilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Utilization>(query)
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
    // We use a Common Table Expression (CTE) to first format the timestamps into daily buckets.
    // This ensures that all entries within the same day are properly grouped together.
    // The CTE approach is cleaner than using strftime directly in the GROUP BY clause
    // and makes the query more maintainable.
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to daily precision (YYYY-MM-DD)
            -- This ensures all entries within the same day have the same timestamp
            SELECT 
                strftime('%Y-%m-%d', time) as time,
                allocated,
                total
            FROM cpu
            WHERE time BETWEEN ? AND ?
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        "#
    } else {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to daily precision (YYYY-MM-DD)
            -- This ensures all entries within the same day have the same timestamp
            SELECT 
                strftime('%Y-%m-%d', time) as time,
                allocated,
                total
            FROM cpu
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        LIMIT 100
        "#
    };

    let daily_cpu_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, Utilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Utilization>(query)
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
    // We use a Common Table Expression (CTE) to first format the timestamps into daily buckets.
    // This ensures that all entries within the same day are properly grouped together.
    // The CTE approach is cleaner than using strftime directly in the GROUP BY clause
    // and makes the query more maintainable.
    let query = if time_range.start_time.is_some() && time_range.end_time.is_some() {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to daily precision (YYYY-MM-DD)
            -- This ensures all entries within the same day have the same timestamp
            SELECT 
                strftime('%Y-%m-%d', time) as time,
                allocated,
                total
            FROM gpu
            WHERE time BETWEEN ? AND ?
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        "#
    } else {
        r#"
        WITH formatted_time AS (
            -- First, format all timestamps to daily precision (YYYY-MM-DD)
            -- This ensures all entries within the same day have the same timestamp
            SELECT 
                strftime('%Y-%m-%d', time) as time,
                allocated,
                total
            FROM gpu
        )
        SELECT 
            time,
            CAST(ROUND(AVG(allocated)) AS INTEGER) as allocated,
            CAST(ROUND(AVG(total)) AS INTEGER) as total
        FROM formatted_time
        GROUP BY time
        ORDER BY time
        LIMIT 100
        "#
    };

    let daily_utilization =
        if let (Some(start), Some(end)) = (time_range.start_time, time_range.end_time) {
            sqlx::query_as::<_, Utilization>(query)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
        } else {
            sqlx::query_as::<_, Utilization>(query)
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
    /// so we can parse it back into our expected data types (e.g., Utilization).
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
        let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

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
        let cpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

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
        let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(hourly_data.len(), 1); // All data is in one hour
        assert_eq!(hourly_data[0].allocated, 83); // ROUND((75 + 80 + 85 + 90) / 4)
        assert_eq!(hourly_data[0].total, 100);
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
        let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(daily_data.len(), 1); // All data is in one day
        assert_eq!(daily_data[0].allocated, 83); // ROUND((75 + 80 + 85 + 90) / 4)
        assert_eq!(daily_data[0].total, 100);
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
        let gpu_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

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
        let hourly_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(hourly_data.len(), 1); // All data is in one hour
        assert_eq!(hourly_data[0].allocated, 68); // ROUND((60 + 65 + 70 + 75) / 4)
        assert_eq!(hourly_data[0].total, 100);
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
        let daily_data: Vec<Utilization> = serde_json::from_slice(&body).unwrap();

        assert_eq!(daily_data.len(), 1); // All data is in one day
        assert_eq!(daily_data[0].allocated, 68); // ROUND((60 + 65 + 70 + 75) / 4)
        assert_eq!(daily_data[0].total, 100);
    }
}
