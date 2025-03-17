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

// Add this function to handle direct product purchase
pub async fn purchase_product(
    state: web::Data<AppState>,
    purchase: web::Json<PurchaseRequest>
) -> impl Responder {
    let order_id = format!("ord-{}", Uuid::new_v4().simple());
    let now = Utc::now().timestamp();
    
    // Create payment record first
    let payment_id = format!("pay-{}", Uuid::new_v4().simple());
    
    // Insert payment - handle errors explicitly
    match sqlx::query(
        "INSERT INTO monero_payments (payment_id, amount, address, status, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&payment_id)
    .bind(purchase.price)
    .bind("44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A")
    .bind("Pending")
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to create payment: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to create payment: {}", e)
            }));
        }
    }
    
    // Insert order without product_id
    let result = sqlx::query(
        "INSERT INTO orders (id, user_id, payment_id, status, shipping_name, shipping_address, 
         shipping_city, shipping_state, shipping_zip, shipping_country, shipping_email, 
         total_amount, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&order_id)
    .bind(&purchase.user_id)
    .bind(&payment_id)
    .bind("Pending")
    .bind("Customer") // Default shipping name
    .bind("") // Default empty address
    .bind("") // Default empty city
    .bind("") // Default empty state
    .bind("") // Default empty zip
    .bind("") // Default empty country
    .bind(&purchase.email)
    .bind(purchase.price)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await;
    
    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "order_id": order_id,
            "message": "Product purchased successfully"
        })),
        Err(e) => {
            error!("Failed to create order: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to create order: {}", e)
            }))
        }
    }
}

// Define the request structure
#[derive(Deserialize)]
pub struct PurchaseRequest {
    pub user_id: String,
    pub price: f64,
    pub email: String,
}
