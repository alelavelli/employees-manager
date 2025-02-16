use jsonwebtoken::Header;
use mongodb::bson::doc;

use crate::{
    auth::{AuthInfo, JWTAuthClaim},
    dtos::{web_app_request, web_app_response},
    enums::{CompanyRole, NotificationType},
    error::{AppError, ServiceAppError},
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
        user_id: *user_model.get_id().expect("User id must be not missing"),
        username: user_model.username().clone(),
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
    AccessControl::new(&auth_info).await?;
    let user_model = user::get_user(auth_info.user_id())
        .await
        .map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })?;
    web_app_response::AuthUserData::try_from(user_model).map_err(|_| {
        AppError::InternalServerError("Error in building response from User document".into())
    })
}

pub async fn get_unread_notifications(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::AppNotification>, AppError> {
    AccessControl::new(&auth_info).await?;
    let notifications: Vec<db_entities::AppNotification> =
        notification::get_unread_notifications(auth_info.user_id())
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    // flat_map filters out entities that are not Ok(), since this conversion should not fail
    // because the documents are read from the database we are safe
    Ok(notifications
        .into_iter()
        .flat_map(web_app_response::AppNotification::try_from)
        .collect())
}

pub async fn set_notification_as_read(
    auth_info: impl AuthInfo,
    notification_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info).await?;
    if let Some(notification) = notification::get_notification(&notification_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        if *notification.user_id() != *auth_info.user_id() {
            Err(AppError::DoesNotExist(format!(
                "Notification with id {} does not exist",
                notification_id
            )))
        } else {
            notification::set_notification_as_read(notification)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))
        }
    } else {
        Err(AppError::DoesNotExist(format!(
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
    AccessControl::new(&auth_info).await?;
    if let Some(notification) = notification::get_notification(&notification_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
    {
        if *notification.user_id() != *auth_info.user_id() {
            Err(AppError::DoesNotExist(format!(
                "Notification with id {} does not exist",
                notification_id
            )))
        } else if *notification.notification_type() != NotificationType::InviteAddCompany {
            Err(AppError::DoesNotExist(format!(
                "Notification with id {} is not of type Invite Add Company",
                notification_id
            )))
        } else {
            notification::answer_to_invite_add_company(notification, payload.accept)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))
        }
    } else {
        Err(AppError::DoesNotExist(format!(
            "Notification with id {} does not exist",
            notification_id
        )))
    }
}

pub async fn create_company(
    auth_info: impl AuthInfo,
    payload: web_app_request::CreateCompany,
) -> Result<String, AppError> {
    AccessControl::new(&auth_info).await?;
    company::create_company(auth_info.user_id(), payload.name, payload.job_title)
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn invite_user_to_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::InviteUserToCompany,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::invite_user(
        *auth_info.user_id(),
        company_id,
        payload.user_id,
        payload.role,
        payload.job_title,
        payload.project_ids,
    )
    .await
    .map_err(|e| match e {
        ServiceAppError::AccessControlError(message) => AppError::AccessControlError(message),
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
        _ => AppError::InternalServerError(e.to_string()),
    })
}

pub async fn get_users_to_invite_in_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::UserToInviteInCompany>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    Ok(company::get_users_to_invite_in_company(company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .map(|(user_id, username)| web_app_response::UserToInviteInCompany::new(user_id, username))
        .collect())
}

pub async fn remove_company_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
    company_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    if auth_info.user_id() == &user_id {
        return Err(AppError::InvalidRequest(
            "You cannot remove yourself from the company".into(),
        ));
    }
    let company_assignment = db_entities::UserCompanyAssignment::find_one(
        doc! {"company_id": company_id, "user_id": user_id},
    )
    .await.map_err(|e| {
        match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(format!("Error {e} occurred with `find_one` query over `UserCompanyAssignment` for `company_id`: {company_id} and `user_id`: {user_id}"))
        }
    })?;
    if let Some(company_assignment) = company_assignment {
        company_assignment.delete(None).await.map_err(|e| AppError::InternalServerError(format!("Error {e} occurred with `delete` of company_assignment for `company_id`: {company_id} and `user_id`: {user_id}")))?;
        Ok(())
    } else {
        Err(AppError::InvalidRequest(format!(
            "User with id {} is not assigned to company with id {}",
            user_id, company_id
        )))
    }
}

pub async fn get_companies_of_user(
    auth_info: impl AuthInfo,
) -> Result<Vec<web_app_response::CompanyInfo>, AppError> {
    AccessControl::new(&auth_info).await?;
    let companies = company::get_user_companies(auth_info.user_id())
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let mut to_return = vec![];
    for doc in companies {
        let id = *doc
            .get_id()
            .expect("expecting document id since it has been loaded from db.");
        to_return.push(
            web_app_response::CompanyInfoBuilder::default()
                .id(id.to_string())
                .name(doc.name().clone())
                .active(*doc.active())
                .total_users(
                    company::get_users_in_company(&id)
                        .await
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?
                        .len() as u16,
                )
                .role(
                    *company::get_user_company_role(auth_info.user_id(), &id)
                        .await
                        .map_err(|e| match e {
                            ServiceAppError::EntityDoesNotExist(message) => {
                                AppError::DoesNotExist(message)
                            }
                            _ => AppError::InternalServerError(e.to_string()),
                        })?
                        .role(),
                )
                .build()
                .map_err(|_| {
                    AppError::InternalServerError(
                        "Error in building response for companies of user".into(),
                    )
                })?,
        );
    }
    Ok(to_return)
}

