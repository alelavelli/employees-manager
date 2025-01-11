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

pub async fn answer_to_invite_add_company(
    auth_info: impl AuthInfo,
    payload: web_app_request::InviteAddCompanyAnswer,
) -> Result<(), AppError> {
    AccessControl::new(auth_info.clone()).await?;
    if let Some(notification) = notification::get_notification(&payload.notification_id).await? {
        if notification.user_id != *auth_info.user_id() {
            Err(AppError::ManagedError(format!(
                "Notification with id {} does not exist",
                payload.notification_id
            )))
        } else if notification.notification_type != NotificationType::InviteAddCompany {
            Err(AppError::ManagedError(format!(
                "Notification with id {} is not of type Invite Add Company",
                payload.notification_id
            )))
        } else {
            notification::answer_to_invite_add_company(notification, payload.accept).await
        }
    } else {
        Err(AppError::ManagedError(format!(
            "Notification with id {} does not exist",
            payload.notification_id
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

pub async fn get_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<web_app_response::Company, AppError> {
    AccessControl::new(auth_info.clone()).await?;
    // any User can read his companies hence, we don't have access control
    let company_model = company::get_user_company(auth_info.user_id(), &company_id).await?;
    Ok(web_app_response::Company {
        id: company_model
            .id
            .expect("field company_id should exist since the model comes from a db query")
            .to_hex(),
        name: company_model.name,
    })
}

pub async fn add_company_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
    company_id: DocumentId,
    role: CompanyRole,
    job_title: String,
) -> Result<(), AppError> {
    // only company owner and admin can add users to the company
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

    let company =
        db_entities::Company::find_one::<db_entities::Company>(doc! {"_id": company_id}).await?;
    if let Some(company) = company {
        let mut company_assignment = db_entities::UserCompanyAssignment {
            id: None,
            company_id: *company.get_id().expect("Company id should exist"),
            user_id,
            role,
            job_title,
        };
        company_assignment.save(None).await?;
        Ok(())
    } else {
        Err(AppError::ManagedError(format!(
            "Company with id {} does not exist",
            company_id
        )))
    }
}

pub async fn remove_company_user(
    auth_info: impl AuthInfo,
    user_id: DocumentId,
    company_id: DocumentId,
) -> Result<(), AppError> {
    AccessControl::new(auth_info)
        .await?
        .has_company_role_or_higher(&company_id, CompanyRole::Admin)
        .await?;

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
    AccessControl::new(auth_info.clone()).await?;
    Ok(company::get_users_in_company(&company_id)
        .await?
        .iter()
        .map(|doc| web_app_response::UserInCompanyInfo {
            user_id: doc.user_id.to_hex(),
            company_id: doc.company_id.to_hex(),
            role: doc.role,
            job_title: doc.job_title.clone(),
            management_team: doc.management_team,
        })
        .collect())
}
