use std::sync::Arc;

use crate::{
    auth::{AuthInfo, JWTAuthClaim},
    dtos::{
        web_app_request,
        web_app_response::{self, AppNotification},
        AppJson, ResponseWithHeader,
    },
    facade::{guest::GuestFacade, user::UserFacade},
    //router::web_app::corporate_group::CORPORATE_GROUP_ROUTER,
    service::db::transaction::DatabaseTransaction,
    AppState,
    DocumentId,
    APP_STATE,
    TRANSACTION,
};

use axum::{
    extract::{Path, Query, State},
    http::header,
    response::IntoResponse,
    routing::{get, patch, post},
    Extension, Json, Router,
};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::error::AppError;
// use crate::facade::web_app as facade;

//mod company;
//mod corporate_group;

pub static WEB_APP_ROUTER: Lazy<Router<AppState>> = Lazy::new(|| {
    Router::new()
        //.nest("/corporate-group", CORPORATE_GROUP_ROUTER.to_owned())
        //.nest("/company", COMPANY_ROUTER.to_owned())
        .route("/auth/login", post(authorize))
        .route("/auth/user", get(get_auth_user_data))
    /*
    .route("/notification", get(get_unread_notifications))
    .route(
        "/notification/invite-add-company/{id}",
        patch(answer_to_invite_add_company),
    )
    .route("/notification/{id}/read", patch(set_notification_as_read))
    .route(
        "/user/{id}/timesheet-project",
        get(get_user_projects_for_timesheet),
    )
    .route("/user/{id}/timesheet-day", post(create_timesheet_day))
    .route("/user/{id}/timesheet-day", get(get_timesheet_days))
    .route("/user/timesheet-export", get(export_personal_timesheet))
     */
});

/// Authorize a user with username and password providing jwt token
async fn authorize(
    State(state): State<AppState>,
    Json(payload): Json<web_app_request::JWTAuthPayload>,
) -> Result<AppJson<web_app_response::JWTAuthResponse>, AppError> {
    APP_STATE
        .scope(state, async {
            GuestFacade::authenticate_user(&payload.username, &payload.password)
                .await
                .map(AppJson)
        })
        .await
}

/// Get user data from jwt token
async fn get_auth_user_data(
    State(state): State<AppState>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<web_app_response::AuthUserData>, AppError> {
    APP_STATE
        .scope(state, async {
            let user = UserFacade::new(
                crate::service::db::document::SmartDocumentReference::Id(*jwt_claim.user_id()),
                jwt_claim,
            )
            .await?
            .get_user_data()
            .await?;
            Ok(AppJson(user))
        })
        .await
}

/*
async fn get_unread_notifications(
    State(state): State<Arc<AppState>>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<AppNotification>>, AppError> {
    APP_STATE
        .scope(state, async {
            let notifications = facade::get_unread_notifications(jwt_claim).await?;
            Ok(AppJson(notifications))
        })
        .await
}

async fn set_notification_as_read(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    facade::set_notification_as_read(jwt_claim, id).await?;
                    Ok(AppJson(()))
                })
                .await
        })
        .await
}

async fn answer_to_invite_add_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::InviteAddCompanyAnswer>,
) -> Result<AppJson<()>, AppError> {
    facade::answer_to_invite_add_company(jwt_claim, id, payload)
        .await
        .map(AppJson)
}

async fn create_timesheet_day(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::CreateTimesheetDay>,
) -> Result<AppJson<()>, AppError> {
    facade::create_timesheet_day(jwt_claim, id, payload)
        .await
        .map(AppJson)
}

async fn get_timesheet_days(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    query: Query<web_app_request::GetUserTimesheetDays>,
) -> Result<AppJson<Vec<web_app_response::TimesheetDay>>, AppError> {
    facade::get_timesheet_days(jwt_claim, id, query.year, query.month)
        .await
        .map(AppJson)
}

async fn get_user_projects_for_timesheet(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<web_app_response::TimesheetProjectInfo>>, AppError> {
    facade::get_user_projects_for_timesheet(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn export_personal_timesheet(
    jwt_claim: JWTAuthClaim,
    query: Query<web_app_request::GetUserTimesheetExport>,
) -> Result<impl IntoResponse, AppError> {
    facade::export_personal_timesheet(jwt_claim, query.year, query.month)
        .await
        .map(|content| {
            ResponseWithHeader::new(content).with_header(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str(
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                )
                .unwrap(),
            )
        })
}
 */
