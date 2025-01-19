use jsonwebtoken::Header;
use mongodb::bson::doc;

use crate::{
    auth::{AuthInfo, JWTAuthClaim},
    dtos::{web_app_request, web_app_response},
    enums::{CompanyRole, NotificationType},
    error::AppError,
    model::db_entities,
    service::{access_control::AccessControl, company, db::DatabaseDocument, notification, user},
    DocumentId,
};

pub async fn authenticate_user(
    username: &str,
    password: &str,
) -> Result<web_app_response::JWTAuthResponse, AppError> {
    let user_model = user::login(username, password).await?;

    let claims = JWTAuthClaim {
        exp: 2000000000,
        user_id: user_model.id.expect("User id must be not missing"),
        username: user_model.username,
    };
    let token = claims.build_token(&Header::default())?;

    Ok(web_app_response::JWTAuthResponse {
        token,
        token_type: "Bearer".into(),
    })
}

pub async fn get_auth_user_data(
    auth_info: impl AuthInfo,
) -> Result<web_app_response::AuthUserData, AppError> {
    AccessControl::new(auth_info.clone()).await?;
    let user_model = user::get_user(auth_info.user_id()).await?;
    Ok(web_app_response::AuthUserData {
        id: user_model
            .id
            .expect("field id should exist since the model comes from a db query")
            .to_hex(),
        username: user_model.username,
        email: user_model.email,
        name: user_model.name,
        surname: user_model.surname,
        platform_admin: user_model.platform_admin,
        active: user_model.active,
    })
}

pub async fn get_unread_notifications(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::AppNotification>, AppError> {
    AccessControl::new(auth_info.clone()).await?;
    let notifications: Vec<db_entities::AppNotification> =
        notification::get_unread_notifications(auth_info.user_id()).await?;
    Ok(notifications
        .iter()
        .map(|doc| web_app_response::AppNotification {
            id: (*doc
                .get_id()
                .expect("expected document id for document read from database"))
            .to_hex(),
            notification_type: doc.notification_type,
            message: doc.message.clone(),
        })
        .collect())
}

pub async fn set_notification_as_read(
    auth_info: impl AuthInfo,
    notification_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone()).await?;
    if let Some(notification) = notification::get_notification(&notification_id).await? {
        if notification.user_id != *auth_info.user_id() {
            Err(AppError::ManagedError(format!(
                "Notification with id {} does not exist",
                notification_id
            )))
        } else {
            notification::set_notification_as_read(notification).await
        }
    } else {
        Err(AppError::ManagedError(format!(
            "Notification with id {} does not exist",
            notification_id
        )))
    }
}

pub async fn answer_to_invite_add_company(
    auth_info: impl AuthInfo,
    notification_id: DocumentId,
    payload: web_app_request::InviteAddCompanyAnswer,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone()).await?;
    if let Some(notification) = notification::get_notification(&notification_id).await? {
        if notification.user_id != *auth_info.user_id() {
            Err(AppError::ManagedError(format!(
                "Notification with id {} does not exist",
                notification_id
            )))
        } else if notification.notification_type != NotificationType::InviteAddCompany {
            Err(AppError::ManagedError(format!(
                "Notification with id {} is not of type Invite Add Company",
                notification_id
            )))
        } else {
            notification::answer_to_invite_add_company(notification, payload.accept).await
        }
    } else {
        Err(AppError::ManagedError(format!(
            "Notification with id {} does not exist",
            notification_id
        )))
    }
}

pub async fn create_company(
    auth_info: impl AuthInfo,
    payload: web_app_request::CreateCompany,
) -> Result<String, AppError> {
    AccessControl::new(auth_info.clone()).await?;
    // we need to verify that the a Company with the same name does not already exist
    let companies = company::get_companies().await?;
    for company in companies {
        if payload.name == company.name {
            return Err(AppError::ManagedError(format!(
                "Failed to create Company: Company with name {} already exists.",
                payload.name
            )))?;
        }
    }
    company::create_company(auth_info.user_id(), payload.name, payload.job_title).await
}

pub async fn invite_user_to_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::InviteUserToCompany,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::invite_user(
        *auth_info.user_id(),
        company_id,
        payload.user_id,
        payload.role,
        payload.job_title,
    )
    .await
}

