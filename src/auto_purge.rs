use actix_web::web;
use chrono::{Utc, Duration};
use std::time::Duration as StdDuration;
use crate::AppState;
use log::info;
use sqlx;

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