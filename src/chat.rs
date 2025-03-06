// src/chat.rs
use actix_web::{web, HttpResponse, Responder, Scope};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;

/// A chat message structure.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub id: String,
    pub user: String,
    pub text: String,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ChatInput {
    pub user: String,
    pub text: String,
}

// Simple endpoint to get chat history
pub async fn chat_handler(data: web::Data<AppState>) -> impl Responder {
    let chat_history = data.chat_history.lock().unwrap();
    
    HttpResponse::Ok().json(&*chat_history)
}

// Endpoint to add a new message
pub async fn post_message(
    msg_data: web::Json<ChatInput>,
    data: web::Data<AppState>
) -> impl Responder {
    let message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        user: msg_data.user.clone(),
        text: msg_data.text.clone(),
        timestamp: Utc::now(),
    };
    
    // Add message to history
    {
        let mut history = data.chat_history.lock().unwrap();
        history.push(message.clone());
    }
    
    HttpResponse::Created().json(message)
}

// Return a properly configured Scope instead of a function
pub fn init_routes() -> Scope {
    web::scope("/chat")
        .route("", web::get().to(chat_handler))
        .route("", web::post().to(post_message))
}
