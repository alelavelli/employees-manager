use serde::Deserialize;

use crate::enums::CompanyRole;

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
pub struct AddCompanyUser {
    pub username: String,
    pub role: CompanyRole,
}
