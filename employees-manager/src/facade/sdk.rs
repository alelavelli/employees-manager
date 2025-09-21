use tracing::debug;

use crate::{
    auth::AuthInfo,
    dtos::{sdk_request, sdk_response},
    error::{AppError, ServiceAppError},
    service::{access_control::AccessControl, db::DatabaseDocument, user},
    DocumentId,
};

pub async fn get_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<sdk_response::User, AppError> {
    // access control over auth info
    debug!(
        "Making access control for auth_info with user {}",
        auth_info.user_id()
    );
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;
    let user_model = user::get_user(&user_id).await.map_err(|e| match e {
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })?;
    Ok(sdk_response::User {
        id: *user_model
            .get_id()
            .expect("field user_id should exist since the model comes from a db query"),
        username: user_model.username().clone(),
    })
}

pub async fn create_user(
    auth_info: impl AuthInfo,
    payload: sdk_request::CreateUser,
) -> Result<String, AppError> {
    // access control over auth info
    debug!(
        "Making access control for auth_info with user {}",
        auth_info.user_id()
    );
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;
    user::create_user(
        payload.username,
        payload.password,
        payload.email,
        payload.name,
        payload.surname,
    )
    .await
    .map_err(|e| match e {
        ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
        _ => AppError::InternalServerError(e.to_string()),
    })
}
