use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;

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
            HttpResponse::Created().json(
                serde_json::json!({
                    "message": "Product added successfully",
                    "product_id": product_id
                })
            )
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
    );
}
