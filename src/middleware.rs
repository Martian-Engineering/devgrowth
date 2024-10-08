use crate::auth::validate_token;
use actix_session::SessionExt;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use futures::future::{ok, Ready};
use log::info;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService { service })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // List of paths that don't require authentication
        let public_paths = vec!["/api/auth/signin", "/api/auth/callback"];
        info!("ServiceRequest: {:?}", req);

        if public_paths
            .iter()
            .any(|&path| req.path().starts_with(path))
        {
            return Box::pin(self.service.call(req));
        }
        let auth_header = req.headers().get("Authorization");
        match auth_header {
            Some(auth_value) => {
                if let Ok(auth_str) = auth_value.to_str() {
                    if auth_str.starts_with("Bearer ") {
                        let token = &auth_str[7..];
                        match validate_token(token) {
                            Ok(claims) => {
                                // Token is valid, you can use claims if needed
                                req.extensions_mut().insert(claims);
                                let fut = self.service.call(req);
                                return Box::pin(async move {
                                    let res = fut.await?;
                                    Ok(res)
                                });
                            }
                            Err(e) => {
                                log::error!("Token validation error: {:?}", e);
                                return Box::pin(
                                    async move { Err(ErrorUnauthorized("Invalid token")) },
                                );
                            }
                        }
                    }
                }
            }
            None => {}
        }
        Box::pin(async move { Err(ErrorUnauthorized("Missing or invalid Authorization header")) })
    }
}

// -----------------------------------------------------------------------------

pub struct SessionLogger;

impl<S, B> actix_web::dev::Transform<S, ServiceRequest> for SessionLogger
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SessionLoggerMiddleware { service })
    }
}

pub struct SessionLoggerMiddleware<S> {
    service: S,
}

impl<S, B> actix_web::dev::Service<ServiceRequest> for SessionLoggerMiddleware<S>
where
    S: actix_web::dev::Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Log session information
        let session = req.get_session();
        info!("Session data:");
        for key in session.entries().keys() {
            if let Ok(value) = session.get::<Value>(key) {
                info!("  {}: {:?}", key, value);
            }
        }

        // Log cookies
        // Log all cookies in detail
        if let Ok(cookies) = req.cookies() {
            info!("All Cookies:");
            for cookie in cookies.iter() {
                info!("  Name: {}", cookie.name());
                info!("  Value: {}", cookie.value());
                info!("  HttpOnly: {}", cookie.http_only().unwrap_or(false));
                info!("  Secure: {}", cookie.secure().unwrap_or(false));
                info!("  SameSite: {:?}", cookie.same_site());
                info!("  Expires: {:?}", cookie.expires());
                info!("  Max-Age: {:?}", cookie.max_age());
                info!("  Domain: {:?}", cookie.domain());
                info!("  Path: {:?}", cookie.path());
            }
        }

        self.service.call(req)
    }
}
