use serde::Serialize;

use crate::model::internal;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPanelOverview {
    total_users: u16,
    total_admins: u16,
    total_active_users: u16,
    total_inactive_users: u16,
    total_companies: u16,
}

impl
    From<(
        internal::AdminPanelOverviewUserInfo,
        internal::AdminPanelOverviewCompanyInfo,
    )> for AdminPanelOverview
{
    fn from(
        value: (
            internal::AdminPanelOverviewUserInfo,
            internal::AdminPanelOverviewCompanyInfo,
        ),
    ) -> Self {
        Self {
            total_users: value.0.total_users,
            total_admins: value.0.total_admins,
            total_active_users: value.0.total_active_users,
            total_inactive_users: value.0.total_inactive_users,
            total_companies: value.1.total_companies,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPanelUserInfo {
    id: String,
    username: String,
    email: String,
    name: String,
    surname: String,
    platform_admin: bool,
    active: bool,
    total_companies: u16,
}

impl From<internal::AdminPanelUserInfo> for AdminPanelUserInfo {
    fn from(value: internal::AdminPanelUserInfo) -> Self {
        Self {
            id: value.id.to_hex(),
            username: value.username,
            email: value.email,
            name: value.name,
            surname: value.surname,
            platform_admin: value.platform_admin,
            active: value.active,
            total_companies: value.total_companies,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPanelCorporateGroupInfo {
    pub id: String,
    pub name: String,
    pub active: bool,
}
