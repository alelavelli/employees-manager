use std::fmt::Display;

use mongodb::bson::Bson;
use serde::{Deserialize, Serialize};

use crate::error::ServiceAppError;

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

impl From<CompanyRole> for Bson {
    fn from(value: CompanyRole) -> Self {
        match value {
            CompanyRole::User => Bson::String("User".to_string()),
            CompanyRole::Admin => Bson::String("Admin".to_string()),
            CompanyRole::Owner => Bson::String("Owner".to_string()),
        }
    }
}

impl Eq for CompanyRole {}

impl PartialOrd<CompanyRole> for CompanyRole {
    fn partial_cmp(&self, other: &CompanyRole) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompanyRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            std::cmp::Ordering::Equal
        } else if *self == CompanyRole::User {
            match other {
                CompanyRole::Admin | CompanyRole::Owner => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Equal,
            }
        } else if *self == CompanyRole::Admin {
            match other {
                CompanyRole::Owner => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Greater,
            }
        } else {
            // self == CompanyRole::Owner
            std::cmp::Ordering::Greater
        }
    }
}

/// Enumeration with roles assigned to Users for a Corporate Group
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum CorporateGroupRole {
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

impl Display for CorporateGroupRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CorporateGroupRole::Admin => "Admin",
                CorporateGroupRole::Owner => "Owner",
                CorporateGroupRole::User => "User",
            }
        )
    }
}

impl From<CorporateGroupRole> for Bson {
    fn from(value: CorporateGroupRole) -> Self {
        match value {
            CorporateGroupRole::User => Bson::String("User".to_string()),
            CorporateGroupRole::Admin => Bson::String("Admin".to_string()),
            CorporateGroupRole::Owner => Bson::String("Owner".to_string()),
        }
    }
}

impl Eq for CorporateGroupRole {}

impl PartialOrd<CorporateGroupRole> for CorporateGroupRole {
    fn partial_cmp(&self, other: &CorporateGroupRole) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CorporateGroupRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self == other {
            std::cmp::Ordering::Equal
        } else if *self == CorporateGroupRole::User {
            match other {
                CorporateGroupRole::Admin | CorporateGroupRole::Owner => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Equal,
            }
        } else if *self == CorporateGroupRole::Admin {
            match other {
                CorporateGroupRole::Owner => std::cmp::Ordering::Less,
                _ => std::cmp::Ordering::Greater,
            }
        } else {
            // self == CompanyRole::Owner
            std::cmp::Ordering::Greater
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

/// Enumeration with type of app notification
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum NotificationType {
    InviteAddCompany,
    InviteAddCompanyAnswer,
}

/// Define the type of work in the timesheet
/// each day is marked with this enumeration
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum WorkingDayType {
    /// When the user works at the office
    Office,
    /// When the user works at home
    Remote,
    /// When the user takes a day off from work
    DayOff,
    /// When the day is a public or national holiday
    Holiday,
    /// When the day is a company closure day
    CompanyClosure,
    /// When the user does not work because of illness
    Sick,
}

impl From<WorkingDayType> for Bson {
    fn from(value: WorkingDayType) -> Self {
        match value {
            WorkingDayType::Office => "Office".to_string(),
            WorkingDayType::Remote => "Remote".to_string(),
            WorkingDayType::DayOff => "DayOff".to_string(),
            WorkingDayType::Holiday => "Holiday".to_string(),
            WorkingDayType::CompanyClosure => "CompanyClosure".to_string(),
            WorkingDayType::Sick => "Sick".to_string(),
        }
        .into()
    }
}

impl TryFrom<Bson> for WorkingDayType {
    type Error = ServiceAppError;

    fn try_from(value: Bson) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "Office" => Ok(Self::Office),
            "Remote" => Ok(Self::Remote),
            "DayOff" => Ok(Self::DayOff),
            "Holiday" => Ok(Self::Holiday),
            "CompanyClosure" => Ok(Self::CompanyClosure),
            "Sick" => Ok(Self::Sick),
            _ => Err(ServiceAppError::DatabaseError(
                "Failed to load WorkingDayType from document".into(),
            )),
        }
    }
}

impl Display for WorkingDayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WorkingDayType::Office => "Office",
                WorkingDayType::Remote => "Remote",
                WorkingDayType::DayOff => "DayOff",
                WorkingDayType::Holiday => "Holiday",
                WorkingDayType::CompanyClosure => "CompanyClosure",
                WorkingDayType::Sick => "Sick",
            }
        )
    }
}

/// Enumeration with possible file types stored in the application
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum FileType {
    Xslx,
    Csv,
    Parquet,
}

/// Enumeration with possible object sources used by the application
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum ObjectSourceType {
    AwsS3,
    GcpGS,
    LocalFileSystem,
}

impl TryFrom<&str> for ObjectSourceType {
    type Error = ServiceAppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "awss3" => Ok(Self::AwsS3),
            "gcpgs" => Ok(Self::GcpGS),
            "localfilesystem" => Ok(Self::LocalFileSystem),
            _ => Err(ServiceAppError::InvalidRequest(format!(
                "Cannot create ObjectSourceType from {value}"
            ))),
        }
    }
}

impl Display for ObjectSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ObjectSourceType::LocalFileSystem => "LocalFileSystem",
                ObjectSourceType::AwsS3 => "AwsS3",
                ObjectSourceType::GcpGS => "GcpGS",
            }
        )
    }
}

/// Define which type of configuration is used for the frontend
///
/// The best configuration depends on the specific use case.
/// Usually, integrated mode is useful during testing and to have a
/// single docker file for small releases.
///
/// External is preferred in production environments since the
/// decoupling allows to scale and update separately the frontend
/// and the backend.
#[derive(Debug, Clone)]
pub enum FrontendMode {
    /// Integrated means that the frontend pages are served
    /// as static content directly by the web server.
    /// The string contained in this variant is the root path
    /// of the folder that contains static files.
    Integrated(String),
    /// External means that the frontend is external to the web
    /// application and the web server is decoupled from it
    External,
}

impl TryFrom<&str> for FrontendMode {
    type Error = ServiceAppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "external" => Ok(Self::External),
            value if value.starts_with("integrated") => {
                let path = value.strip_prefix("integrated:").ok_or(ServiceAppError::InvalidRequest(format!("Cannot create FrontendMode from {value}. integrated:path is the expected format")))?;
                Ok(Self::Integrated(path.to_string()))
            }
            _ => Err(ServiceAppError::InvalidRequest(format!(
                "Cannot create FrontendMode from {value}"
            ))),
        }
    }
}
