use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Enumeration with roles assigned to Users for a Company
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum CompanyRole {
    /// Basic user
    User,
    /// Admin user
    Admin,
    /// Owner user, it is like the Admin but it cannot be removed
    Owner,
}

impl Display for CompanyRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CompanyRole::Admin => "Admin",
                CompanyRole::Owner => "Owner",
                CompanyRole::User => "User",
            }
        )
    }
}

/// Enumeration with employee request for permission or other
/// it has an outcome which is another enumeration that defines
/// how the request is
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum EmployeeRequest {
    /// day of holiday
    Holiday(EmployeeRequestOutcome),
    /// remote work
    Remote(EmployeeRequestOutcome),
    /// half day permission
    Permission(EmployeeRequestOutcome),
}

/// Enumeration with employee request outcome
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum EmployeeRequestOutcome {
    /// when the request is submitted but it is awaiting for a response
    Awaiting,
    /// when the request is accepted by management team
    Accepted,
    /// when the request is refused by management team
    Refused,
}
