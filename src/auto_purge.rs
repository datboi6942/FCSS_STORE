use actix_web::web;
use chrono::{Utc, Duration};
use std::time::Duration as StdDuration;
use crate::AppState;
use log::{info, error};
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
    info!("Purging old records...");
    
    // Purge transactions older than 90 days
    let result = sqlx::query!(
        "DELETE FROM transactions WHERE created_at < datetime('now', '-90 days')"
    )
    .execute(pool)
    .await?;
    
    let count = result.rows_affected();
    
    // Also purge completed orders older than 60 days
    let order_result = sqlx::query!(
        "DELETE FROM orders WHERE status = 'completed' AND created_at < datetime('now', '-60 days')"
    )
    .execute(pool)
    .await?;
    
    let total_count = count + order_result.rows_affected();
    
    Ok(total_count)
}

// Function to purge old transactions
async fn purge_old_transactions(pool: &SqlitePool) {
    info!("Purging old transactions...");
    
    // Delete transactions older than 30 days
    match sqlx::query!(
        "DELETE FROM transactions WHERE created_at < datetime('now', '-30 days')"
    )
    .execute(pool)
    .await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                info!("Purged {} old transactions", result.rows_affected());
            } else {
                info!("No old transactions to purge");
            }
        },
        Err(e) => {
            error!("Failed to purge old transactions: {}", e);
        }
    }
}

// Function to purge old chat messages
async fn purge_old_chat_messages(app_state: web::Data<AppState>) {
    info!("Purging old chat messages...");
    
    // Access chat history through mutex
    let mut chat_history = app_state.chat_history.lock().unwrap();
    
    // Calculate timestamp for 7 days ago
    let week_ago = chrono::Utc::now() - chrono::Duration::days(7);
    
    // Count messages before deletion
    let before_count = chat_history.len();
    
    // Remove messages older than 7 days
    chat_history.retain(|msg| msg.timestamp > week_ago);
    
    // Count removed messages
    let removed_count = before_count - chat_history.len();
    
    if removed_count > 0 {
        info!("Purged {} old chat messages", removed_count);
    } else {
        info!("No old chat messages to purge");
    }
}

// Main auto-purge function
pub async fn start_auto_purge(app_state: web::Data<AppState>) {
    info!("Starting auto-purge service...");
    
    loop {
        // Purge old transactions
        purge_old_transactions(&app_state.db).await;
        
        // Purge old chat messages
        purge_old_chat_messages(app_state.clone()).await;
        
        // Sleep for 24 hours
        tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
    }
}