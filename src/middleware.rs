use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::Error, HttpMessage
};
use futures_util::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use crate::session;

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
        ready(Ok(JwtMiddleware {
            service: Rc::new(service),
        }))
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