pub async fn get_users_to_invite_in_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::UserToInviteInCompany>, AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    Ok(company::get_users_to_invite_in_company(company_id)
        .await?
        .into_iter()
        .map(
            |(user_id, username)| web_app_response::UserToInviteInCompany {
                username,
                user_id: user_id.to_hex(),
            },
        )
        .collect())
}

pub async fn remove_company_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
    company_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    if auth_info.user_id() == &user_id {
        return Err(AppError::ManagedError(
            "You cannot remove yourself from the company".into(),
        ));
    }
    let company_assignment = db_entities::UserCompanyAssignment::find_one::<
        db_entities::UserCompanyAssignment,
    >(doc! {"company_id": company_id, "user_id": user_id})
    .await?;
    if let Some(company_assignment) = company_assignment {
        company_assignment.delete(None).await?;
        Ok(())
    } else {
        Err(AppError::ManagedError(format!(
            "User with id {} is not assigned to company with id {}",
            user_id, company_id
        )))
    }
}

pub async fn get_companies_of_user(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::CompanyInfo>, AppError> {
    AccessControl::new(auth_info.clone()).await?;
    let companies = company::get_user_companies(auth_info.user_id()).await?;
    let mut to_return = vec![];
    for doc in companies {
        let id = *doc
            .get_id()
            .expect("expecting document id since it has been loaded from db.");
        to_return.push(web_app_response::CompanyInfo {
            id: id.to_hex(),
            name: doc.name,
            active: doc.active,
            total_users: company::get_users_in_company(&id).await?.len() as u16,
            role: company::get_user_company_role(auth_info.user_id(), &id)
                .await?
                .role,
        })
    }
    Ok(to_return)
}

pub async fn get_users_in_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::UserInCompanyInfo>, AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    Ok(company::get_users_in_company(&company_id)
        .await?
        .iter()
        .map(|doc| web_app_response::UserInCompanyInfo {
            user_id: doc.user_id.to_hex(),
            company_id: doc.company_id.to_hex(),
            role: doc.role,
            user_surname: doc.surname.clone(),
            user_name: doc.name.clone(),
            user_username: doc.username.clone(),
            job_title: doc.job_title.clone(),
            management_team: doc.management_team,
        })
        .collect())
}

pub async fn change_user_company_role(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::ChangeUserCompanyRole,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone())
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    // A user cannot change the role of himself
    if auth_info.user_id() == &payload.user_id {
        Err(AppError::ManagedError(
            "You cannot change your own role".into(),
        ))
    } else {
        company::update_user_in_company(&payload.user_id, &company_id, Some(payload.role), None)
            .await
    }
}

pub async fn change_user_company_job_title(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::ChangeUserJobTitle,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    company::update_user_in_company(&payload.user_id, &company_id, None, Some(payload.job_title))
        .await
}

pub async fn change_user_company_manager(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::ChangeUserCompanyManager,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    company::change_user_company_manager(&payload.user_id, &company_id, payload.manager).await
}

pub async fn get_pending_invited_users_in_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::InvitedUserInCompanyInfo>, AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    Ok(company::get_pending_invited_users(&company_id)
        .await?
        .into_iter()
        .map(|elem| web_app_response::InvitedUserInCompanyInfo {
            notification_id: elem.notification_id,
            user_id: elem.user_id,
            username: elem.username,
            role: elem.role,
            job_title: elem.job_title,
            company_id: elem.company_id,
        })
        .collect())
}

pub async fn cancel_invite_user_to_company(
    auth_info: impl AuthInfo,
    notification_id: DocumentId,
    company_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    notification::cancel_invite_user_to_company(notification_id).await
}

pub async fn get_company_projects(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::CompanyProjectInfo>, AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::User)
        .await?;

    Ok(company::get_company_projects(company_id)
        .await?
        .into_iter()
        .map(|elem| web_app_response::CompanyProjectInfo {
            id: elem
                .get_id()
                .expect("Expecting document id from doc read from db")
                .to_hex(),
            name: elem.name,
            code: elem.code,
        })
        .collect())
}

pub async fn create_company_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::CreateCompanyProject,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::create_project(company_id, payload.name, payload.code).await?;
    Ok(())
}

pub async fn edit_company_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
    payload: web_app_request::EditCompanyProject,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_project(company_id, project_id, payload.name, payload.code).await?;
    Ok(())
}

pub async fn delete_company_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    company::delete_project(company_id, project_id).await
}
