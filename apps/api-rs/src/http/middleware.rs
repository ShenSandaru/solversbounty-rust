use std::cell::RefCell;

use axum::extract::Request;
use axum::http::{header::HeaderName, HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;
use tracing::info;
use uuid::Uuid;

thread_local! {
    static REQUEST_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn current_request_id() -> String {
    REQUEST_ID.with(|id| id.borrow().clone().unwrap_or_else(|| "unknown".to_string()))
}

pub async fn request_id(mut request: Request, next: Next) -> Response {
    let id = Uuid::new_v4().to_string();
    request
        .headers_mut()
        .insert("x-request-id", HeaderValue::from_str(&id).unwrap());
    REQUEST_ID.with(|slot| *slot.borrow_mut() = Some(id.clone()));
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    info!(%id, %method, path, status, "request");
    REQUEST_ID.with(|slot| *slot.borrow_mut() = None);
    response
}

pub fn cors_layer(origins: &[String]) -> tower_http::cors::CorsLayer {
    use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
    use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};

    let allowed: Vec<HeaderValue> = origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_credentials(true)
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            CONTENT_TYPE,
            HeaderName::from_static("x-csrf-token"),
            ACCEPT,
            AUTHORIZATION,
        ]))
        .max_age(std::time::Duration::from_secs(3600))
}
