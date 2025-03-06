// src/auth.rs
use actix_web::{web, HttpResponse, Responder, HttpRequest, http::header};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;
use crate::AppState;
use crate::session;
use sqlx;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2
};
use log::{info, error, warn};
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};

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
pub struct UserRegistration {
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

// Secret key for JWT tokens - in production, use environment variables
const JWT_SECRET: &[u8] = b"secure_jwt_secret_key";
const TOKEN_EXPIRY_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub username: String,   // Username
    pub role: String,       // User role
    pub exp: i64,           // Expiration time (UTC timestamp)
    pub iat: i64,           // Issued at (UTC timestamp)
}

pub async fn register(
    user_data: web::Json<UserRegistration>,
    state: web::Data<AppState>,
) -> impl Responder {
    let user = user_data.into_inner();
    
    // Check if username already exists
    let existing_user = sqlx::query!(
        "SELECT id FROM users WHERE username = ?",
        user.username
    )
    .fetch_optional(&state.db)
    .await;
    
    match existing_user {
        Ok(Some(_)) => {
            warn!("Registration failed: Username {} already exists", user.username);
            return HttpResponse::Conflict().json(
                serde_json::json!({"error": "Username already exists"})
            );
        }
        Ok(None) => {
            // Username is available, create user
            info!("Username {} is available, creating new user", user.username);
        }
        Err(e) => {
            error!("Database error checking username: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Internal server error"})
            );
        }
    }
    
    // Hash password
    let password_hash = match hash(&user.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to process password"})
            );
        }
    };
    
    // Create new user
    let user_id = format!("usr-{}", Uuid::new_v4().simple());
    let now = Utc::now();
    
    let result = sqlx::query!(
        "INSERT INTO users (id, username, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?)",
        user_id,
        user.username,
        password_hash,
        "user", // Default role
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            info!("User {} successfully registered with ID: {}", user.username, user_id);
            
            // Generate JWT token
            let claims = Claims {
                sub: user_id.clone(),
                username: user.username.clone(),
                role: "user".to_string(),
                exp: (Utc::now() + Duration::hours(TOKEN_EXPIRY_HOURS)).timestamp(),
                iat: Utc::now().timestamp(),
            };
            
            match encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET)) {
                Ok(token) => {
                    info!("JWT token generated for user: {}", user.username);
                    HttpResponse::Created().json(
                        serde_json::json!({
                            "user_id": user_id,
                            "username": user.username,
                            "token": token,
                            "role": "user"
                        })
                    )
                }
                Err(e) => {
                    error!("Failed to generate token: {}", e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Failed to generate authentication token"})
                    )
                }
            }
        }
        Err(e) => {
            error!("Failed to create user: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to create user"})
            )
        }
    }
}

pub async fn login(
    user: web::Json<UserLogin>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user = user.into_inner();
    
    match sqlx::query!(
        "SELECT id, username, password_hash, role, created_at FROM users WHERE username = ?",
        user.username
    )
    .fetch_optional(&data.db)
    .await {
        Ok(Some(db_user)) => {
            // Check password
            let parsed_hash = match PasswordHash::new(&db_user.password_hash) {
                Ok(hash) => hash,
                Err(e) => {
                    log::error!("Failed to parse password hash: {}", e);
                    return HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Authentication error"})
                    );
                }
            };
            
            let argon2 = Argon2::default();
            let is_valid = argon2.verify_password(user.password.as_bytes(), &parsed_hash).is_ok();
            
            if is_valid {
                // Replace the placeholder token with proper JWT
                let token = match session::create_jwt(&db_user.id, &db_user.role) {
                    Ok(token) => token,
                    Err(e) => {
                        log::error!("Failed to create JWT: {}", e);
                        return HttpResponse::InternalServerError().json(
                            serde_json::json!({"error": "Authentication error"})
                        );
                    }
                };
                
                HttpResponse::Ok().json(UserResponse {
                    id: db_user.id,
                    username: db_user.username,
                    role: db_user.role,
                    token,
                })
            } else {
                HttpResponse::Unauthorized().json(
                    serde_json::json!({"error": "Invalid credentials"})
                )
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

pub async fn refresh_token(req: HttpRequest) -> impl Responder {
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                match session::verify_jwt(token) {
                    Ok(claims) => {
                        // Generate new token with the same claims but extended expiration
                        match session::create_jwt(&claims.sub, &claims.role) {
                            Ok(new_token) => {
                                return HttpResponse::Ok().json(serde_json::json!({
                                    "token": new_token
                                }));
                            }
                            Err(e) => {
                                log::error!("Failed to create new token: {}", e);
                                return HttpResponse::InternalServerError().json(
                                    serde_json::json!({"error": "Failed to refresh token"})
                                );
                            }
                        }
                    }
                    Err(e) => {
                        return HttpResponse::Unauthorized().json(
                            serde_json::json!({"error": format!("Invalid token: {}", e)})
                        );
                    }
                }
            }
        }
    }
    
    HttpResponse::Unauthorized().json(serde_json::json!({"error": "Authorization header missing"}))
}

