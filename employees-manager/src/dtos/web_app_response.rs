use serde::Serialize;

use crate::DocumentId;

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
    pub id: DocumentId,
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
    pub id: DocumentId,
    pub name: String,
}
