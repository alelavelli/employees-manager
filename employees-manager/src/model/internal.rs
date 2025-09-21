use crate::{dtos::web_app_request, enums::CompanyRole, DocumentId};

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
    pub username: String,
    pub name: String,
    pub surname: String,
    pub job_title: String,
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
    pub work_package_id: DocumentId,
    pub activity_id: DocumentId,
    pub notes: String,
    pub hours: u32,
}

impl From<web_app_request::TimesheetActivityHours> for TimesheetActivityHours {
    fn from(value: web_app_request::TimesheetActivityHours) -> Self {
        Self {
            company_id: value.company_id,
            project_id: value.project_id,
            work_package_id: value.work_package_id,
            activity_id: value.activity_id,
            notes: value.notes,
            hours: value.hours,
        }
    }
}
