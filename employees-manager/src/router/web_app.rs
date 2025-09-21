use crate::{
    auth::JWTAuthClaim,
    dtos::{
        web_app_request,
        web_app_response::{self, AppNotification, CompanyInfo, UserInCompanyInfo},
        AppJson, ResponseWithHeader,
    },
    DocumentId,
};

use axum::{
    extract::{Path, Query},
    http::header,
    response::IntoResponse,
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
            "/notification/invite-add-company/{id}",
            patch(answer_to_invite_add_company),
        )
        .route("/notification/{id}/read", patch(set_notification_as_read))
        .route("/company", get(get_companies_of_user))
        .route("/company/{id}/user", get(get_users_in_company))
        .route("/company", post(create_company))
        .route("/company/{id}/role", patch(change_user_company_role))
        .route("/company/{id}/job-title", patch(change_user_job_title))
        .route("/company/{id}/manager", patch(change_user_company_manager))
        .route(
            "/company/{id}/user-to-invite",
            get(get_users_to_invite_in_company),
        )
        .route(
            "/company/{id}/pending-user",
            get(get_pending_invited_users_in_company),
        )
        .route(
            "/company/{id}/invite-user/{notification_id}",
            delete(cancel_invite_user_to_company),
        )
        .route("/company/{id}/invite-user", post(invite_user_to_company))
        .route("/company/{id}/user/{user_id}", delete(remove_company_user))
        .route("/company/{id}/project", get(get_company_projects))
        .route(
            "/company/{id}/project-allocation/{project_id}",
            get(get_company_project_allocations_by_project),
        )
        .route(
            "/company/{id}/user-allocation/{user_id}",
            get(get_company_project_allocations_by_user),
        )
        .route(
            "/company/{id}/project-allocation/{project_id}",
            patch(edit_company_project_allocations_by_project),
        )
        .route(
            "/company/{id}/user-allocation/{user_id}",
            patch(edit_company_project_allocations_by_user),
        )
        .route("/company/{id}/project", post(create_company_project))
        .route(
            "/company/{id}/project/{project_id}",
            patch(edit_company_project),
        )
        .route(
            "/company/{id}/project/{project_id}",
            delete(delete_company_project),
        )
        .route("/company/{id}/activity", post(create_project_activity))
        .route("/company/{id}/activity", get(get_project_activities))
        .route(
            "/company/{id}/activity/{activity_id}",
            patch(edit_project_activity),
        )
        .route(
            "/company/{id}/activity/{activity_id}",
            delete(delete_project_activity),
        )
        .route(
            "/company/{id}/activity-assignment/{activity_id}",
            get(get_project_activity_assignment_by_activity),
        )
        .route(
            "/company/{id}/project-activity/{project_id}",
            get(get_project_activity_assignment_by_project),
        )
        .route(
            "/company/{id}/activity-assignment/{activity_id}",
            patch(edit_project_activity_assignment_by_activity),
        )
        .route(
            "/company/{id}/project-activity/{project_id}",
            patch(edit_project_activity_assignment_by_project),
        )
        .route(
            "/user/{id}/timesheet-project",
            get(get_user_projects_for_timesheet),
        )
        .route("/user/{id}/timesheet-day", post(create_timesheet_day))
        .route("/user/{id}/timesheet-day", get(get_timesheet_days))
        .route(
            "/corporate-group/eligible-company",
            get(get_eligible_companies_for_corporate_group),
        )
        .route("/corporate-group", get(get_user_corporate_groups))
        .route("/corporate-group", post(create_corporate_group))
        .route("/corporate-group/{id}", delete(delete_corporate_group))
        .route("/corporate-group/{id}", patch(edit_corporate_group))
        .route("/user/timesheet-export", get(export_personal_timesheet))
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

