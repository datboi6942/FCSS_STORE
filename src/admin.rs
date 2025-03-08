// src/admin.rs
use actix_web::{web, HttpResponse, Responder, HttpRequest, get, post, put, delete};
use crate::AppState;
use sqlx;
use serde_json;
use log::{info, error};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;

// Admin dashboard HTML template
const ADMIN_DASHBOARD_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Admin Dashboard</title>
    <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/css/bootstrap.min.css" rel="stylesheet">
    <style>
        body { padding-top: 2rem; }
        .data-container { margin-top: 2rem; }
    </style>
</head>
<body>
    <div class="container">
        <h1 class="mb-4">Admin Dashboard</h1>
        
        <div class="row">
            <div class="col-md-4">
                <div class="card">
                    <div class="card-body">
                        <h5 class="card-title">Users</h5>
                        <p class="card-text">Manage system users</p>
                        <button class="btn btn-primary load-data" data-target="users">View Users</button>
                    </div>
                </div>
            </div>
            
            <div class="col-md-4">
                <div class="card">
                    <div class="card-body">
                        <h5 class="card-title">Orders</h5>
                        <p class="card-text">View and manage orders</p>
                        <button class="btn btn-primary load-data" data-target="orders">View Orders</button>
                    </div>
                </div>
            </div>
            
            <div class="col-md-4">
                <div class="card">
                    <div class="card-body">
                        <h5 class="card-title">Products</h5>
                        <p class="card-text">Manage product catalog</p>
                        <button class="btn btn-primary load-data" data-target="products">View Products</button>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="data-container mt-4">
            <div id="data-result" class="bg-light p-3 rounded">
                <p class="text-center text-muted">Select a category to view data</p>
            </div>
        </div>
    </div>
    
    <script>
        document.addEventListener('DOMContentLoaded', () => {
            const dataButtons = document.querySelectorAll('.load-data');
            const dataResult = document.getElementById('data-result');
            
            dataButtons.forEach(button => {
                button.addEventListener('click', async () => {
                    const target = button.getAttribute('data-target');
                    
                    try {
                        const response = await fetch(`/admin/${target}`, {
                            headers: {
                                'Authorization': 'Bearer admintoken'
                            }
                        });
                        
                        if (response.ok) {
                            const data = await response.json();
                            displayData(target, data);
                        } else {
                            dataResult.innerHTML = `<div class="alert alert-danger">Error loading ${target}</div>`;
                        }
                    } catch (error) {
                        dataResult.innerHTML = `<div class="alert alert-danger">Network error: ${error.message}</div>`;
                    }
                });
            });
            
            function displayData(type, data) {
                if (data.length === 0) {
                    dataResult.innerHTML = `<div class="alert alert-info">No ${type} found</div>`;
                    return;
                }
                
                let tableHTML = `
                    <h3 class="mb-3">${capitalizeFirstLetter(type)}</h3>
                    <div class="table-responsive">
                        <table class="table table-striped">
                            <thead>
                                <tr>
                `;
                
                // Create headers based on the first item's keys
                const headers = Object.keys(data[0]);
                headers.forEach(header => {
                    tableHTML += `<th>${capitalizeFirstLetter(header)}</th>`;
                });
                
                tableHTML += `
                                </tr>
                            </thead>
                            <tbody>
                `;
                
                // Add rows for each item
                data.forEach(item => {
                    tableHTML += '<tr>';
                    headers.forEach(header => {
                        tableHTML += `<td>${item[header] || ''}</td>`;
                    });
                    tableHTML += '</tr>';
                });
                
                tableHTML += `
                            </tbody>
                        </table>
                    </div>
                `;
                
                dataResult.innerHTML = tableHTML;
            }
            
            function capitalizeFirstLetter(string) {
                return string.charAt(0).toUpperCase() + string.slice(1);
            }
        });
    </script>
    <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.0-alpha1/dist/js/bootstrap.bundle.min.js"></script>
</body>
</html>
"#;

