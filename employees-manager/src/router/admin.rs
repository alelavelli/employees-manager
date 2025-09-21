use crate::{
    auth::JWTAuthClaim,
    dtos::{
        web_app_request,
        web_app_response::{self, AdminPanelOverview},
        AppJson,
    },
    DocumentId,
};

use axum::{
    extract::Path,
    routing::{delete, get, patch, post},
    Json, Router,
};
use once_cell::sync::Lazy;

use crate::error::AppError;
use crate::facade::admin as facade;

pub static ADMIN_ROUTER: Lazy<Router> = Lazy::new(|| {
    Router::new()
        .route("/overview", get(get_admin_panel_overview))
        .route("/user", get(get_admin_panel_users_info))
        .route("/user", post(create_user))
        .route("/user/{id}/platform-admin", post(set_platform_admin))
        .route("/user/{id}/platform-admin", delete(unset_platform_admin))
        .route("/user/{id}/activate", post(activate_platform_admin))
        .route("/user/{id}/activate", delete(deactivate_platform_admin))
        .route("/user/{id}", delete(delete_user))
        .route("/user/{id}", get(get_user))
        .route("/user/{id}/password", patch(set_user_password))
});

/// Returns overview of all users and companies in application
async fn get_admin_panel_overview(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<AdminPanelOverview>, AppError> {
    let overview = facade::get_admin_panel_overview(jwt_claim).await?;
    Ok(AppJson(overview))
}

/// Returns all users in the application
async fn get_admin_panel_users_info(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<web_app_response::AdminPanelUserInfo>>, AppError> {
    let users = facade::get_admin_panel_users_info(jwt_claim).await?;
    Ok(AppJson(users))
}

/// Create new user providing required attributes
async fn create_user(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::CreateUser>,
) -> Result<AppJson<String>, AppError> {
    let user = facade::create_user(jwt_claim, payload).await?;
    Ok(AppJson(user))
}

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

/// Delete user from the application
async fn delete_user(jwt_claim: JWTAuthClaim, Path(id): Path<DocumentId>) -> Result<(), AppError> {
    facade::delete_user(jwt_claim, id).await
}

async fn set_platform_admin(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    facade::set_platform_admin(jwt_claim, id).await
}

async fn unset_platform_admin(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    facade::unset_platform_admin(jwt_claim, id).await
}

async fn activate_platform_admin(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    facade::activate_platform_admin(jwt_claim, id).await
}

async fn deactivate_platform_admin(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    facade::deactivate_platform_admin(jwt_claim, id).await
}

async fn set_user_password(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::SetUserPassword>,
) -> Result<(), AppError> {
    facade::set_user_password(jwt_claim, id, payload.password).await
}
