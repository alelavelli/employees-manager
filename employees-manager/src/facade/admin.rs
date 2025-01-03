use crate::{
    auth::AuthInfo,
    dtos::{web_app_request, web_app_response},
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
    Ok(web_app_response::AdminPanelOverview {
        total_users: users_info.total_users,
        total_admins: users_info.total_admins,
        total_active_users: users_info.total_active_users,
        total_inactive_users: users_info.total_inactive_users,
        total_companies: companies_info.total_companies,
    })
}

pub async fn get_admin_panel_users_info(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::AdminPanelUserInfo>, AppError> {
    AccessControl::new(auth_info)
        .await?
        .is_platform_admin()
        .await?;
    user::get_admin_panel_users_info().await.map(|info| {
        info.into_iter()
            .map(|user_info| web_app_response::AdminPanelUserInfo {
                id: user_info.id.to_hex(),
                username: user_info.username,
                email: user_info.email,
                name: user_info.name,
                surname: user_info.surname,
                active: user_info.active,
                platform_admin: user_info.platform_admin,
                total_companies: user_info.total_companies,
            })
            .collect()
    })
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
    Ok(web_app_response::User {
        id: user_model
            .id
            .expect("field user_id should exist since the model comes from a db query"),
        username: user_model.username,
    })
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
