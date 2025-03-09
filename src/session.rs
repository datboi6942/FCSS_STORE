/// Stub for session-related functionality.

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use log::error;

// Define the JWT secret directly here
const JWT_SECRET: &[u8] = b"default_secret_change_in_production";

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // subject (user ID)
    pub role: String, // user role (admin, user, etc.)
    pub exp: usize,   // expiration time
    pub iat: usize,   // issued at
}

/// Create JWT token
pub fn create_jwt(user_id: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    
    let claims = Claims {
        sub: user_id.to_owned(),
        role: role.to_owned(),
        exp: expiration,
        iat: Utc::now().timestamp() as usize,
    };
    
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_change_in_production".to_string());
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

/// Verify JWT token
pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation
    ) {
        Ok(c) => c,
        Err(err) => {
            // Print the token and error for debugging
            error!("Token validation error: {}", err);
            error!("Token: {}", token);
            return Err(format!("Invalid token: {}", err));
        }
    };
    
    Ok(token_data.claims)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub expires: i64,
}

impl Session {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            expires: chrono::Utc::now().timestamp() + 3600, // 1 hour from now
        }
    }
}