// Admin dashboard handler
pub async fn admin_dashboard() -> HttpResponse {
    info!("Serving admin dashboard");
    
    // Create a more robust HTML with better error handling
    let html = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Simple Admin Dashboard</title>
        <style>
            body { font-family: Arial, sans-serif; padding: 20px; }
            h1 { color: #333; }
            .btn { 
                padding: 8px 15px; 
                background-color: #4CAF50; 
                color: white; 
                border: none; 
                cursor: pointer;
                margin-right: 10px;
            }
            .error { color: red; font-weight: bold; }
            .status { margin: 20px 0; padding: 10px; background-color: #f5f5f5; }
            table { border-collapse: collapse; width: 100%; margin-top: 20px; }
            th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
            th { background-color: #f2f2f2; }
        </style>
    </head>
    <body>
        <h1>Admin Dashboard</h1>
        <div>
            <button class="btn" id="loadUsers">Load Users</button>
            <button class="btn" id="loadOrders">Load Orders</button>
            <button class="btn" id="loadProducts">Load Products</button>
        </div>
        
        <div class="status" id="status">Ready</div>
        
        <div id="result"></div>
        
        <script>
            // Display any errors or messages in the status div
            function showStatus(message, isError = false) {
                const statusEl = document.getElementById('status');
                statusEl.textContent = message;
                statusEl.className = isError ? 'status error' : 'status';
            }
            
            // Simple function to load data and display it
            async function loadData(endpoint) {
                showStatus(`Loading ${endpoint}...`);
                
                try {
                    // Log the fetch URL for debugging
                    console.log(`Fetching from: /admin/${endpoint}`);
                    
                    const response = await fetch(`/admin/${endpoint}`, {
                        headers: {
                            'Authorization': 'Bearer admintoken'
                        }
                    });
                    
                    // Log response status for debugging
                    console.log(`Response status: ${response.status}`);
                    
                    if (!response.ok) {
                        const errorText = await response.text();
                        throw new Error(`HTTP error ${response.status}: ${errorText}`);
                    }
                    
                    const data = await response.json();
                    console.log(`Data received:`, data);
                    
                    // Display the data as a table
                    displayTable(data, endpoint);
                    showStatus(`${endpoint} loaded successfully (${data.length} items)`);
                } catch (error) {
                    console.error(`Error loading ${endpoint}:`, error);
                    showStatus(`Error: ${error.message}`, true);
                    document.getElementById('result').innerHTML = 
                        `<p class="error">Failed to load ${endpoint}: ${error.message}</p>`;
                }
            }
            
            // Display data as a simple table
            function displayTable(data, title) {
                if (!data || data.length === 0) {
                    document.getElementById('result').innerHTML = 
                        `<p>No ${title} found</p>`;
                    return;
                }
                
                const columns = Object.keys(data[0]);
                
                let html = `<h2>${title.charAt(0).toUpperCase() + title.slice(1)}</h2>
                           <table>
                               <tr>`;
                               
                // Table headers
                columns.forEach(col => {
                    html += `<th>${col}</th>`;
                });
                html += '</tr>';
                
                // Table rows
                data.forEach(item => {
                    html += '<tr>';
                    columns.forEach(col => {
                        html += `<td>${item[col] !== null ? item[col] : ''}</td>`;
                    });
                    html += '</tr>';
                });
                
                html += '</table>';
                document.getElementById('result').innerHTML = html;
            }
            
            // Set up event listeners
            document.getElementById('loadUsers').addEventListener('click', () => loadData('users'));
            document.getElementById('loadOrders').addEventListener('click', () => loadData('orders'));
            document.getElementById('loadProducts').addEventListener('click', () => loadData('products'));
            
            // Show initial status
            showStatus('Click a button to load data');
        </script>
    </body>
    </html>
    "#;
    
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub async fn get_all_users(_req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    info!("Fetching all users");
    
    match sqlx::query!("SELECT id, username, role, created_at FROM users")
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
                
                info!("Found {} users in database", users_json.len());
                
                // Return a consistent format
                HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "users": users_json
                }))
            }
            Err(e) => {
                error!("Failed to fetch users: {}", e);
                HttpResponse::InternalServerError().json(
                    serde_json::json!({
                        "success": false,
                        "error": format!("Failed to fetch users: {}", e)
                    })
                )
            }
        }
}

// Fix product display in orders table
pub async fn get_all_orders(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Check authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        info!("Auth header found: {:?}", auth_header);
    } else {
        info!("No auth header, but allowing access for testing");
    }
    
    // Update query to match our actual database schema
    match sqlx::query!(
        "SELECT o.id, o.user_id, o.payment_id, o.status, o.created_at, 
                o.shipping_name, o.shipping_email, o.total_amount,
                u.username
         FROM orders o
         LEFT JOIN users u ON o.user_id = u.id
         ORDER BY o.created_at DESC"
    )
    .fetch_all(&data.db)
    .await
    {
        Ok(orders) => {
            let orders_json = orders
                .iter()
                .map(|o| {
                    info!("Order ID: {:?}", o.id);
                    
                    serde_json::json!({
                        "id": o.id,
                        "user_id": o.user_id,
                        "username": o.username,
                        "payment_id": o.payment_id,
                        "shipping_name": o.shipping_name,
                        "shipping_email": o.shipping_email,
                        "status": o.status,
                        "total": o.total_amount,
                        "created_at": o.created_at
                    })
                })
                .collect::<Vec<_>>();
            
            info!("Found {} orders in database", orders_json.len());
            HttpResponse::Ok().json(orders_json)
        }
        Err(e) => {
            error!("Failed to fetch orders: {}", e);
            HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to fetch orders: {}", e)})
            )
        }
    }
}

