use serde::Deserialize;

use crate::{enums::CompanyRole, DocumentId};

/// Authorization payload for jwt token
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JWTAuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub name: String,
    pub surname: String,
    pub email: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompany {
    pub name: String,
    /// Job Title the User has on the Company he creates
    pub job_title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteAddCompanyAnswer {
    pub accept: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeUserCompanyRole {
    pub user_id: DocumentId,
    pub role: CompanyRole,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeUserJobTitle {
    pub user_id: DocumentId,
    pub job_title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeUserCompanyManager {
    pub user_id: DocumentId,
    pub manager: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InviteUserToCompany {
    pub user_id: DocumentId,
    pub role: CompanyRole,
    pub job_title: String,
    pub project_ids: Vec<DocumentId>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyProject {
    pub name: String,
    pub code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditCompanyProject {
    pub name: String,
    pub code: String,
    pub active: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeProjectAllocations {
    pub user_ids: Vec<DocumentId>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeProjectAllocationsForUser {
    pub project_ids: Vec<DocumentId>,
}
