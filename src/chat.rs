// src/chat.rs
use actix::{Actor, StreamHandler, AsyncContext, ActorContext};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, Scope};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;
use crate::AppState;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use log::{info, warn};
use crate::auth;
use sqlx;

/// A chat message structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub username: String,
    pub content: String,
    pub timestamp: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ChatInput {
    pub user: String,
    pub text: String,
}

// Define timeout constants
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// WebSocket session struct
pub struct ChatSession {
    pub id: String,
    pub username: String,
    pub chat_history: Arc<Mutex<Vec<ChatMessage>>>,
    pub heartbeat: Instant,
}

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;
    
    // Start heartbeat on session start
    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat(ctx);
    }
}

impl ChatSession {
    // Heartbeat to keep connection alive
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                warn!("Websocket client timed out: {}", act.id);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

// Handle WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let message = ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    username: self.username.clone(),
                    content: text.trim().to_string(),
                    timestamp: Utc::now(),
                };
                
                info!("Chat message from {}: {}", self.username, message.content);
                
                // Add message to history
                if let Ok(mut history) = self.chat_history.lock() {
                    history.push(message.clone());
                }
                
                // Broadcast the message
                let message_json = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string());
                ctx.text(message_json);
            }
            Ok(ws::Message::Binary(_)) => warn!("Unexpected binary message"),
            Ok(ws::Message::Close(reason)) => {
                info!("Connection closed for {}: {:?}", self.username, reason);
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

// WebSocket route handler
pub async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    app_data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    // Extract username from token or query params (simplified for demo)
    let username = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("anonymous")
        .trim_start_matches("Bearer ")
        .to_string();
    
    info!("WebSocket connection established for user: {}", username);
    
    // Create session
    let chat_session = ChatSession {
        id: Uuid::new_v4().to_string(),
        username,
        chat_history: app_data.chat_history.clone(),
        heartbeat: Instant::now(),
    };
    
    // Start WebSocket connection
    let resp = ws::start(chat_session, &req, stream)?;
    Ok(resp)
}

// Simple endpoint to get chat history
pub async fn chat_handler(_data: web::Data<AppState>) -> impl Responder {
    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Secure Store Chat</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 2rem; }
            #messages { height: 300px; overflow-y: scroll; border: 1px solid #ccc; padding: 1rem; margin-bottom: 1rem; }
            .message { margin-bottom: 0.5rem; }
            .user { font-weight: bold; }
            .input-area { display: flex; }
            #message-input { flex-grow: 1; padding: 0.5rem; margin-right: 0.5rem; }
            button { padding: 0.5rem 1rem; background-color: #4CAF50; color: white; border: none; cursor: pointer; }
        </style>
    </head>
    <body>
        <h1>Secure Store Chat</h1>
        <div id="messages"></div>
        <div class="input-area">
            <input type="text" id="message-input" placeholder="Type your message...">
            <button id="send-button">Send</button>
        </div>

        <script>
            const messagesDiv = document.getElementById('messages');
            const messageInput = document.getElementById('message-input');
            const sendButton = document.getElementById('send-button');
            
            // Load initial messages
            fetch('/chat/messages')
                .then(response => response.json())
                .then(messages => {
                    messages.forEach(msg => {
                        appendMessage(msg.user, msg.text, msg.timestamp);
                    });
                })
                .catch(error => console.error('Error loading messages:', error));
            
            // Send message function
            function sendMessage() {
                const text = messageInput.value.trim();
                if (!text) return;
                
                fetch('/chat/message', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': 'Bearer ' + localStorage.getItem('token')
                    },
                    body: JSON.stringify({ text })
                })
                .then(response => {
                    if (response.ok) {
                        messageInput.value = '';
                        return response.json();
                    }
                    throw new Error('Failed to send message');
                })
                .then(data => {
                    appendMessage(data.user, data.text, data.timestamp);
                })
                .catch(error => console.error('Error:', error));
            }
            
            // Append message to chat
            function appendMessage(user, text, timestamp) {
                const msgElement = document.createElement('div');
                msgElement.className = 'message';
                msgElement.innerHTML = `<span class="user">${user}:</span> ${text} <span class="timestamp">${new Date(timestamp).toLocaleTimeString()}</span>`;
                messagesDiv.appendChild(msgElement);
                messagesDiv.scrollTop = messagesDiv.scrollHeight;
            }
            
            // Event listeners
            sendButton.addEventListener('click', sendMessage);
            messageInput.addEventListener('keypress', event => {
                if (event.key === 'Enter') sendMessage();
            });
            
            // Periodically check for new messages
            setInterval(() => {
                fetch('/chat/messages?since=' + encodeURIComponent(lastMessageTime))
                    .then(response => response.json())
                    .then(messages => {
                        messages.forEach(msg => {
                            appendMessage(msg.user, msg.text, msg.timestamp);
                            lastMessageTime = msg.timestamp;
                        });
                    })
                    .catch(error => console.error('Error loading messages:', error));
            }, 5000);
            
            let lastMessageTime = new Date().toISOString();
        </script>
    </body>
    </html>
    "#;
    
    HttpResponse::Ok().content_type("text/html").body(html)
}

// Endpoint to add a new message
pub async fn post_message(
    data: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<ChatInput>
) -> impl Responder {
    // Verify the user is authenticated
    let claims = match auth::validate_token(req) {
        Ok(claims) => claims,
        Err(e) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": format!("Invalid token: {}", e)})),
    };
    
    // Get username from database
    let username = match sqlx::query!("SELECT username FROM users WHERE id = ?", claims.sub)
        .fetch_optional(&data.db)
        .await {
            Ok(Some(user)) => user.username,
            Ok(None) => return HttpResponse::Unauthorized().json(serde_json::json!({"error": "User not found"})),
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": format!("Database error: {}", e)})),
        };
    
    // Create new message
    let new_message = ChatMessage {
        id: Uuid::new_v4().to_string(),
        username: username.clone(),
        content: payload.text.clone(),
        timestamp: Utc::now(),
    };
    
    // Add to chat history
    {
        let mut chat_history = data.chat_history.lock().unwrap();
        chat_history.push(new_message.clone());
    }
    
    HttpResponse::Ok().json(new_message)
}

// Return a properly configured Scope instead of a function
pub fn init_routes() -> Scope {
    web::scope("/chat")
        .route("", web::get().to(chat_handler))
        .route("/message", web::post().to(post_message))
}
