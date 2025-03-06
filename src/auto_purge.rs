use actix_web::web;
use chrono::{Utc, Duration};
use std::time::Duration as StdDuration;
use crate::AppState;
use log::info;
use sqlx;
use sqlx::SqlitePool;
use tokio::task;
use tokio::time;

/// Public function that continuously purges orders older than 30 days.
pub async fn auto_purge_old_orders(app_state: web::Data<AppState>) {
    info!("Auto-purging old orders...");
    
    let now = Utc::now();
    let thirty_days_ago = now - Duration::days(30);
    
    // Use the database instead of in-memory orders
    let result = sqlx::query!(
        "DELETE FROM orders WHERE created_at < ?",
        thirty_days_ago
    )
    .execute(&app_state.db)
    .await;
    
    match result {
        Ok(deleted) => info!("Purged {} old orders", deleted.rows_affected()),
        Err(e) => log::error!("Failed to purge old orders: {}", e)
    }
    
    info!("Auto-purge completed");
}

pub fn schedule_auto_purge(app_state: web::Data<AppState>) {
    let app_state = app_state.clone();
    
    tokio::spawn(async move {
        loop {
            auto_purge_old_orders(app_state.clone()).await;
            
            // Sleep for 24 hours
            tokio::time::sleep(StdDuration::from_secs(24 * 60 * 60)).await;
        }
    });
}

pub fn spawn_purge_task(pool: SqlitePool) -> Result<task::JoinHandle<()>, ()> {
    let handle = task::spawn(async move {
        let mut interval = time::interval(StdDuration::from_secs(3600)); // Run every hour
        
        loop {
            interval.tick().await;
            
            match purge_old_records(&pool).await {
                Ok(count) => {
                    if count > 0 {
                        log::info!("Auto-purged {} old records", count);
                    }
                },
                Err(e) => {
                    log::error!("Error during auto-purge: {}", e);
                }
            }
        }
    });
    
    Ok(handle)
}

async fn purge_old_records(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    // Get timestamp for records older than 30 days
    let cutoff_date = Utc::now() - chrono::Duration::days(30);
    let timestamp = cutoff_date.timestamp();
    
    // Delete old records (adjust table and column names as needed)
    // This is a placeholder - modify for your actual schema
    let result = sqlx::query!(
        r#"
        DELETE FROM orders
        WHERE created_at < ?
        "#,
        timestamp
    )
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected())
}