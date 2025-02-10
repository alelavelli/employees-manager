use crate::{
    auth::AuthInfo,
    dtos::{
        web_app_request,
        web_app_response::{self},
    },
    error::AppError,
    service::{access_control::AccessControl, company, user},
    DocumentId,
};

pub async fn get_admin_panel_overview(
    auth_info: impl AuthInfo,
) -> Result<web_app_response::AdminPanelOverview, AppError> {
    AccessControl::new(auth_info)
        .await?
        .is_platform_admin()
        .await?;

    let users_info = user::get_admin_panel_overview_users_info().await?;

    let companies_info = company::get_admin_panel_overview_companies_info().await?;

    Ok(web_app_response::AdminPanelOverview::from((
        users_info,
        companies_info,
    )))
}

pub async fn get_admin_panel_users_info(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::AdminPanelUserInfo>, AppError> {
    AccessControl::new(auth_info)
        .await?
        .is_platform_admin()
        .await?;

    user::get_admin_panel_users_info()
        .await
        .map(|info| info.into_iter().map(|user_info| user_info.into()).collect())
}

pub async fn set_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::ManagedError(
            "You cannot set yourself as platform admin".to_string(),
        ));
    }

    user::set_platform_admin(&user_id).await
}

pub async fn unset_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::ManagedError(
            "You cannot unset yourself as platform admin".to_string(),
        ));
    }

    user::unset_platform_admin(&user_id).await
}

pub async fn activate_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::ManagedError(
            "You cannot activate yourself".to_string(),
        ));
    }

    user::activate_user(&user_id).await
}

pub async fn deactivate_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::ManagedError(
            "You cannot deactivate yourself".to_string(),
        ));
    }

    user::deactivate_user(&user_id).await
}

pub async fn get_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<web_app_response::User, AppError> {
    // access control over auth info
    AccessControl::new(auth_info)
        .await?
        .is_platform_admin()
        .await?;
    let user_model = user::get_user(&user_id).await?;

    web_app_response::User::try_from(user_model)
}

pub async fn create_user(
    auth_info: impl AuthInfo,
    payload: web_app_request::CreateUser,
) -> Result<String, AppError> {
    // access control over auth info
    AccessControl::new(auth_info)
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
}

pub async fn delete_user(auth_info: impl AuthInfo, user_id: DocumentId) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::ManagedError(
            "You cannot delete yourself".to_string(),
        ));
    }
    user::delete_user(&user_id).await
}
