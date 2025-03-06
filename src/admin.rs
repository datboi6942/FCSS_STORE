// src/admin.rs
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::AppState;
use sqlx;
use serde_json;
use log::{info, error};

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

pub async fn get_all_users(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    // Log all headers for debugging
    for (name, value) in req.headers().iter() {
        info!("Header: {} = {:?}", name, value);
    }
    
    // Check if request has authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        info!("Auth header found: {:?}", auth_header);
        
        // Accept any authorization for testing purposes
        info!("Attempting to fetch all users from database");
        
        match sqlx::query!("SELECT id, username, role, created_at FROM users")
            .fetch_all(&data.db)
            .await
        {
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
                return HttpResponse::Ok().json(users_json);
            }
            Err(e) => {
                error!("Failed to fetch users: {}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": format!("Failed to fetch users: {}", e)})
                );
            }
        }
    } else {
        // For testing, allow access even without authorization
        info!("No auth header, but allowing access for testing");
        
        match sqlx::query!("SELECT id, username, role, created_at FROM users")
            .fetch_all(&data.db)
            .await
        {
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
                return HttpResponse::Ok().json(users_json);
            }
            Err(e) => {
                error!("Failed to fetch users: {}", e);
                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"error": format!("Failed to fetch users: {}", e)})
                );
            }
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
    
    // Query orders with all needed fields
    match sqlx::query!(
        "SELECT o.id, o.user_id, o.product_id, o.status, o.created_at, 
                u.username, p.name as product_name, p.price
         FROM orders o
         JOIN users u ON o.user_id = u.id
         JOIN products p ON o.product_id = p.id
         ORDER BY o.created_at DESC"
    )
    .fetch_all(&data.db)
    .await
    {
        Ok(orders) => {
            // Create fixed structure JSON with all required fields explicitly set
            let orders_json = orders
                .iter()
                .map(|o| {
                    // Log each order's details to help with debugging
                    info!("Order ID: {}, Product: {:?}", o.id, o.product_name);
                    
                    // Create a JSON object that includes additional fields the frontend expects
                    let obj = serde_json::json!({
                        "id": o.id.clone(),
                        "user_id": o.user_id.clone(),
                        "username": o.username,
                        "product_id": o.product_id.clone(),
                        "product_name": o.product_name,
                        "product": o.product_name,  // Add this field - frontend might be looking for "product" 
                        "price": o.price,
                        "status": o.status.clone(),
                        "created_at": o.created_at,
                        "total": o.price
                    });
                    
                    obj
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

// Change the init_routes function to return a Scope
pub fn init_routes() -> actix_web::Scope {
    web::scope("/admin")
        .route("", web::get().to(admin_dashboard))
        .route("/", web::get().to(admin_dashboard))
        .route("/dashboard", web::get().to(admin_dashboard))
        .route("/panel", web::get().to(admin_dashboard))
        .route("/users", web::get().to(get_all_users))
        .route("/orders", web::get().to(get_all_orders))
        .service(
            web::scope("/api")
                .route("/users", web::get().to(get_all_users))
                .route("/orders", web::get().to(get_all_orders))
                .route("/products", web::get().to(get_all_products))
        )
}
