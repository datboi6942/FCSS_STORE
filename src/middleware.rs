use actix_web::{
    body::{EitherBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use crate::session;
use serde_json::json;

// Define our middleware struct
pub struct JwtAuthentication;

// Define the middleware factory
impl<S, B> Transform<S, ServiceRequest> for JwtAuthentication 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddleware {
            service: Rc::new(service),
        })
    }
}

// The middleware itself
pub struct JwtMiddleware<S> {
    service: Rc<S>,
}

// Implement the service for the middleware
impl<S, B> Service<ServiceRequest> for JwtMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone services because they might be used across threads
        let srv = self.service.clone();

        Box::pin(async move {
            // Get authorization header
            if let Some(auth_header) = req.headers().get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    // Check if it starts with "Bearer "
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        // Verify JWT
                        match session::verify_jwt(token) {
                            Ok(claims) => {
                                // Add claims to request extensions for later use
                                req.extensions_mut().insert(claims);
                                let res = srv.call(req).await?;
                                return Ok(res);
                            }
                            Err(e) => {
                                return Err(actix_web::error::ErrorUnauthorized(format!(
                                    "Invalid token: {}", e
                                )));
                            }
                        }
                    }
                }
            }
            
            // No valid authorization header
            Err(actix_web::error::ErrorUnauthorized("Authorization required"))
        })
    }
}

// Admin authorization middleware
pub fn admin_required() -> AdminMiddleware {
    AdminMiddleware {}
}

pub struct AdminMiddleware {}

// Update the Transform implementation to return EitherBody responses
impl<S, B> Transform<S, ServiceRequest> for AdminMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AdminMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminMiddlewareService { service })
    }
}

pub struct AdminMiddlewareService<S> {
    service: S,
}

// Implement Service with EitherBody as the response body type
impl<S, B> Service<ServiceRequest> for AdminMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        // Check if user is admin
        let is_admin = auth_header
            .and_then(|header| header.to_str().ok())
            .and_then(|header_str| {
                if header_str.starts_with("Bearer ") {
                    Some(header_str[7..].to_string())
                } else {
                    None
                }
            })
            .map(|token| token.contains("admin"))
            .unwrap_or(false);

        if is_admin {
            // User is an admin, proceed with the request
            let fut = self.service.call(req);
            Box::pin(async move {
                match fut.await {
                    Ok(res) => Ok(res.map_into_left_body()),
                    Err(err) => Err(err),
                }
            })
        } else {
            // User is not an admin, return forbidden
            Box::pin(async move {
                let response = HttpResponse::Forbidden()
                    .json(json!({
                        "error": "Admin access required",
                        "message": "You don't have permission to access this resource"
                    }));
                
                let (req, _) = req.into_parts();
                // Using right body for the custom response
                Ok(ServiceResponse::new(req, response).map_into_right_body())
            })
        }
    }
}
