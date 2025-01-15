use crate::{
    auth::JWTAuthClaim,
    dtos::{
        web_app_request,
        web_app_response::{self, AppNotification, CompanyInfo, UserInCompanyInfo},
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
use crate::facade::web_app as facade;

pub static WEB_APP_ROUTER: Lazy<Router> = Lazy::new(|| {
    Router::new()
        .route("/auth/login", post(authorize))
        .route("/auth/user", get(get_auth_user_data))
        .route("/notification", get(get_unread_notifications))
        .route(
            "/notification/invite-add-company",
            patch(answer_to_invite_add_company),
        )
        .route("/company", get(get_companies_of_user))
        .route("/company/{id}/user", get(get_users_in_company))
        .route("/company", post(create_company))
        .route("/company/{id}/role", patch(change_user_company_role))
        .route("/company/{id}/job-title", patch(change_user_job_title))
        .route("/company/{id}/manager", patch(change_user_company_manager))
        .route("/company/{id}/invite-user", post(invite_user_to_company))
        .route("/company/{id}/user/{user_id}", delete(remove_company_user))
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

async fn get_unread_notifications(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<AppNotification>>, AppError> {
    let notifications = facade::get_unread_notifications(jwt_claim).await?;
    Ok(AppJson(notifications))
}
async fn answer_to_invite_add_company(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::InviteAddCompanyAnswer>,
) -> Result<AppJson<()>, AppError> {
    facade::answer_to_invite_add_company(jwt_claim, payload)
        .await
        .map(AppJson)
}

async fn get_companies_of_user(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<CompanyInfo>>, AppError> {
    facade::get_companies_of_user(jwt_claim).await.map(AppJson)
}

async fn get_users_in_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<UserInCompanyInfo>>, AppError> {
    facade::get_users_in_company(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn change_user_company_role(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::ChangeUserCompanyRole>,
) -> Result<(), AppError> {
    facade::change_user_company_role(jwt_claim, id, payload).await
}

async fn change_user_job_title(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::ChangeUserJobTitle>,
) -> Result<(), AppError> {
    facade::change_user_company_job_title(jwt_claim, id, payload).await
}

async fn change_user_company_manager(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::ChangeUserCompanyManager>,
) -> Result<(), AppError> {
    facade::change_user_company_manager(jwt_claim, id, payload).await
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

async fn invite_user_to_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::InviteUserToCompany>,
) -> Result<(), AppError> {
    facade::invite_user_to_company(jwt_claim, id, payload).await
}

/// Remove user from the Company
async fn remove_company_user(
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<()>, AppError> {
    facade::remove_company_user(jwt_claim, user_id, id).await?;
    Ok(AppJson(()))
}
