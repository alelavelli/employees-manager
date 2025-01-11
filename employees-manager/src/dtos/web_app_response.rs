use serde::Serialize;

use crate::enums::{CompanyRole, NotificationType};

/// Authorization response for jwt token
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JWTAuthResponse {
    pub token: String,
    pub token_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPanelOverview {
    pub total_users: u16,
    pub total_admins: u16,
    pub total_active_users: u16,
    pub total_inactive_users: u16,
    pub total_companies: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPanelUserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: String,
    pub surname: String,
    pub platform_admin: bool,
    pub active: bool,
    pub total_companies: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserData {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: String,
    pub surname: String,
    pub platform_admin: bool,
    pub active: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Company {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppNotification {
    pub id: String,
    pub notification_type: NotificationType,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInfo {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub total_users: u16,
    pub role: CompanyRole,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInCompanyInfo {
    pub user_id: String,
    pub company_id: String,
    pub role: CompanyRole,
    pub job_title: String,
    pub management_team: bool,
}
