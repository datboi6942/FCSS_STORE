// src/auth.rs
use actix_web::{web, HttpResponse, Responder, HttpRequest, HttpMessage};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;
use crate::session;
use crate::middleware::JwtAuthentication;
use actix_web::http::header;

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

pub async fn get_user_profile(req: HttpRequest) -> impl Responder {
    // Get claims from the request extensions
    if let Some(claims) = req.extensions().get::<session::Claims>() {
        return HttpResponse::Ok().json(serde_json::json!({
            "id": claims.sub,
            "role": claims.role
        }));
    }
    
    HttpResponse::Unauthorized().json(serde_json::json!({"error": "Not authenticated"}))
}

pub fn init_routes() -> actix_web::Scope {
    web::scope("/auth")
        .route("/register", web::post().to(register))
        .route("/login", web::post().to(login))
        .route("/refresh", web::post().to(refresh_token))
        .service(
            web::resource("/profile")
                .wrap(JwtAuthentication)
                .route(web::get().to(get_user_profile))
        )
}
