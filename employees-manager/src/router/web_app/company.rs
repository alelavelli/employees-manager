use std::sync::Arc;

use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use once_cell::sync::Lazy;

use crate::{
    auth::JWTAuthClaim,
    dtos::{
        web_app_request,
        web_app_response::{self, CompanyInfo, UserInCompanyInfo},
        AppJson,
    },
    error::AppError,
    facade::{user::UserFacade, web_app as facade},
    AppState, DocumentId, APP_STATE,
};

pub static COMPANY_ROUTER: Lazy<Router<Arc<AppState>>> = Lazy::new(|| {
    Router::new()
        .route("/company", get(get_companies_of_user))
        .route("/company/{id}/user", get(get_users_in_company))
        .route("/company", post(create_company))
        .route("/company/{id}/job-title", patch(change_user_job_title))
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
});

async fn get_companies_of_user(
    State(state): State<Arc<AppState>>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<CompanyInfo>>, AppError> {
    APP_STATE
        .scope(state, async {
            UserFacade::new(
                crate::service::db::document::SmartDocumentReference::Id(jwt_claim.user_id),
                jwt_claim,
            )
            .await?
            .get_companies()
            .await
            .map(AppJson)
        })
        .await
}

async fn get_users_in_company(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<Vec<UserInCompanyInfo>>, AppError> {
    facade::get_users_in_company(jwt_claim, id)
        .await
        .map(AppJson)
}

async fn change_user_job_title(
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::ChangeUserJobTitle>,
) -> Result<(), AppError> {
    facade::change_user_company_job_title(jwt_claim, id, payload).await
}

/// Create a Company in the portal becoming the owner
/// POST /company
async fn create_company(
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::CreateCompany>,
) -> Result<AppJson<()>, AppError> {
    facade::create_company(jwt_claim, payload).await?;
    Ok(AppJson(()))
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
