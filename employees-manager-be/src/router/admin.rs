use crate::{
    auth::JWTAuthClaim,
    dtos::{web_app_request, web_app_response, AppJson},
    DocumentId,
};

use axum::{
    extract::Path,
    routing::{delete, get, post},
    Json, Router,
};
use once_cell::sync::Lazy;

use crate::error::AppError;
use crate::facade::admin as facade;

pub static ADMIN_ROUTER: Lazy<Router> = Lazy::new(|| {
    Router::new()
        .route("/user/:id", get(get_user))
        .route("/user", post(create_user))
        .route("/user/:id", delete(delete_user))
});

/// Returns the user if it exists with all the information
///
/// Request parameter is extracted from the url
async fn get_user(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<web_app_response::User>, AppError> {
    let user = facade::get_user(jwt_claim, id).await?;
    Ok(AppJson(user))
}

/// Create new user providing required attributes
async fn create_user(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::CreateUser>,
) -> Result<AppJson<String>, AppError> {
    let user = facade::create_user(jwt_claim, payload).await?;
    Ok(AppJson(user))
}

/// Delete user from the application
async fn delete_user(jwt_claim: JWTAuthClaim, Path(id): Path<DocumentId>) -> Result<(), AppError> {
    facade::delete_user(jwt_claim, id).await
}