async fn set_notification_as_read(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<()>, AppError> {
    facade::set_notification_as_read(jwt_claim, id).await?;
    Ok(AppJson(()))
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

async fn get_users_to_invite_in_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<web_app_response::UserToInviteInCompany>>, AppError> {
    facade::get_users_to_invite_in_company(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn get_pending_invited_users_in_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<web_app_response::InvitedUserInCompanyInfo>>, AppError> {
    facade::get_pending_invited_users_in_company(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn cancel_invite_user_to_company(
    jwt_claim: JWTAuthClaim,
    Path((id, notification_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<()>, AppError> {
    facade::cancel_invite_user_to_company(jwt_claim, notification_id, id)
        .await
        .map(AppJson)
}

async fn get_company_projects(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<web_app_response::CompanyProjectInfo>>, AppError> {
    facade::get_company_projects(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn get_company_project_allocations_by_project(
    jwt_claim: JWTAuthClaim,
    Path((id, project_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<Vec<String>>, AppError> {
    facade::get_company_project_allocations_by_project(jwt_claim, id, project_id)
        .await
        .map(AppJson)
}

async fn get_company_project_allocations_by_user(
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<Vec<String>>, AppError> {
    facade::get_company_project_allocations_by_user(jwt_claim, id, user_id)
        .await
        .map(AppJson)
}

async fn create_company_project(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::CreateCompanyProject>,
) -> Result<AppJson<()>, AppError> {
    facade::create_company_project(jwt_claim, id, payload)
        .await
        .map(AppJson)
}

async fn edit_company_project(
    jwt_claim: JWTAuthClaim,
    Path((id, project_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::EditCompanyProject>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_company_project(jwt_claim, id, project_id, payload)
        .await
        .map(AppJson)
}

async fn delete_company_project(
    jwt_claim: JWTAuthClaim,
    Path((id, project_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<()>, AppError> {
    facade::delete_company_project(jwt_claim, id, project_id)
        .await
        .map(AppJson)
}

async fn edit_company_project_allocations_by_project(
    jwt_claim: JWTAuthClaim,
    Path((id, project_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::ChangeProjectAllocations>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_company_project_allocations_by_project(jwt_claim, id, project_id, payload)
        .await
        .map(AppJson)
}

async fn edit_company_project_allocations_by_user(
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::ChangeProjectAllocationsForUser>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_company_project_allocations_by_user(jwt_claim, id, user_id, payload)
        .await
        .map(AppJson)
}

async fn create_project_activity(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::NewProjectActivity>,
) -> Result<AppJson<()>, AppError> {
    facade::create_project_activity(jwt_claim, id, payload)
        .await
        .map(AppJson)
}

async fn get_project_activities(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<web_app_response::ProjectActivityInfo>>, AppError> {
    facade::get_project_activities(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn edit_project_activity(
    jwt_claim: JWTAuthClaim,
    Path((id, activity_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::EditProjectActivity>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_project_activity(jwt_claim, id, activity_id, payload)
        .await
        .map(AppJson)
}

async fn delete_project_activity(
    jwt_claim: JWTAuthClaim,
    Path((id, activity_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<()>, AppError> {
    facade::delete_project_activity(jwt_claim, id, activity_id)
        .await
        .map(AppJson)
}

async fn get_project_activity_assignment_by_activity(
    jwt_claim: JWTAuthClaim,
    Path((id, activity_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<Vec<String>>, AppError> {
    facade::get_project_activity_assignment_by_activity(jwt_claim, id, activity_id)
        .await
        .map(AppJson)
}

async fn get_project_activity_assignment_by_project(
    jwt_claim: JWTAuthClaim,
    Path((id, project_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<Vec<String>>, AppError> {
    facade::get_project_activity_assignment_by_project(jwt_claim, id, project_id)
        .await
        .map(AppJson)
}

async fn edit_project_activity_assignment_by_activity(
    jwt_claim: JWTAuthClaim,
    Path((id, activity_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::ChangeProjectActivityAssignmentByActivity>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_project_activity_assignment_by_activity(
        jwt_claim,
        id,
        activity_id,
        payload.project_ids,
    )
    .await
    .map(AppJson)
}

async fn edit_project_activity_assignment_by_project(
    jwt_claim: JWTAuthClaim,
    Path((id, project_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::ChangeProjectActivityAssignmentByProject>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_project_activity_assignment_by_project(
        jwt_claim,
        id,
        project_id,
        payload.activity_ids,
    )
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

async fn get_eligible_companies_for_corporate_group(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<web_app_response::CorporateGroupCompanyInfo>>, AppError> {
    facade::get_eligible_companies_for_corporate_group(jwt_claim)
        .await
        .map(AppJson)
}

async fn get_user_corporate_groups(
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<web_app_response::CorporateGroupInfo>>, AppError> {
    facade::get_user_corporate_groups(jwt_claim)
        .await
        .map(AppJson)
}

async fn create_corporate_group(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::CreateCorporateGroup>,
) -> Result<AppJson<()>, AppError> {
    facade::create_corporate_group(jwt_claim, payload)
        .await
        .map(AppJson)
}

async fn delete_corporate_group(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<()>, AppError> {
    facade::delete_corporate_group(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn edit_corporate_group(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::EditCorporateGroup>,
) -> Result<AppJson<()>, AppError> {
    facade::edit_corporate_group(jwt_claim, id, payload)
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
