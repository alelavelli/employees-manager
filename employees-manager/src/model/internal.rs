use crate::DocumentId;

/// Internal data type returned by the user service to the facade for the admin panel overview
#[derive(Debug)]
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
