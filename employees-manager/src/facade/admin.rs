use crate::{
    auth::AuthInfo,
    dtos::{
        web_app_request,
        web_app_response::{self},
    },
    error::{AppError, ServiceAppError},
    service::{access_control::AccessControl, company, user},
    DocumentId,
};

pub async fn get_admin_panel_overview(
    auth_info: impl AuthInfo,
) -> Result<web_app_response::AdminPanelOverview, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    let users_info = user::get_admin_panel_overview_users_info()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let companies_info = company::get_admin_panel_overview_companies_info()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(web_app_response::AdminPanelOverview::from((
        users_info,
        companies_info,
    )))
}

pub async fn get_admin_panel_users_info(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::AdminPanelUserInfo>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    user::get_admin_panel_users_info()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))
        .map(|info| info.into_iter().map(|user_info| user_info.into()).collect())
}

pub async fn set_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::InvalidRequest(
            "You cannot set yourself as platform admin".into(),
        ));
    }

    user::set_platform_admin(&user_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub async fn unset_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::InvalidRequest(
            "You cannot unset yourself as platform admin".into(),
        ));
    }

    user::unset_platform_admin(&user_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub async fn activate_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::InvalidRequest(
            "You cannot activate yourself".into(),
        ));
    }

    user::activate_user(&user_id).await.map_err(|e| match e {
        ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })
}

pub async fn deactivate_platform_admin(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::InvalidRequest(
            "You cannot deactivate yourself".into(),
        ));
    }

    user::deactivate_user(&user_id).await.map_err(|e| match e {
        ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })
}

pub async fn set_user_password(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
    password: String,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::InvalidRequest(
            "You cannot set password of yourself via this method. You must use reset password."
                .into(),
        ));
    }

    user::update_user(&user_id, None, Some(password), None, None)
        .await
        .map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn get_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
) -> Result<web_app_response::User, AppError> {
    // access control over auth info
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;
    let user_model = user::get_user(&user_id).await.map_err(|e| match e {
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })?;

    web_app_response::User::try_from(user_model).map_err(|_| {
        AppError::InternalServerError("Error in building the response from User document".into())
    })
}

pub async fn create_user(
    auth_info: impl AuthInfo,
    payload: web_app_request::CreateUser,
) -> Result<String, AppError> {
    // access control over auth info
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

pub async fn delete_user(auth_info: impl AuthInfo, user_id: DocumentId) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .is_platform_admin()
        .await?;

    if *auth_info.user_id() == user_id {
        return Err(AppError::InvalidRequest(
            "You cannot delete yourself".into(),
        ));
    }
    user::delete_user(&user_id).await.map_err(|e| match e {
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })
}
