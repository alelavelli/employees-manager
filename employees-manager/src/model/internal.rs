use crate::{enums::CompanyRole, DocumentId};

use super::db_entities;

/// Internal data type returned by the user service to the facade for the admin panel overview
#[derive(Default)]
pub struct AdminPanelOverviewUserInfo {
    pub total_users: u16,
    pub total_admins: u16,
    pub total_active_users: u16,
    pub total_inactive_users: u16,
}

pub struct AdminPanelUserInfo {
    pub id: DocumentId,
    pub username: String,
    pub email: String,
    pub name: String,
    pub surname: String,
    pub platform_admin: bool,
    pub active: bool,
    pub total_companies: u16,
}

/// Internal data type returned by the company service to the facade for the admin panel overview
pub struct AdminPanelOverviewCompanyInfo {
    pub total_companies: u16,
}

/// Internal data type returned to give information of a user inside a company
pub struct UserInCompanyInfo {
    pub user_id: DocumentId,
    pub company_id: DocumentId,
    pub role: CompanyRole,
    pub username: String,
    pub name: String,
    pub surname: String,
    pub job_title: String,
    pub management_team: bool,
}

pub struct InvitedUserInCompanyInfo {
    pub notification_id: String,
    pub user_id: String,
    pub username: String,
    pub role: CompanyRole,
    pub job_title: String,
    pub company_id: String,
}

/// Internal data type that contains working hours for a single project activity
pub struct TimesheetActivityHours {
    pub company_id: DocumentId,
    pub project_id: DocumentId,
    pub activity_id: DocumentId,
    pub description: String,
    pub hours: u32,
}

impl Into<db_entities::TimesheetActivityHours> for TimesheetActivityHours {
    fn into(self) -> db_entities::TimesheetActivityHours {
        db_entities::TimesheetActivityHours::new(
            self.company_id,
            self.project_id,
            self.activity_id,
            self.description,
            self.hours,
        )
    }
}
