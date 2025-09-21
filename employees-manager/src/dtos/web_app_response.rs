use chrono::{DateTime, Utc};
use derive_builder::Builder;
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

use crate::{
    enums::{CompanyRole, NotificationType, WorkingDayType},
    error::ServiceAppError,
    model::{db_entities, internal},
    service::db::document::DatabaseDocument,
};
pub mod admin_panel;
pub mod corporate_group;
/// Authorization response for jwt token
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JWTAuthResponse {
    pub token: String,
    pub token_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: String,
    username: String,
}

impl TryFrom<db_entities::User> for User {
    type Error = ServiceAppError;

    fn try_from(value: db_entities::User) -> Result<Self, Self::Error> {
        if let Some(id) = value.get_id() {
            Ok(Self {
                id: id.to_hex(),
                username: value.username().into(),
            })
        } else {
            Err(ServiceAppError::ResponseBuildError(
                "Document Id should exist for User".into(),
            ))
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserData {
    id: String,
    username: String,
    email: String,
    name: String,
    surname: String,
    platform_admin: bool,
    active: bool,
}

impl TryFrom<db_entities::User> for AuthUserData {
    type Error = ServiceAppError;

    fn try_from(value: db_entities::User) -> Result<Self, Self::Error> {
        if let Some(id) = value.get_id() {
            Ok(Self {
                id: id.to_hex(),
                username: value.username().into(),
                email: value.email().into(),
                name: value.name().into(),
                surname: value.surname().into(),
                platform_admin: *value.platform_admin(),
                active: *value.active(),
            })
        } else {
            Err(ServiceAppError::ResponseBuildError(
                "Document Id should exist for User".into(),
            ))
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    id: String,
    notification_type: NotificationType,
    message: String,
}

impl TryFrom<db_entities::AppNotification> for AppNotification {
    type Error = ServiceAppError;

    fn try_from(value: db_entities::AppNotification) -> Result<Self, Self::Error> {
        if let Some(id) = value.get_id() {
            Ok(Self {
                id: id.to_hex(),
                notification_type: *value.notification_type(),
                message: value.message().into(),
            })
        } else {
            Err(ServiceAppError::ResponseBuildError(
                "Document Id should exist for User".into(),
            ))
        }
    }
}

#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfo {
    id: String,
    name: String,
    active: bool,
    role: CompanyRole,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInCompanyInfo {
    user_id: String,
    company_id: String,
    user_username: String,
    user_name: String,
    user_surname: String,
    job_title: String,
}

impl From<internal::UserInCompanyInfo> for UserInCompanyInfo {
    fn from(value: internal::UserInCompanyInfo) -> Self {
        Self {
            user_id: value.user_id.to_string(),
            company_id: value.company_id.to_string(),
            user_surname: value.surname,
            user_name: value.name,
            user_username: value.username,
            job_title: value.job_title,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserToInviteInCompany {
    username: String,
    user_id: String,
}

impl UserToInviteInCompany {
    pub fn new(user_id: ObjectId, username: String) -> Self {
        Self {
            user_id: user_id.to_hex(),
            username,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitedUserInCompanyInfo {
    notification_id: String,
    user_id: String,
    username: String,
    role: CompanyRole,
    job_title: String,
    company_id: String,
}

impl From<internal::InvitedUserInCompanyInfo> for InvitedUserInCompanyInfo {
    fn from(value: internal::InvitedUserInCompanyInfo) -> Self {
        Self {
            notification_id: value.notification_id,
            user_id: value.user_id,
            username: value.username,
            role: value.role,
            job_title: value.job_title,
            company_id: value.company_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProjectInfo {
    id: String,
    name: String,
    code: String,
    active: bool,
}

impl TryFrom<db_entities::CompanyProject> for CompanyProjectInfo {
    type Error = ServiceAppError;

    fn try_from(value: db_entities::CompanyProject) -> Result<Self, Self::Error> {
        if let Some(id) = value.get_id() {
            Ok(Self {
                id: id.to_hex(),
                name: value.name().into(),
                code: value.code().into(),
                active: *value.active(),
            })
        } else {
            Err(ServiceAppError::ResponseBuildError(
                "Document Id should exist for CompanyProject".into(),
            ))
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectActivityInfo {
    id: String,
    name: String,
    description: String,
}

impl TryFrom<db_entities::ProjectActivity> for ProjectActivityInfo {
    type Error = ServiceAppError;

    fn try_from(value: db_entities::ProjectActivity) -> Result<Self, Self::Error> {
        if let Some(id) = value.get_id() {
            Ok(Self {
                id: id.to_hex(),
                name: value.name().into(),
                description: value.description().into(),
            })
        } else {
            Err(ServiceAppError::ResponseBuildError(
                "Document Id should exist for ProjectActivity".into(),
            ))
        }
    }
}

#[derive(Serialize, Builder, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetActivityHours {
    pub company_id: String,
    pub company_name: String,
    pub project_id: String,
    pub project_name: String,
    pub activity_id: String,
    pub activity_name: String,
    pub notes: String,
    pub hours: u32,
}

#[derive(Serialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetDay {
    pub user_id: String,
    pub date: DateTime<Utc>,
    pub permit_hours: u32,
    pub working_type: WorkingDayType,
    pub activities: Vec<TimesheetActivityHours>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetProjectInfo {
    pub company_id: String,
    pub company_name: String,
    pub project_id: String,
    pub project_name: String,
    pub activities: Vec<ProjectActivityInfo>,
}
