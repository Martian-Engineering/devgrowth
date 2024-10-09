use crate::account::upsert_account;
use crate::auth::{validate_token, Claims};
use actix_session::SessionExt;
use actix_web::http::header;
use actix_web::http::header::HeaderValue;
use actix_web::web;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    Error, HttpMessage,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use log::info;
use serde_json::Value;
use sqlx::postgres::PgPool;
use std::cell::RefCell;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::rc::Rc;

#[derive(Clone)]
pub struct AuthMiddleware {
    pool: web::Data<PgPool>,
}

impl AuthMiddleware {
    pub fn new(pool: web::Data<PgPool>) -> Self {
        Self { pool }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(RefCell::new(service)),
            pool: self.pool.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<RefCell<S>>,
    pool: web::Data<PgPool>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let pool = self.pool.clone();

        Box::pin(async move {
            let token = match Self::extract_token(&req) {
                Ok(token) => token,
                Err(e) => return Err(e),
            };

            let claims = match validate_token(&token) {
                Ok(claims) => claims,
                Err(e) => {
                    log::error!("Token validation error: {:?}", e);
                    return Err(ErrorUnauthorized("Invalid token"));
                }
            };

            let (claims, new_token) = Self::handle_user_creation(claims, &pool).await?;
            req.extensions_mut().insert(claims);

            let mut res = srv.borrow_mut().call(req).await?;

            if let Some(token) = new_token {
                res.headers_mut().insert(
                    header::AUTHORIZATION,
                    HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
                );
            }

            Ok(res)
        })
    }
}

impl<S> AuthMiddlewareService<S> {
    fn extract_token(req: &ServiceRequest) -> Result<String, Error> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .ok_or_else(|| ErrorUnauthorized("Missing Authorization header"))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| ErrorUnauthorized("Invalid Authorization header"))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(ErrorUnauthorized("Invalid Authorization header format"));
        }

        Ok(auth_str[7..].to_string())
    }

    async fn handle_user_creation(
        mut claims: Claims,
        pool: &web::Data<PgPool>,
    ) -> Result<(Claims, Option<String>), Error> {
        let mut new_token = None;
        if claims.db_id.is_none() {
            claims = match upsert_account(pool, &claims.id.to_string(), Some(&claims.email)).await {
                Ok(db_id) => {
                    info!("User created/updated with db_id: {}", db_id);
                    Claims {
                        db_id: Some(db_id),
                        ..claims
                    }
                }
                Err(e) => {
                    log::error!("Failed to create/update user: {:?}", e);
                    return Err(ErrorInternalServerError("User creation failed"));
                }
            };

            // Generate new token with db_id
            new_token = Some(
                encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(std::env::var("JWT_SECRET").unwrap().as_bytes()),
                )
                .map_err(|e| {
                    log::error!("Failed to generate new token: {:?}", e);
                    ErrorInternalServerError("Token generation failed")
                })?,
            );
        }

        Ok((claims, new_token))
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
        ready(Ok(SessionLoggerMiddleware { service }))
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