pub async fn get_users_in_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::UserInCompanyInfo>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    Ok(company::get_users_in_company(&company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .map(web_app_response::UserInCompanyInfo::from)
        .collect())
}

pub async fn change_user_company_role(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::ChangeUserCompanyRole,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    // A user cannot change the role of himself
    if auth_info.user_id() == &payload.user_id {
        Err(AppError::InvalidRequest(
            "You cannot change your own role".into(),
        ))
    } else {
        company::update_user_in_company(&payload.user_id, &company_id, Some(payload.role), None)
            .await
            .map_err(|e| match e {
                ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }
}

pub async fn change_user_company_job_title(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::ChangeUserJobTitle,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    company::update_user_in_company(&payload.user_id, &company_id, None, Some(payload.job_title))
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn change_user_company_manager(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::ChangeUserCompanyManager,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    company::change_user_company_manager(&payload.user_id, &company_id, payload.manager)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub async fn get_pending_invited_users_in_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::InvitedUserInCompanyInfo>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    Ok(company::get_pending_invited_users(&company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .map(web_app_response::InvitedUserInCompanyInfo::from)
        .collect())
}

pub async fn cancel_invite_user_to_company(
    auth_info: impl AuthInfo,
    notification_id: DocumentId,
    company_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    notification::cancel_invite_user_to_company(notification_id)
        .await
        .map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn get_company_projects(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::CompanyProjectInfo>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    Ok(company::get_company_projects(company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .flat_map(web_app_response::CompanyProjectInfo::try_from)
        .collect())
}

pub async fn get_company_project_allocations_by_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
) -> Result<Vec<String>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    // TODO: optimize this by filtering directly the query
    let allocation: Option<Vec<String>> = company::get_company_project_allocations(company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .filter(|(p, _)| p == &project_id)
        .map(|(_, user_ids)| {
            user_ids
                .into_iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
        })
        .next();
    Ok(allocation.unwrap_or_default())
}

pub async fn get_company_project_allocations_by_user(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    user_id: DocumentId,
) -> Result<Vec<String>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    // TODO: optimize this by filtering directly the query
    let allocation: Vec<String> = company::get_company_project_allocations(company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .filter(|(_, user_ids)| user_ids.contains(&user_id))
        .map(|(project_id, _)| project_id.to_string())
        .collect();

    Ok(allocation)
}

pub async fn create_company_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::CreateCompanyProject,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::create_project(company_id, payload.name, payload.code)
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            _ => AppError::InternalServerError(e.to_string()),
        })?;
    Ok(())
}

pub async fn edit_company_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
    payload: web_app_request::EditCompanyProject,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_project(
        company_id,
        project_id,
        payload.name,
        payload.code,
        payload.active,
    )
    .await
    .map_err(|e| match e {
        ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })?;
    Ok(())
}

pub async fn delete_company_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;
    company::delete_project(company_id, project_id)
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn edit_company_project_allocations_by_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
    payload: web_app_request::ChangeProjectAllocations,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_company_project_allocations(company_id, project_id, payload.user_ids)
        .await
        .map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn edit_company_project_allocations_by_user(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    user_id: DocumentId,
    payload: web_app_request::ChangeProjectAllocationsForUser,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_company_project_allocations_for_user(company_id, user_id, payload.project_ids)
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn create_project_activity(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    payload: web_app_request::NewProjectActivity,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::create_company_project_activity(company_id, payload.name, payload.description)
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn get_project_activities(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<Vec<web_app_response::ProjectActivityInfo>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::User)
        .await?;

    Ok(company::get_company_project_activities(company_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?
        .into_iter()
        .flat_map(web_app_response::ProjectActivityInfo::try_from)
        .collect())
}

pub async fn edit_project_activity(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    activity_id: DocumentId,
    payload: web_app_request::EditProjectActivity,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_company_project_activity(
        company_id,
        activity_id,
        payload.name,
        payload.description,
    )
    .await
    .map_err(|e| match e {
        ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
        ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
        _ => AppError::InternalServerError(e.to_string()),
    })?;

    Ok(())
}

pub async fn delete_project_activity(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    activity_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::delete_company_project_activity(company_id, activity_id)
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })?;

    Ok(())
}

pub async fn get_project_activity_assignment_by_activity(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    activity_id: DocumentId,
) -> Result<Vec<String>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::User)
        .await?;

    company::get_projects_with_activity(activity_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub async fn get_project_activity_assignment_by_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
) -> Result<Vec<String>, AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::User)
        .await?;

    company::get_projects_activity_assignment(project_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub async fn edit_project_activity_assignment_by_activity(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    activity_id: DocumentId,
    project_ids: Vec<DocumentId>,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_project_activity_assignment_by_activity(activity_id, project_ids)
        .await
        .map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}

pub async fn edit_project_activity_assignment_by_project(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
    project_id: DocumentId,
    activity_ids: Vec<DocumentId>,
) -> Result<(), AppError> {
    AccessControl::new(&auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    company::edit_project_activity_assignment(company_id, project_id, activity_ids)
        .await
        .map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
}
