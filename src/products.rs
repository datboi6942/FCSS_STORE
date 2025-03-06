use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;
use sqlx::SqlitePool;
use log::{info, error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub available: bool,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct ProductInput {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub available: bool,
}

/// Endpoint to list all products.
pub async fn list_products(state: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query!(
        "SELECT id, name, description, price, available, created_at FROM products"
    )
    .fetch_all(&state.db)
    .await;
    
    match result {
        Ok(rows) => {
            let products: Vec<Product> = rows.into_iter().map(|row| {
                Product {
                    id: row.id,
                    name: row.name,
                    description: row.description,
                    price: row.price,
                    available: row.available,
                    created_at: Some(Utc::now()),
                }
            }).collect();
            
            HttpResponse::Ok().json(products)
        },
        Err(e) => {
            log::error!("Failed to fetch products: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to fetch products"})
            )
        }
    }
}

/// Endpoint to add a new product.
/// (In a production system, you'd secure this endpoint to admin users only.)
pub async fn add_product(
    product_data: web::Json<ProductInput>,
    state: web::Data<AppState>,
) -> impl Responder {
    let product = product_data.into_inner();
    let product_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let result = sqlx::query!(
        "INSERT INTO products (id, name, description, price, available, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        product_id,
        product.name,
        product.description,
        product.price,
        product.available,
        now
    )
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => {
            let created_product = Product {
                id: product_id.clone(),
                name: product.name,
                description: product.description,
                price: product.price,
                available: product.available,
                created_at: Some(now),
            };
            
            println!("Product successfully created in database with ID: {}", product_id);
            
            HttpResponse::Created().json(created_product)
        }
        Err(e) => {
            log::error!("Failed to add product: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to add product"})
            )
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .route("", web::get().to(list_products))
            .route("", web::post().to(add_product))
            .route("/ids", web::get().to(get_product_ids))
    );
}

pub async fn get_all_products(
    _pool: web::Data<SqlitePool>,
) -> impl Responder {
    HttpResponse::Ok().json("Products list")
}

// Add this function to handle direct product purchase
pub async fn purchase_product(
    purchase_req: web::Json<PurchaseRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let purchase = purchase_req.into_inner();
    
    info!("Processing purchase for product: {}, user: {}", 
          purchase.product_id, purchase.user_id);
    
    // Validate product exists and is available
    let product = sqlx::query!(
        "SELECT id, name, price, available FROM products WHERE id = ?",
        purchase.product_id
    )
    .fetch_optional(&state.db)
    .await;
    
    match product {
        Ok(Some(product)) => {
            if !product.available {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": "Product is not available for purchase"})
                );
            }
            
            // Generate a unique order ID
            let order_id = format!("ord-{}", Uuid::new_v4().simple());
            let now = Utc::now();
            
            // Create the order record
            let result = sqlx::query!(
                "INSERT INTO orders (id, user_id, product_id, status, created_at) VALUES (?, ?, ?, ?, ?)",
                order_id,
                purchase.user_id,
                purchase.product_id,
                "pending", // Initial status is pending until payment is completed
                now
            )
            .execute(&state.db)
            .await;
            
            match result {
                Ok(_) => {
                    // Return order details with payment information
                    HttpResponse::Created().json(serde_json::json!({
                        "order_id": order_id,
                        "product_id": purchase.product_id,
                        "product_name": product.name,
                        "price": product.price,
                        "status": "pending",
                        "created_at": now,
                        "payment_info": {
                            "amount": product.price,
                            "currency": "USD",
                            "crypto_address": "0xabc123...def456", // Replace with your actual crypto address
                            "payment_methods": ["BTC", "ETH", "USDT"]
                        }
                    }))
                },
                Err(e) => {
                    error!("Failed to create order: {}", e);
                    HttpResponse::InternalServerError().json(
                        serde_json::json!({"error": "Failed to create order"})
                    )
                }
            }
        },
        Ok(None) => {
            HttpResponse::NotFound().json(
                serde_json::json!({"error": "Product not found"})
            )
        },
        Err(e) => {
            error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to retrieve product"})
            )
        }
    }
}

// Define the request structure
#[derive(Deserialize)]
pub struct PurchaseRequest {
    pub user_id: String,
    pub product_id: String,
}

// Add this new function to get all product IDs
pub async fn get_product_ids(state: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query!(
        "SELECT id FROM products"
    )
    .fetch_all(&state.db)
    .await;
    
    match result {
        Ok(rows) => {
            let ids: Vec<String> = rows.into_iter().map(|row| row.id).collect();
            println!("Returning {} product IDs", ids.len());
            HttpResponse::Ok().json(ids)
        },
        Err(e) => {
            log::error!("Failed to fetch product IDs: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to fetch product IDs"})
            )
        }
    }
}
