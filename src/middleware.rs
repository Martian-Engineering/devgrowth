use actix_session::SessionExt;
use actix_web::body::{BoxBody, MessageBody};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::Method;
use actix_web::{Error, HttpResponse};
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
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
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
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().clone();

        // Only check authentication for POST, PUT, DELETE methods
        if method == Method::POST || method == Method::PUT || method == Method::DELETE {
            let authenticated = req
                .get_session()
                .get::<i32>("account_id")
                .map(|account_id| account_id.is_some())
                .unwrap_or(false);

            if !authenticated {
                let (http_req, _payload) = req.into_parts();
                let response = HttpResponse::Unauthorized().finish();
                return Box::pin(async move {
                    Ok(ServiceResponse::new(
                        http_req,
                        response.map_into_boxed_body(),
                    ))
                });
            }
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body())
        })
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
