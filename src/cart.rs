use actix_web::{web, HttpResponse, Responder, post, HttpRequest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use log::{info, error};
use crate::AppState;
use crate::monero::MoneroPaymentRequest;
use chrono::{Utc};
use crate::auth;

// Define our data structures
pub type CartStore = Mutex<HashMap<String, Cart>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub id: String,
    pub user_id: String,
    pub items: Vec<CartItem>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: i32,
    pub price: f64,
    pub name: String,
    pub image: Option<String>,
}

// Request models
#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub user_id: String,
    pub product_id: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CartItemId {
    pub cart_id: String,
    pub item_index: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckoutRequest {
    pub items: Vec<CartItem>,
    pub total: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckoutResponse {
    pub success: bool,
    pub order_id: String,
    pub payment: Option<MoneroPaymentRequest>,
    pub message: Option<String>,
}

// Helper function to get or create a cart
fn get_or_create_cart(
    carts: &CartStore,
    user_id: &str,
) -> Cart {
    let mut carts = carts.lock().unwrap();
    
    // Try to find existing cart for user
    for (_id, cart) in carts.iter() {
        if cart.user_id == user_id {
            return cart.clone();
        }
    }
    
    // Create new cart if not found
    let now = chrono::Utc::now();
    let cart_id = format!("cart-{}", Uuid::new_v4().simple());
    
    let new_cart = Cart {
        id: cart_id.clone(),
        user_id: user_id.to_string(),
        items: Vec::new(),
        created_at: now,
        updated_at: now,
    };
    
    carts.insert(cart_id.clone(), new_cart.clone());
    new_cart
}

// Add item to cart
pub async fn add_to_cart(
    req: web::Json<AddToCartRequest>,
    state: web::Data<AppState>
) -> impl Responder {
    let add_request = req.into_inner();
    let user_id = &add_request.user_id;
    
    info!("Adding product {} to cart for user {}", add_request.product_id, user_id);
    
    // Check if product exists
    let product = match sqlx::query!(
        "SELECT id, name, price, available FROM products WHERE id = ?",
        add_request.product_id
    )
    .fetch_optional(&state.db)
    .await {
        Ok(Some(product)) => {
            if !product.available {
                return HttpResponse::BadRequest().json(
                    serde_json::json!({"error": "Product is not available"})
                );
            }
            product
        },
        Ok(None) => {
            return HttpResponse::NotFound().json(
                serde_json::json!({"error": "Product not found"})
            );
        },
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": "Failed to retrieve product"})
            );
        }
    };
    
    // Get or create cart
    let mut cart = get_or_create_cart(&state.carts, user_id);
    
    // Check if product already exists in cart
    let mut found = false;
    for item in &mut cart.items {
        if item.product_id == add_request.product_id {
            item.quantity += add_request.quantity;
            found = true;
            break;
        }
    }
    
    // Add new item if not found
    if !found {
        cart.items.push(CartItem {
            product_id: product.id.clone(),
            quantity: add_request.quantity,
            price: product.price,
            name: product.name.clone(),
            image: None,
        });
    }
    
    // Update cart
    cart.updated_at = chrono::Utc::now();
    
    // Save updated cart
    {
        let mut carts = state.carts.lock().unwrap();
        carts.insert(cart.id.clone(), cart.clone());
    }
    
    HttpResponse::Ok().json(cart)
}

// Remove item from cart
pub async fn remove_from_cart(
    path: web::Path<CartItemId>,
    state: web::Data<AppState>
) -> impl Responder {
    let params = path.into_inner();
    let cart_id = &params.cart_id;
    let item_index = params.item_index;
    
    info!("Removing item at index {} from cart {}", item_index, cart_id);
    
    // Find the cart
    let mut carts = state.carts.lock().unwrap();
    if let Some(mut cart) = carts.get(cart_id).cloned() {
        // Remove the item if index is valid
        if item_index < cart.items.len() {
            cart.items.remove(item_index);
            cart.updated_at = chrono::Utc::now();
            
            // Save updated cart
            carts.insert(cart_id.clone(), cart.clone());
            
            return HttpResponse::Ok().json(cart);
        } else {
            return HttpResponse::BadRequest().json(
                serde_json::json!({"error": "Invalid item index"})
            );
        }
    }
    
    HttpResponse::NotFound().json(
        serde_json::json!({"error": "Cart not found"})
    )
}

