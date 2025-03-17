use actix::{Actor, StreamHandler, Addr, AsyncContext, ActorContext};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use serde_json::json;
use log::info;
use crate::AppState;
use crate::monero::PaymentStatus;
use std::collections::HashMap;


const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// WebSocket session data
pub struct PaymentWebsocket {
    order_id: String,
    app_state: web::Data<AppState>,
    heartbeat: Instant,
    last_status: Option<PaymentStatus>,
}

// Store of all active connections by order ID
pub struct WebsocketConnections {
    connections: HashMap<String, Vec<Addr<PaymentWebsocket>>>,
}

impl WebsocketConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
    
    pub(crate) fn add_connection(&mut self, order_id: String, addr: Addr<PaymentWebsocket>) {
        self.connections.entry(order_id.clone())
            .or_insert_with(Vec::new)
            .push(addr);
        info!("New websocket connection for order {}, total: {}", 
            order_id, 
            self.connections.get(&order_id).map(|v| v.len()).unwrap_or(0));
    }
    
    pub(crate) fn remove_connection(&mut self, order_id: &str, addr: &Addr<PaymentWebsocket>) {
        if let Some(conns) = self.connections.get_mut(order_id) {
            conns.retain(|a| a != addr);
            if conns.is_empty() {
                self.connections.remove(order_id);
            }
        }
    }
    
    pub fn notify_payment_status(&self, order_id: &str, status: PaymentStatus) {
        if let Some(conns) = self.connections.get(order_id) {
            for conn in conns {
                conn.do_send(PaymentStatusMessage(status.clone()));
            }
        }
    }
}

// Message type for payment status updates
struct PaymentStatusMessage(PaymentStatus);

impl actix::Message for PaymentStatusMessage {
    type Result = ();
}

impl Actor for PaymentWebsocket {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat process
        self.heartbeat(ctx);
        
        // Register with the connection store
        let order_id = self.order_id.clone();
        let app_state_clone = self.app_state.clone();
        
        // Get current payment status and send it
        if let Some(payment) = app_state_clone.monero_payments.get_payment_by_order_id(&order_id) {
            self.last_status = Some(payment.status.clone());
            
            let status_message = json!({
                "type": "payment_status",
                "status": format!("{:?}", payment.status),
                "order_id": order_id
            }).to_string();
            
            ctx.text(status_message);
        }
        
        // Add this connection to the store
        let mut connections = app_state_clone.ws_connections.lock().unwrap();
        connections.add_connection(order_id, ctx.address());
    }
    
    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        // Remove from connection store
        let mut connections = self.app_state.ws_connections.lock().unwrap();
        connections.remove_connection(&self.order_id, &ctx.address());
        
        actix::Running::Stop
    }
}

// Handle incoming websocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PaymentWebsocket {
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
                // Handle command messages from client
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(json) => {
                        if let Some(command) = json["command"].as_str() {
                            match command {
                                "check_status" => {
                                    // Force check payment status
                                    let order_id = self.order_id.clone();
                                    if let Some(payment) = self.app_state.monero_payments.get_payment_by_order_id(&order_id) {
                                        let status_message = json!({
                                            "type": "payment_status",
                                            "status": format!("{:?}", payment.status),
                                            "order_id": order_id
                                        }).to_string();
                                        
                                        ctx.text(status_message);
                                    }
                                },
                                _ => {
                                    ctx.text(json!({
                                        "type": "error",
                                        "message": format!("Unknown command: {}", command)
                                    }).to_string());
                                }
                            }
                        }
                    },
                    Err(e) => {
                        ctx.text(json!({
                            "type": "error",
                            "message": format!("Invalid JSON: {}", e)
                        }).to_string());
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

// Handle payment status updates
impl actix::Handler<PaymentStatusMessage> for PaymentWebsocket {
    type Result = ();
    
    fn handle(&mut self, msg: PaymentStatusMessage, ctx: &mut Self::Context) {
        // Only send updates if the status has changed
        if self.last_status.as_ref() != Some(&msg.0) {
            self.last_status = Some(msg.0.clone());
            
            let status_message = json!({
                "type": "payment_status",
                "status": format!("{:?}", msg.0),
                "order_id": self.order_id
            }).to_string();
            
            ctx.text(status_message);
        }
    }
}

impl PaymentWebsocket {
    // Heartbeat to keep the connection alive
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.heartbeat) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }
            
            ctx.ping(b"");
        });
    }
}

// HTTP handler for WebSocket connection
pub async fn payment_ws(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let order_id = path.into_inner();
    info!("WebSocket connection request for order: {}", order_id);
    
    let ws = PaymentWebsocket {
        order_id,
        app_state: app_state.clone(),
        heartbeat: Instant::now(),
        last_status: None,
    };
    
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
} 