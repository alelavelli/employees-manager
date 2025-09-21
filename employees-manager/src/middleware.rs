//! Middleware module contains functions to add middlewares to a generic Router.
//!
//! All the functions receive a `Router` object and return it adding a new `layer`.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Method, Request},
    middleware::Next,
    response::Response,
    Router,
};
use tokio::sync::RwLock;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::debug;

use crate::{service::db::transaction::DatabaseTransaction, AppState};

/// Create CorsLayer for application
///
/// This simple version allow everything but it can
/// be modified restricting it
pub fn add_cors_middleware(router: Router<AppState>) -> Router<AppState> {
    router.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    )
}

/// Create Logging middleware for application
pub fn add_logging_middleware(
    router: Router<AppState>,
    include_headers: bool,
    logging_level: tracing::Level,
) -> Router<AppState> {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(include_headers))
            .on_request(DefaultOnRequest::new().level(logging_level))
            .on_response(
                DefaultOnResponse::new()
                    .level(logging_level)
                    .latency_unit(LatencyUnit::Micros),
            ),
    )
}

/// Creates a mongodb transaction if the request is not a GET
/// and put it in the request extensions.
///
/// If the request is success then the transaction is committed
/// otherwise it is aborted
async fn mongodb_transaction_middleware(
    State(app_state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, axum::http::StatusCode>
where
{
    let method = request.method().clone();
    debug!("Method is {method}", method = method);
    if matches!(
        method,
        Method::POST | Method::PATCH | Method::DELETE | Method::PUT
    ) {
        let db_service = app_state.database_service.clone();
        let transaction = Arc::new(RwLock::new(DatabaseTransaction::new(
            db_service
                .get_client()
                .start_session()
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?,
        )));
        let result = transaction.write().await.start_transaction().await;
        if result.is_err() {
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }

        request.extensions_mut().insert(Arc::clone(&transaction));
        debug!("forward to next layer");
        let response = next.run(request).await;
        debug!("got response");
        let mut guard = transaction.write().await;
        debug!("got guard");
        if response.status().is_success() {
            debug!(
                "Response status {status}, committing transaction",
                status = response.status()
            );
            let _ = guard
                .commit_transaction()
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        } else {
            debug!(
                "Response status {status}, aborting transaction",
                status = response.status()
            );
            let _ = guard
                .abort_transaction()
                .await
                .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        }
        Ok(response)
    } else {
        let response = next.run(request).await;
        debug!("{}", format!("Got response {:?}", response));
        Ok(response)
    }
}

pub fn add_mongodb_transaction_middleware(
    state: AppState,
    router: Router<AppState>,
) -> Router<AppState> {
    router.layer(axum::middleware::from_fn_with_state(
        state,
        mongodb_transaction_middleware,
    ))
}