// Fix the unused variable in get_all_products
pub async fn get_all_products(_req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Always allow access for testing
    info!("Attempting to fetch all products from database");
    
    match sqlx::query!("SELECT * FROM products ORDER BY created_at DESC")
        .fetch_all(&data.db)
        .await
    {
        Ok(products) => {
            let products_json: Vec<serde_json::Value> = products
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "description": p.description,
                        "price": p.price,
                        "available": p.available,
                        "created_at": p.created_at
                    })
                })
                .collect();
            
            info!("Found {} products in database", products_json.len());
            return HttpResponse::Ok().json(products_json);
        }
        Err(e) => {
            error!("Failed to fetch products: {}", e);
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"error": format!("Failed to fetch products: {}", e)})
            );
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub role: String,
    pub created_at: i64,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

// Create a new user
#[post("/users")]
pub async fn create_user(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    user_data: web::Json<CreateUserRequest>
) -> impl Responder {
    // In production, verify admin rights here
    info!("Creating new user: {}", user_data.username);
    
    // Check if username already exists
    match sqlx::query("SELECT COUNT(*) as count FROM users WHERE username = ?")
        .bind(&user_data.username)
        .fetch_one(&app_state.db)
        .await {
            Ok(row) => {
                let count: i64 = row.get("count");
                if count > 0 {
                    return HttpResponse::BadRequest().json(json!({
                        "success": false,
                        "error": "Username already exists"
                    }));
                }
            },
            Err(e) => {
                error!("Database error checking username: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Database error"
                }));
            }
        }
    
    // In production, hash the password (use bcrypt, argon2, etc.)
    let password_hash = user_data.password.clone(); // Replace with real hashing
    
    let user_id = format!("usr-{}", Uuid::new_v4().simple());
    let now = Utc::now().timestamp();
    
    // Insert user
    match sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, created_at) VALUES (?, ?, ?, ?, ?)")
        .bind(&user_id)
        .bind(&user_data.username)
        .bind(&password_hash)
        .bind(&user_data.role)
        .bind(now)
        .execute(&app_state.db)
        .await {
            Ok(_) => {
                HttpResponse::Created().json(json!({
                    "success": true,
                    "message": "User created successfully",
                    "user": {
                        "id": user_id,
                        "username": user_data.username,
                        "role": user_data.role,
                        "created_at": now
                    }
                }))
            },
            Err(e) => {
                error!("Failed to create user: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to create user"
                }))
            }
        }
}

// Update a user
#[put("/users/{id}")]
pub async fn update_user(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    user_data: web::Json<UpdateUserRequest>
) -> impl Responder {
    let user_id = path.into_inner();
    info!("Updating user: {}", user_id);
    
    // Check if user exists
    match sqlx::query("SELECT COUNT(*) as count FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_one(&app_state.db)
        .await {
            Ok(row) => {
                let count: i64 = row.get("count");
                if count == 0 {
                    return HttpResponse::NotFound().json(json!({
                        "success": false,
                        "error": "User not found"
                    }));
                }
            },
            Err(e) => {
                error!("Database error checking user: {}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Database error"
                }));
            }
        }
    
    // Update user based on what was provided
    let mut query_parts = Vec::new();
    let mut bindings = Vec::new();
    
    if let Some(username) = &user_data.username {
        query_parts.push("username = ?");
        bindings.push(username.clone());
    }
    
    if let Some(password) = &user_data.password {
        if !password.is_empty() {
            // In production, hash the password
            let password_hash = password.clone(); // Replace with real hashing
            query_parts.push("password_hash = ?");
            bindings.push(password_hash);
        }
    }
    
    if let Some(role) = &user_data.role {
        query_parts.push("role = ?");
        bindings.push(role.clone());
    }
    
    if query_parts.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": "No fields to update"
        }));
    }
    
    let query = format!(
        "UPDATE users SET {} WHERE id = ?",
        query_parts.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    
    // Add bindings
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }
    
    // Add user ID for WHERE clause
    query_builder = query_builder.bind(&user_id);
    
    // Execute query
    match query_builder.execute(&app_state.db).await {
        Ok(_) => {
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "User updated successfully"
            }))
        },
        Err(e) => {
            error!("Failed to update user: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Failed to update user"
            }))
        }
    }
}

