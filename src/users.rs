use actix_web::{web, HttpResponse, Responder, HttpRequest, get, post, put, delete};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use sqlx::Row;
use crate::AppState;
use crate::auth;
use log::error;
use serde_json::json;

#[derive(Deserialize, Serialize)]
pub struct Address {
    pub id: Option<String>,
    pub name: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
    pub is_default: bool,
}

// Get all addresses for the current user
#[get("/addresses")]
pub async fn get_user_addresses(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    // Validate authentication
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": e
            }));
        }
    };
    
    let user_id = claims.sub;
    
    // Query addresses
    match sqlx::query(
        "SELECT id, name, address, city, state, zip, country, is_default FROM addresses WHERE user_id = ?"
    )
    .bind(&user_id)
    .fetch_all(&app_state.db)
    .await {
        Ok(rows) => {
            let addresses: Vec<serde_json::Value> = rows.iter().map(|row| {
                json!({
                    "id": row.get::<String, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "address": row.get::<String, _>("address"),
                    "city": row.get::<String, _>("city"),
                    "state": row.get::<String, _>("state"),
                    "zip": row.get::<String, _>("zip"),
                    "country": row.get::<String, _>("country"),
                    "isDefault": row.get::<bool, _>("is_default")
                })
            }).collect();
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "addresses": addresses
            }))
        },
        Err(e) => {
            error!("Database error fetching addresses: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to retrieve addresses"
            }))
        }
    }
}

// Get a specific address
#[get("/addresses/{id}")]
pub async fn get_address(
    req: HttpRequest, 
    path: web::Path<String>, 
    app_state: web::Data<AppState>
) -> impl Responder {
    // Validate authentication
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": e
            }));
        }
    };
    
    let user_id = claims.sub;
    let address_id = path.into_inner();
    
    // Query address
    match sqlx::query(
        "SELECT id, name, address, city, state, zip, country, is_default FROM addresses WHERE id = ? AND user_id = ?"
    )
    .bind(&address_id)
    .bind(&user_id)
    .fetch_optional(&app_state.db)
    .await {
        Ok(Some(row)) => {
            let address = json!({
                "id": row.get::<String, _>("id"),
                "name": row.get::<String, _>("name"),
                "address": row.get::<String, _>("address"),
                "city": row.get::<String, _>("city"),
                "state": row.get::<String, _>("state"),
                "zip": row.get::<String, _>("zip"),
                "country": row.get::<String, _>("country"),
                "isDefault": row.get::<bool, _>("is_default")
            });
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "address": address
            }))
        },
        Ok(None) => {
            HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Address not found"
            }))
        },
        Err(e) => {
            error!("Database error fetching address: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to retrieve address"
            }))
        }
    }
}

// Create a new address
#[post("/addresses")]
pub async fn create_address(
    req: HttpRequest, 
    address: web::Json<Address>, 
    app_state: web::Data<AppState>
) -> impl Responder {
    // Validate authentication
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": e
            }));
        }
    };
    
    let user_id = claims.sub;
    let address_data = address.into_inner();
    
    // If this is set as default, first update all other addresses to not be default
    if address_data.is_default {
        match sqlx::query("UPDATE addresses SET is_default = 0 WHERE user_id = ?")
            .bind(&user_id)
            .execute(&app_state.db)
            .await {
                Ok(_) => {},
                Err(e) => {
                    error!("Database error updating default addresses: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": "Failed to update default address status"
                    }));
                }
            };
    }
    
    // Generate ID and current timestamp
    let address_id = format!("addr-{}", Uuid::new_v4().simple());
    let now = Utc::now().timestamp();
    
    // Insert address
    match sqlx::query(
        "INSERT INTO addresses (id, user_id, name, address, city, state, zip, country, is_default, created_at) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&address_id)
    .bind(&user_id)
    .bind(&address_data.name)
    .bind(&address_data.address)
    .bind(&address_data.city)
    .bind(&address_data.state)
    .bind(&address_data.zip)
    .bind(&address_data.country)
    .bind(&address_data.is_default)
    .bind(now)
    .execute(&app_state.db)
    .await {
        Ok(_) => {
            HttpResponse::Created().json(json!({
                "success": true,
                "message": "Address created successfully",
                "id": address_id
            }))
        },
        Err(e) => {
            error!("Database error creating address: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to create address"
            }))
        }
    }
}

// Update an address
#[put("/addresses/{id}")]
pub async fn update_address(
    req: HttpRequest, 
    path: web::Path<String>,
    address: web::Json<Address>, 
    app_state: web::Data<AppState>
) -> impl Responder {
    // Validate authentication
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": e
            }));
        }
    };
    
    let user_id = claims.sub;
    let address_id = path.into_inner();
    let address_data = address.into_inner();
    
    // Check if address exists and belongs to the user
    match sqlx::query("SELECT id FROM addresses WHERE id = ? AND user_id = ?")
        .bind(&address_id)
        .bind(&user_id)
        .fetch_optional(&app_state.db)
        .await {
            Ok(Some(_)) => {},
            Ok(None) => {
                return HttpResponse::NotFound().json(json!({
                    "success": false,
                    "error": "Address not found or does not belong to this user"
                }));
            },
            Err(e) => {
                error!("Database error checking address ownership: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to verify address ownership"
                }));
            }
        };
    
    // If this is set as default, first update all other addresses to not be default
    if address_data.is_default {
        match sqlx::query("UPDATE addresses SET is_default = 0 WHERE user_id = ? AND id != ?")
            .bind(&user_id)
            .bind(&address_id)
            .execute(&app_state.db)
            .await {
                Ok(_) => {},
                Err(e) => {
                    error!("Database error updating default addresses: {}", e);
                    return HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "error": "Failed to update default address status"
                    }));
                }
            };
    }
    
    // Update the address
    match sqlx::query(
        "UPDATE addresses SET name = ?, address = ?, city = ?, state = ?, zip = ?, country = ?, is_default = ? WHERE id = ? AND user_id = ?"
    )
    .bind(&address_data.name)
    .bind(&address_data.address)
    .bind(&address_data.city)
    .bind(&address_data.state)
    .bind(&address_data.zip)
    .bind(&address_data.country)
    .bind(&address_data.is_default)
    .bind(&address_id)
    .bind(&user_id)
    .execute(&app_state.db)
    .await {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Address updated successfully"
            }))
        },
        Err(e) => {
            error!("Database error updating address: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to update address"
            }))
        }
    }
}

// Delete an address
#[delete("/addresses/{id}")]
pub async fn delete_address(
    req: HttpRequest, 
    path: web::Path<String>, 
    app_state: web::Data<AppState>
) -> impl Responder {
    // Validate authentication
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => {
            return HttpResponse::Unauthorized().json(json!({
                "success": false,
                "error": e
            }));
        }
    };
    
    let user_id = claims.sub;
    let address_id = path.into_inner();
    
    // Delete the address
    match sqlx::query("DELETE FROM addresses WHERE id = ? AND user_id = ?")
        .bind(&address_id)
        .bind(&user_id)
        .execute(&app_state.db)
        .await {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    HttpResponse::Ok().json(json!({
                        "success": true,
                        "message": "Address deleted successfully"
                    }))
                } else {
                    HttpResponse::NotFound().json(json!({
                        "success": false,
                        "error": "Address not found or does not belong to this user"
                    }))
                }
            },
            Err(e) => {
                error!("Database error deleting address: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to delete address"
                }))
            }
        }
}

// Function to initialize user routes
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(get_user_addresses)
            .service(get_address)
            .service(create_address)
            .service(update_address)
            .service(delete_address)
    );
} 