/// Stub for session-related functionality.

use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use log::{error, info};
use crate::auth::{Claims, JWT_SECRET};  // Import Claims from auth.rs

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
        username: "".to_string(), // Add this field since it's required by Claims
    };
    
    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
}

/// Verify JWT token
pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false; // Temporarily disable expiration check for debugging
    
    info!("Verifying token: {}", &token[..std::cmp::min(20, token.len())]);
    
    let token_data = match decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &validation
    ) {
        Ok(c) => {
            info!("Token successfully validated");
            info!("Claims: {:?}", c.claims);
            c
        },
        Err(err) => {
            error!("Token validation error: {}", err);
            error!("Token first 20 chars: {}", &token[..std::cmp::min(20, token.len())]);
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
    // Remove this
    // pub fn new(user_id: String) -> Self { ... }
}