// Delete a user
#[delete("/users/{id}")]
pub async fn delete_user(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> impl Responder {
    let user_id = path.into_inner();
    info!("Deleting user: {}", user_id);
    
    match sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(&user_id)
        .execute(&app_state.db)
        .await {
            Ok(result) => {
                let rows_affected = result.rows_affected();
                if rows_affected > 0 {
                    HttpResponse::Ok().json(json!({
                        "success": true,
                        "message": "User deleted successfully"
                    }))
                } else {
                    HttpResponse::NotFound().json(json!({
                        "success": false,
                        "error": "User not found"
                    }))
                }
            },
            Err(e) => {
                error!("Failed to delete user: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to delete user"
                }))
            }
        }
}

// Get a single user
#[get("/users/{id}")]
pub async fn get_user(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<String>
) -> impl Responder {
    let user_id = path.into_inner();
    info!("Getting user: {}", user_id);
    
    match sqlx::query("SELECT id, username, role, created_at FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&app_state.db)
        .await {
            Ok(row_opt) => {
                match row_opt {
                    Some(row) => {
                        let user = json!({
                            "id": row.get::<String, _>("id"),
                            "username": row.get::<String, _>("username"),
                            "role": row.get::<String, _>("role"),
                            "created_at": row.get::<i64, _>("created_at")
                        });
                        
                        HttpResponse::Ok().json(json!({
                            "success": true,
                            "user": user
                        }))
                    },
                    None => {
                        HttpResponse::NotFound().json(json!({
                            "success": false,
                            "error": "User not found"
                        }))
                    }
                }
            },
            Err(e) => {
                error!("Failed to fetch user: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Failed to fetch user"
                }))
            }
        }
}

// Add this new struct for order status updates
#[derive(Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

// Add this function to get all orders with details
#[get("/orders")]
pub async fn admin_get_all_orders(_req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    info!("Admin: Fetching all orders with details");
    
    // This query joins orders with multiple tables to get comprehensive information
    let query = r#"
        SELECT 
            o.id, o.user_id, o.payment_id, o.status, o.shipping_name, 
            o.shipping_address, o.shipping_city, o.shipping_state,
            o.shipping_zip, o.shipping_country, o.shipping_email,
            o.total_amount, o.created_at, o.updated_at,
            mp.address as monero_address, mp.status as payment_status
        FROM orders o
        LEFT JOIN monero_payments mp ON o.payment_id = mp.payment_id
        ORDER BY o.created_at DESC
    "#;
    
    match sqlx::query(query)
        .fetch_all(&app_state.db)
        .await {
            Ok(rows) => {
                let orders = rows.iter().map(|row| {
                    // Get order id
                    let order_id = row.get::<String, _>("id");
                    
                    json!({
                        "id": order_id,
                        "user_id": row.get::<Option<String>, _>("user_id"),
                        "payment_id": row.get::<Option<String>, _>("payment_id"),
                        "status": row.get::<String, _>("status"),
                        "shipping_name": row.get::<String, _>("shipping_name"),
                        "shipping_address": row.get::<String, _>("shipping_address"),
                        "shipping_city": row.get::<String, _>("shipping_city"),
                        "shipping_state": row.get::<String, _>("shipping_state"),
                        "shipping_zip": row.get::<String, _>("shipping_zip"), 
                        "shipping_country": row.get::<String, _>("shipping_country"),
                        "shipping_email": row.get::<String, _>("shipping_email"),
                        "total_amount": row.get::<f64, _>("total_amount"),
                        "created_at": row.get::<i64, _>("created_at"),
                        "updated_at": row.get::<i64, _>("updated_at"),
                        "monero_address": row.get::<Option<String>, _>("monero_address"),
                        "payment_status": row.get::<Option<String>, _>("payment_status")
                    })
                }).collect::<Vec<_>>();
                
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "orders": orders
                }))
            },
            Err(e) => {
                error!("Failed to fetch orders: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": format!("Failed to fetch orders: {}", e)
                }))
            }
        }
}

