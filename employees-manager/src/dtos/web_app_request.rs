use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    enums::{CompanyRole, WorkingDayType},
    DocumentId,
};

pub mod corporate_group;

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
pub struct SetUserPassword {
    pub password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompany {
    pub name: String,
    pub corporate_group_id: DocumentId,
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProjectActivity {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditProjectActivity {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeProjectActivityAssignmentByActivity {
    pub project_ids: Vec<DocumentId>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeProjectActivityAssignmentByProject {
    pub activity_ids: Vec<DocumentId>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetActivityHours {
    pub company_id: DocumentId,
    pub project_id: DocumentId,
    pub work_package_id: DocumentId,
    pub activity_id: DocumentId,
    pub notes: String,
    pub hours: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTimesheetDay {
    pub date: DateTime<Utc>,
    pub permit_hours: u32,
    pub working_type: WorkingDayType,
    pub activities: Vec<TimesheetActivityHours>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUserTimesheetDays {
    pub year: i32,
    pub month: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetUserTimesheetExport {
    pub year: i32,
    pub month: u32,
}