pub async fn get_user_profile(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Check authorization
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = session::verify_jwt(token) {
                    // Get user details from database
                    match sqlx::query!(
                        "SELECT id, username, role, created_at FROM users WHERE id = ?",
                        claims.sub
                    )
                    .fetch_optional(&data.db)
                    .await {
                        Ok(Some(user)) => {
                            return HttpResponse::Ok().json(serde_json::json!({
                                "id": user.id,
                                "username": user.username,
                                "role": user.role,
                                "created_at": user.created_at
                            }));
                        }
                        Ok(None) => {
                            return HttpResponse::NotFound().json(
                                serde_json::json!({"error": "User not found"})
                            );
                        }
                        Err(e) => {
                            log::error!("Database error: {}", e);
                            return HttpResponse::InternalServerError().json(
                                serde_json::json!({"error": "Failed to fetch user profile"})
                            );
                        }
                    }
                }
            }
        }
    }
    
    HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
}

pub async fn get_all_users(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Check authorization
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(claims) = session::verify_jwt(token) {
                    // Only allow admin users to get all users
                    if claims.role != "admin" {
                        return HttpResponse::Forbidden().json(
                            serde_json::json!({"error": "Admin access required"})
                        );
                    }
                    
                    // Fetch all users from database
                    match sqlx::query!(
                        "SELECT id, username, role, created_at FROM users"
                    )
                    .fetch_all(&data.db)
                    .await {
                        Ok(users) => {
                            let users_json: Vec<serde_json::Value> = users
                                .iter()
                                .map(|u| {
                                    serde_json::json!({
                                        "id": u.id,
                                        "username": u.username,
                                        "role": u.role,
                                        "created_at": u.created_at
                                    })
                                })
                                .collect();
                            return HttpResponse::Ok().json(users_json);
                        }
                        Err(e) => {
                            log::error!("Failed to fetch users: {}", e);
                            return HttpResponse::InternalServerError().json(
                                serde_json::json!({"error": "Failed to fetch users"})
                            );
                        }
                    }
                }
            }
        }
    }
    
    HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
}

pub fn validate_token(req: HttpRequest) -> Result<session::Claims, String> {
    // Extract token from Authorization header
    let auth_header = req.headers().get("Authorization")
        .ok_or_else(|| "Authorization header missing".to_string())?;
    
    let auth_str = auth_header.to_str()
        .map_err(|_| "Invalid Authorization header".to_string())?;
    
    // Extract the token part
    let token = auth_str.strip_prefix("Bearer ")
        .ok_or_else(|| "Invalid token format. Expected 'Bearer <token>'".to_string())?;
    
    // Verify the JWT token
    session::verify_jwt(token)
        .map_err(|e| format!("Invalid token: {}", e))
}

pub fn init_routes() -> actix_web::Scope {
    web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/refresh", web::post().to(refresh_token))
        .route("/profile", web::get().to(get_user_profile))
        .route("/users", web::get().to(get_all_users))
}
