use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{delete, get, patch, post};
use axum::{Extension, Json, Router};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::dtos::web_app_request::corporate_group as web_app_request;
use crate::error::AppError;
use crate::facade::admin::AdminFacade;
use crate::facade::corporate_group_admin::CorporateGroupAdminFacade;
use crate::facade::corporate_group_user::CorporateGroupUserFacade;
use crate::facade::user::UserFacade;
use crate::service::db::document::SmartDocumentReference;
use crate::service::db::transaction::DatabaseTransaction;
use crate::{
    auth::JWTAuthClaim,
    dtos::{web_app_response::corporate_group as web_app_response, AppJson},
};
use crate::{AppState, DocumentId, APP_STATE, TRANSACTION};

pub static CORPORATE_GROUP_ROUTER: Lazy<Router<AppState>> = Lazy::new(|| {
    Router::new()
        .route("/{id}", get(get_corporate_group_info))
        .route("/", get(get_user_corporate_groups))
        .route("/{id}", delete(delete_corporate_group))
        .route("/{id}", patch(update_corporate_group))
        .route("/{id}/activate", post(activate_corporate_group))
        .route("/{id}/deactivate", post(deactivate_corporate_group))
        .route("/{id}/user/{user_id}", post(add_user_to_corporate_group))
        .route(
            "/{id}/user/{user_id}",
            delete(remove_user_from_corporate_group),
        )
        .route(
            "/{id}/user/{user_id}",
            patch(update_user_in_corporate_group),
        )
});

async fn get_corporate_group_info(
    State(state): State<Arc<AppState>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<web_app_response::CorporateGroupInfo>, AppError> {
    APP_STATE
        .scope(state, async {
            CorporateGroupUserFacade::new(SmartDocumentReference::Id(id), jwt_claim)
                .await?
                .get_corporate_group_info()
                .await
                .map(AppJson)
        })
        .await
}

async fn get_user_corporate_groups(
    State(state): State<Arc<AppState>>,
    jwt_claim: JWTAuthClaim,
) -> Result<AppJson<Vec<web_app_response::CorporateGroupInfo>>, AppError> {
    APP_STATE
        .scope(state, async {
            UserFacade::new(SmartDocumentReference::from(jwt_claim.user_id), jwt_claim)
                .await?
                .get_corporate_groups()
                .await
                .map(AppJson)
        })
        .await
}

async fn delete_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    CorporateGroupAdminFacade::new(SmartDocumentReference::Id(id), jwt_claim)
                        .await?
                        .delete()
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}

async fn update_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
    Json(payload): Json<web_app_request::EditCorporateGroup>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    CorporateGroupAdminFacade::new(SmartDocumentReference::Id(id), jwt_claim)
                        .await?
                        .update(payload)
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}

async fn add_user_to_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::AddUserToCorporateGroup>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    CorporateGroupAdminFacade::new(SmartDocumentReference::Id(id), jwt_claim)
                        .await?
                        .add_user(
                            SmartDocumentReference::Id(user_id),
                            payload.role,
                            payload.employment_contract,
                        )
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}

async fn remove_user_from_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    CorporateGroupAdminFacade::new(SmartDocumentReference::Id(id), jwt_claim)
                        .await?
                        .remove_user(SmartDocumentReference::from(user_id))
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}
async fn update_user_in_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path((id, user_id)): Path<(DocumentId, DocumentId)>,
    Json(payload): Json<web_app_request::UpdateUserInCorporateGroup>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    CorporateGroupAdminFacade::new(SmartDocumentReference::Id(id), jwt_claim)
                        .await?
                        .update_user(
                            SmartDocumentReference::from(user_id),
                            payload.role,
                            payload.employment_contract,
                        )
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}

async fn activate_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .activate_corporate_group(SmartDocumentReference::Id(id))
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}

async fn deactivate_corporate_group(
    State(state): State<Arc<AppState>>,
    Extension(transaction): Extension<Arc<RwLock<DatabaseTransaction>>>,
    jwt_claim: JWTAuthClaim,
    Path(id): Path<DocumentId>,
) -> Result<AppJson<()>, AppError> {
    APP_STATE
        .scope(state, async {
            TRANSACTION
                .scope(transaction, async {
                    AdminFacade::new(jwt_claim)
                        .await?
                        .deactivate_corporate_group(SmartDocumentReference::Id(id))
                        .await
                        .map(AppJson)
                })
                .await
        })
        .await
}