// Get cart contents
pub async fn get_cart(
    path: web::Path<String>,
    state: web::Data<AppState>
) -> impl Responder {
    let user_id = path.into_inner();
    
    info!("Getting cart for user {}", user_id);
    
    // Find cart for user
    let carts = state.carts.lock().unwrap();
    for (_id, cart) in carts.iter() {
        if cart.user_id == user_id {
            return HttpResponse::Ok().json(cart.clone());
        }
    }
    
    // Return empty cart if not found
    let empty_cart = get_or_create_cart(&state.carts, &user_id);
    HttpResponse::Ok().json(empty_cart)
}

// New checkout endpoint that uses Monero payments
#[post("/api/checkout")]
#[allow(non_snake_case)]
pub async fn checkout(
    app_state: web::Data<AppState>,
    checkout_data: web::Json<CheckoutRequest>,
) -> impl Responder {
    // Add debug logging
    println!("Received checkout request: {:?}", checkout_data);
    
    // Create a unique order ID
    let order_id = Uuid::new_v4().to_string();
    
    // Create Monero payment request
    let total_amount = checkout_data.total;
    let payment = app_state.monero_payments.create_payment_usd(order_id.clone(), total_amount);
    
    // Add debug logging
    println!("Created Monero payment: {:?}", payment);
    
    // Return the checkout response with payment details
    HttpResponse::Ok()
        .content_type("application/json")
        .json(CheckoutResponse {
            success: true,
            order_id,
            payment: Some(payment),
            message: Some("Please send Monero to the provided address".to_string()),
        })
}

// Direct checkout endpoint
#[post("/api/direct-checkout")]
pub async fn direct_checkout(
    app_state: web::Data<AppState>,
    checkout_data: web::Json<CheckoutRequest>,
    req: HttpRequest,
) -> impl Responder {
    println!("Received direct checkout request: {:?}", checkout_data);
    
    // Extract user ID from the authentication token
    let user_id = match auth::validate_token(req.clone()) {
        Ok(claims) => {
            println!("User authenticated, ID: {}", claims.sub);
            claims.sub
        },
        Err(e) => {
            println!("Auth error: {}. Using guest ID", e);
            "guest".to_string()
        }
    };
    
    // Create a unique order ID
    let order_id = Uuid::new_v4().to_string();
    
    // Create Monero payment request
    let total_amount = checkout_data.total;
    println!("Creating payment for total amount: {} for user {}", total_amount, user_id);
    
    let payment = app_state.monero_payments.create_payment_sync(order_id.clone(), total_amount);
    println!("Created Monero payment: {:?}", payment);
    
    // Store the order in the database with the user_id
    let now = Utc::now().timestamp();
    
    // Create the order record with proper debugging
    println!("Inserting order into database: id={}, user_id={}", order_id, user_id);
    match sqlx::query(
        "INSERT INTO orders (id, user_id, payment_id, status, shipping_name, shipping_address, 
         shipping_city, shipping_state, shipping_zip, shipping_country, shipping_email, 
         total_amount, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&order_id)
    .bind(&user_id)  // Make sure user_id is passed correctly
    .bind(&payment.payment_id)
    .bind("Pending")
    .bind("Customer")  // Default shipping info
    .bind("Address")
    .bind("City")
    .bind("State")
    .bind("Zip")
    .bind("Country")
    .bind("customer@example.com")
    .bind(total_amount)
    .bind(now)
    .bind(now)
    .execute(&app_state.db)
    .await {
        Ok(_) => {
            println!("✅ Successfully created order {} for user {}", order_id, user_id);
        },
        Err(e) => {
            println!("❌ Error creating order: {}", e);
            return HttpResponse::InternalServerError().json(CheckoutResponse {
                success: false,
                order_id: "".to_string(),
                payment: None,
                message: Some(format!("Failed to create order: {}", e)),
            });
        }
    }
    
    // Return the checkout response with payment details
    HttpResponse::Ok()
        .content_type("application/json")
        .json(CheckoutResponse {
            success: true,
            order_id,
            payment: Some(payment),
            message: Some("Please send Monero to the provided address".to_string()),
        })
}

// Initialize routes
pub fn init_routes() -> actix_web::Scope {
    web::scope("/cart")
        .route("/add", web::post().to(add_to_cart))
        .route("/{user_id}", web::get().to(get_cart))
        .route("/remove/{cart_id}/{item_index}", web::delete().to(remove_from_cart))
}