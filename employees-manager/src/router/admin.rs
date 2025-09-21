use std::sync::Arc;

use crate::{
    auth::JWTAuthClaim,
    dtos::{web_app_request, web_app_response, AppJson},
    service::db::{document::SmartDocumentReference, transaction::DatabaseTransaction},
    AppState, DocumentId, APP_STATE, TRANSACTION,
};

use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::facade::admin::AdminFacade;

pub static ADMIN_ROUTER: Lazy<Router<AppState>> = Lazy::new(|| {
    Router::new()
        .route("/overview", get(get_admin_panel_overview))
        .route("/user", get(get_admin_panel_users_info))
        .route(
            "/corporate-group",
            get(get_admin_panel_corporate_groups_info),
        )
        .route("/corporate-group", post(create_corporate_group))
        .route(
            "/corporate-group/{id}/owner/{user_id}",
            post(set_corporate_group_owner),
        )
        .route("/user", post(create_user))
        .route("/user/{id}/platform-admin", post(set_platform_admin))
        .route("/user/{id}/platform-admin", delete(unset_platform_admin))
        .route("/user/{id}/activate", post(activate_user))
        .route("/user/{id}/activate", delete(deactivate_user))
        .route("/user/{id}", delete(delete_user))
        .route("/user/{id}", get(get_user))
        .route("/user/{id}/password", patch(set_user_password))
});

async fn create_corporate_group(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::corporate_group::CreateCorporateGroup>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .create_corporate_group(payload.name)
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}

/// Returns overview of all users and companies in application
async fn get_admin_panel_overview(
    State(state): State<AppState>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<web_app_response::admin_panel::AdminPanelOverview>, AppError> {
    APP_STATE
        .scope(state, async {
            let overview = AdminFacade::new(jwt_claim)
                .await?
                .get_admin_panel_overview()
                .await?;
            Ok(AppJson(overview))
        })
        .await
}

/// Returns all users in the application
async fn get_admin_panel_users_info(
    State(state): State<AppState>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<web_app_response::admin_panel::AdminPanelUserInfo>>, AppError> {
    APP_STATE
        .scope(state, async {
            let users = AdminFacade::new(jwt_claim)
                .await?
                .get_admin_panel_users_info()
                .await?;
            Ok(AppJson(users))
        })
        .await
}

/// Returns the list of corporate groups in the application
async fn get_admin_panel_corporate_groups_info(
    State(state): State<AppState>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<web_app_response::admin_panel::AdminPanelCorporateGroupInfo>>, AppError> {
    APP_STATE
        .scope(state, async {
            let result = AdminFacade::new(jwt_claim)
                .await?
                .get_admin_panel_corporate_groups_info()
                .await?;
            Ok(AppJson(result))
        })
        .await
}

/// Set the user as corporate group owner
///
/// If the user is not in the corporate group then it is added.
/// IF the corporate group already has an owner he is changed to admin.
async fn set_corporate_group_owner(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .set_corporate_group_owner(
                            SmartDocumentReference::from(id),
                            SmartDocumentReference::from(user_id),
                        )
                        .await?;
                    Ok(AppJson(()))
                })
                .await
        })
        .await
}

/// Create new user providing required attributes
async fn create_user(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Json(payload): Json<web_app_request::CreateUser>,
) -> Result<AppJson<String>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    let user = AdminFacade::new(jwt_claim)
                        .await?
                        .create_user(payload)
                        .await?;
                    Ok(AppJson(user))
                })
                .await
        })
        .await
}

/// Returns the user if it exists with all the information
///
/// Request parameter is extracted from the url
async fn get_user(
    State(state): State<AppState>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<web_app_response::User>, AppError> {
    APP_STATE
        .scope(state, async {
            let user = AdminFacade::new(jwt_claim).await?.get_user(id).await?;
            Ok(AppJson(user))
        })
        .await
}

/// Delete user from the application
async fn delete_user(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim).await?.delete_user(id).await
                })
                .await
        })
        .await
}

async fn set_platform_admin(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .set_platform_admin(id)
                        .await
                })
                .await
        })
        .await
}

async fn unset_platform_admin(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .unset_platform_admin(id)
                        .await
                })
                .await
        })
        .await
}

async fn activate_user(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim).await?.activate_user(id).await
                })
                .await
        })
        .await
}

async fn deactivate_user(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<(), AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim).await?.deactivate_user(id).await
                })
                .await
        })
        .await
}

async fn set_user_password(
    State(state): State<AppState>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::SetUserPassword>,
) -> Result<(), AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .set_user_password(id, payload.password)
                        .await
                })
                .await
        })
        .await
}
