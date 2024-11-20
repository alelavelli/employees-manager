use std::fmt::Display;

use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

/// Enumeration with roles assigned to Users for a Company
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum CompanyRole {
    /// Basic user
    ///
    /// A User is a standard employee that uses the application functionalities
    User,
    /// Admin user
    ///
    /// A user that has administration privileges, he can assign Users to the Company
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

impl Into<Bson> for CompanyRole {
    fn into(self) -> Bson {
        match self {
            CompanyRole::User => Bson::String("User".to_string()),
            CompanyRole::Admin => Bson::String("Admin".to_string()),
            CompanyRole::Owner => Bson::String("Owner".to_string()),
        }
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