// Add a function to get a single order with details
#[get("/orders/{id}")]
pub async fn admin_get_order(_req: HttpRequest, app_state: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let order_id = path.into_inner();
    info!("Admin: Fetching order details for ID: {}", order_id);
    
    // First get the order details
    let query = r#"
        SELECT 
            o.id, o.user_id, o.payment_id, o.status, o.shipping_name, 
            o.shipping_address, o.shipping_city, o.shipping_state,
            o.shipping_zip, o.shipping_country, o.shipping_email,
            o.total_amount, o.created_at, o.updated_at,
            mp.address as monero_address, mp.status as payment_status
        FROM orders o
        LEFT JOIN monero_payments mp ON o.payment_id = mp.payment_id
        WHERE o.id = ?
    "#;
    
    let order_result = sqlx::query(query)
        .bind(&order_id)
        .fetch_optional(&app_state.db)
        .await;
        
    match order_result {
        Ok(Some(row)) => {
            // Now get the order items
            let items_query = r#"
                SELECT oi.id, oi.product_id, oi.quantity, oi.price, p.name
                FROM order_items oi
                LEFT JOIN products p ON oi.product_id = p.id
                WHERE oi.order_number = ?
            "#;
            
            let items_result = sqlx::query(items_query)
                .bind(&order_id)
                .fetch_all(&app_state.db)
                .await;
                
            let items = match items_result {
                Ok(items) => {
                    items.iter().map(|item_row| {
                        json!({
                            "id": item_row.get::<i64, _>("id"),
                            "product_id": item_row.get::<String, _>("product_id"),
                            "name": item_row.get::<Option<String>, _>("name"),
                            "quantity": item_row.get::<i64, _>("quantity"),
                            "price": item_row.get::<f64, _>("price")
                        })
                    }).collect::<Vec<_>>()
                },
                Err(e) => {
                    error!("Failed to fetch order items: {}", e);
                    Vec::new()
                }
            };
            
            let order = json!({
                "id": row.get::<String, _>("id"),
                "user_id": row.get::<Option<String>, _>("user_id"),
                "payment_id": row.get::<Option<String>, _>("payment_id"),
                "status": row.get::<String, _>("status"),
                "shipping_name": row.get::<String, _>("shipping_name"),
                "shipping_address": row.get::<String, _>("shipping_address"),
                "shipping_city": row.get::<String, _>("shipping_city"),
                "shipping_state": row.get::<String, _>("shipping_state"),
                "shipping_zip": row.get::<String, _>("shipping_zip"), 
                "shipping_country": row.get::<String, _>("shipping_country"),
                "shipping_email": row.get::<String, _>("shipping_email"),
                "total_amount": row.get::<f64, _>("total_amount"),
                "created_at": row.get::<i64, _>("created_at"),
                "updated_at": row.get::<i64, _>("updated_at"),
                "monero_address": row.get::<Option<String>, _>("monero_address"),
                "payment_status": row.get::<Option<String>, _>("payment_status"),
                "items": items
            });
            
            HttpResponse::Ok().json(json!({
                "success": true,
                "order": order
            }))
        },
        Ok(None) => {
            HttpResponse::NotFound().json(json!({
                "success": false,
                "error": "Order not found"
            }))
        },
        Err(e) => {
            error!("Failed to fetch order: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to fetch order: {}", e)
            }))
        }
    }
}

// Add endpoint to update order status
#[put("/orders/{id}/status")]
pub async fn update_order_status(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    status_update: web::Json<UpdateOrderStatusRequest>
) -> impl Responder {
    let order_id = path.into_inner();
    let new_status = &status_update.status;
    
    info!("Updating order {} status to {}", order_id, new_status);
    
    let now = Utc::now().timestamp();
    
    match sqlx::query(
        "UPDATE orders SET status = ?, updated_at = ? WHERE id = ?"
    )
    .bind(new_status)
    .bind(now)
    .bind(&order_id)
    .execute(&app_state.db)
    .await {
        Ok(result) => {
            if result.rows_affected() > 0 {
                HttpResponse::Ok().json(json!({
                    "success": true,
                    "message": format!("Order status updated to {}", new_status)
                }))
            } else {
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "error": "Order not found"
                }))
            }
        },
        Err(e) => {
            error!("Failed to update order status: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Failed to update order status: {}", e)
            }))
        }
    }
}

// Update the init_routes function to correctly use the handlers
pub fn init_routes() -> actix_web::Scope {
    web::scope("/admin")
        .route("", web::get().to(admin_dashboard))
        .route("/", web::get().to(admin_dashboard))
        .route("/dashboard", web::get().to(admin_dashboard))
        .route("/panel", web::get().to(admin_dashboard))
        .route("/users", web::get().to(get_all_users))
        .service(admin_get_all_orders)  // Use service instead of route
        .service(
            web::scope("/api")
                .route("/users", web::get().to(get_all_users))
                .route("/orders", web::get().to(get_all_orders))  // Use the non-decorated function
                .route("/products", web::get().to(get_all_products))
        )
        .service(get_user)
        .service(create_user)
        .service(update_user)
        .service(delete_user)
        .service(admin_get_order)
        .service(update_order_status)
}
