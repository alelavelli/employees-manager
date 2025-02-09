//! Middleware module contains functions to add middlewares to a generic Router.
//!
//! All the functions receive a `Router` object and return it adding a new `layer`.

use axum::Router;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

use crate::service::environment::ENVIRONMENT;

/// Create CorsLayer for application
///
/// This simple version allow everything but it can
/// be modified restricting it
pub fn add_cors_middleware(router: Router) -> Router {
    router.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    )
}

/// Create Logging middleware for application
pub fn add_logging_middleware(router: Router) -> Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(
                DefaultMakeSpan::new().include_headers(ENVIRONMENT.logging.include_headers),
            )
            .on_request(DefaultOnRequest::new().level(ENVIRONMENT.logging.level))
            .on_response(
                DefaultOnResponse::new()
                    .level(ENVIRONMENT.logging.level)
                    .latency_unit(LatencyUnit::Micros),
            ),
    )
}
