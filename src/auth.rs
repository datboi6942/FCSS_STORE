// src/auth.rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct UserRegister {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub token: String,
}

pub async fn register(
    user_data: web::Json<UserRegister>,
    state: web::Data<AppState>,
) -> impl Responder {
    let user = user_data.into_inner();
    
    // Basic validation
    if user.username.is_empty() || user.password.is_empty() {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"error": "Username and password are required"})
        );
    }
    
    // Check if username already exists
    let existing_user = sqlx::query!(
        "SELECT username FROM users WHERE username = ?",
        user.username
    )
    .fetch_optional(&state.db)
    .await;
    
    match existing_user {
        Ok(Some(_)) => {
            return HttpResponse::Conflict().json(
                serde_json::json!({"error": "Username already exists"})
            );
        }
        Ok(None) => { /* Username is available */ }
        Err(e) => {
            log::error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to check username"})
            );
        }
    }
    
    // Hash password
    let hashed_password = match hash(user.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to process password"})
            );
        }
    };
    
    // Create new user
    let user_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    // Insert into database
    let result = sqlx::query!(
        "INSERT INTO users (id, username, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?)",
        user_id,
        user.username,
        hashed_password,
        "user", // Default role
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            HttpResponse::Created().json(
                serde_json::json!({"message": "User registered successfully", "user_id": user_id})
            )
        }
        Err(e) => {
            log::error!("Failed to register user: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to register user"})
            )
        }
    }
}

pub async fn login(
    user_data: web::Json<UserLogin>,
    state: web::Data<AppState>,
) -> impl Responder {
    let user = user_data.into_inner();
    
    // Find user by username using regular query
    let db_user = sqlx::query!(
        "SELECT id, username, password_hash, role, created_at FROM users WHERE username = ?",
        user.username
    )
    .fetch_optional(&state.db)
    .await;
    
    match db_user {
        Ok(Some(row)) => {
            // Manually create User from row
            let db_user = User {
                id: row.id,
                username: row.username,
                password_hash: row.password_hash,
                role: row.role,
                created_at: Some(Utc::now()),
            };
            
            // Verify password
            match verify(&user.password, &db_user.password_hash) {
                Ok(valid) if valid => {
                    // Generate a fake token (in real app, use JWT)
                    let token = format!("token-{}", Uuid::new_v4());
                    
                    HttpResponse::Ok().json(UserResponse {
                        id: db_user.id,
                        username: db_user.username,
                        role: db_user.role,
                        token,
                    })
                }
                _ => {
                    HttpResponse::Unauthorized().json(
                        serde_json::json!({"error": "Invalid credentials"})
                    )
                }
            }
        }
        Ok(None) => {
            // User doesn't exist, but return same error for security
            HttpResponse::Unauthorized().json(
                serde_json::json!({"error": "Invalid credentials"})
            )
        }
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "An error occurred during login"})
            )
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
    );
}
