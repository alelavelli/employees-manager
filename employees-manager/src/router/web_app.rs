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
use crate::facade::web_app as facade;

pub static WEB_APP_ROUTER: Lazy<Router> = Lazy::new(|| {
    Router::new()
        .route("/auth/login", post(authorize))
        .route("/auth/user", get(get_auth_user_data))
        .route("/company", post(create_company))
        .route("/company/:id", get(get_company))
        .route("/company/user", post(add_company_user))
        .route("/company/user", delete(remove_company_user))
});

/// Authorize a user with username and password providing jwt token
async fn authorize(
    Json(payload): Json<web_app_request::JWTAuthPayload>,
) -> Result<AppJson<web_app_response::JWTAuthResponse>, AppError> {
    facade::authenticate_user(&payload.username, &payload.password)
        .await
        .map(AppJson)
}

/// Get user data from jwt token
async fn get_auth_user_data(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<web_app_response::AuthUserData>, AppError> {
    let user = facade::get_auth_user_data(jwt_claim).await?;
    Ok(AppJson(user))
}

/// Create a Company in the portal becoming the owner
/// POST /company
async fn create_company(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::CreateCompany>,
) -> Result<AppJson<String>, AppError> {
    let company = facade::create_company(jwt_claim, payload).await?;
    Ok(AppJson(company))
}

/// Get a Company the user belong with
/// GET /company/:id
async fn get_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<web_app_response::Company>, AppError> {
    let company = facade::get_company(jwt_claim, id).await?;
    Ok(AppJson(company))
}

/// Add user to Company
/// POST /company/user
async fn add_company_user(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::AddCompanyUser>,
) -> Result<AppJson<()>, AppError> {
    facade::add_company_user(
        jwt_claim,
        payload.user_id,
        payload.company_id,
        payload.role,
        payload.job_title,
    )
    .await?;
    Ok(AppJson(()))
}

/// Remove user from the Company
/// DELETE /company/user/:id
async fn remove_company_user(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::RemoveCompanyUser>,
) -> Result<AppJson<()>, AppError> {
    facade::remove_company_user(jwt_claim, payload.user_id, payload.company_id).await?;
    Ok(AppJson(()))
}
