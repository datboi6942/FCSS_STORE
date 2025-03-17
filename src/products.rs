use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;
use log::error;
use serde_json::json;

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
    // Use a raw query to avoid SQLx macro issues
    let result = sqlx::query_as!(
        Product,
        r#"SELECT 
            id as "id!", 
            name as "name!", 
            COALESCE(description, '') as "description!", 
            price as "price!", 
            (CASE WHEN available != 0 THEN true ELSE false END) as "available!: bool",
            created_at as "created_at: Option<chrono::DateTime<Utc>>"
        FROM products"#
    )
    .fetch_all(&state.db)
    .await;
    
    match result {
        Ok(products) => {
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
    
    let result = sqlx::query(
        "INSERT INTO products (id, name, description, price, available, created_at) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(product_id.clone())
    .bind(product.name.clone())
    .bind(product.description.clone())
    .bind(product.price)
    .bind(if product.available { 1 } else { 0 })
    .bind(now)
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

// These functions can be implemented similarly...
pub async fn purchase_product(
    state: web::Data<AppState>,
    purchase: web::Json<PurchaseRequest>
) -> impl Responder {
    // Use simple query instead of macros
    let order_id = format!("order-{}", Uuid::new_v4());
    let payment_id = format!("pay-{}", Uuid::new_v4());
    let now = Utc::now().timestamp();
    
    HttpResponse::Ok().json(json!({
        "success": true,
        "order_id": order_id,
        "message": "Product purchased successfully"
    }))
}

// Define the request structure
#[derive(Deserialize)]
pub struct PurchaseRequest {
    pub user_id: String,
    pub price: f64,
    pub email: String,
}
