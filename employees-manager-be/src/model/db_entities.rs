use anyhow::anyhow;
use axum::async_trait;
use serde::{Deserialize, Serialize};

use crate::{
    enums::{CompanyRole, EmployeeRequest},
    error::AppError,
    service::db::DatabaseDocument,
    DocumentId,
};

/// Struct representing user model
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub name: String,
    pub surname: String,
    pub api_key: Option<String>,
    /// if the user is global platform administrator
    pub platform_admin: bool,
}

/// Assignment of a user to a company
///
/// A User has a CompanyRole in the Company and a Job Title
#[derive(Debug, Serialize, Deserialize)]
pub struct UserCompanyAssignment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub user_id: DocumentId,
    pub company_id: DocumentId,
    pub role: CompanyRole,
    pub job_title: String,
}

/// Management Team is a list of Company Employees that
/// has special permissions
#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyManagementTeam {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub company_id: DocumentId,
    pub user_ids: Vec<DocumentId>,
}

/// Struct representing a company that has some employees
#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub name: String,
}

/// Employee request in a company
#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyEmployeeRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub user_id: DocumentId,
    pub company_id: DocumentId,
    pub request: EmployeeRequest,
}

// Impl blocks

#[async_trait]
impl DatabaseDocument for User {
    fn collection_name() -> &'static str {
        "User"
    }

    fn get_id(&self) -> Result<&DocumentId, AppError> {
        if self.id.is_some() {
            Ok(self.id.as_ref().unwrap())
        } else {
            Err(AppError::InternalServerError(anyhow!(
                "Requested ObjectId for Document but it is None"
            )))
        }
    }
}

#[async_trait]
impl DatabaseDocument for UserCompanyAssignment {
    fn collection_name() -> &'static str {
        "UserCompanyAssignment"
    }

    fn get_id(&self) -> Result<&DocumentId, AppError> {
        if self.id.is_some() {
            Ok(self.id.as_ref().unwrap())
        } else {
            Err(AppError::InternalServerError(anyhow!(
                "Requested ObjectId for Document but it is None"
            )))
        }
    }
}

#[async_trait]
impl DatabaseDocument for CompanyManagementTeam {
    fn collection_name() -> &'static str {
        "CompanyManagementTeam"
    }

    fn get_id(&self) -> Result<&DocumentId, AppError> {
        if self.id.is_some() {
            Ok(self.id.as_ref().unwrap())
        } else {
            Err(AppError::InternalServerError(anyhow!(
                "Requested ObjectId for Document but it is None"
            )))
        }
    }
}

#[async_trait]
impl DatabaseDocument for Company {
    fn collection_name() -> &'static str {
        "Company"
    }

    fn get_id(&self) -> Result<&DocumentId, AppError> {
        if self.id.is_some() {
            Ok(self.id.as_ref().unwrap())
        } else {
            Err(AppError::InternalServerError(anyhow!(
                "Requested ObjectId for Document but it is None"
            )))
        }
    }
}

#[async_trait]
impl DatabaseDocument for CompanyEmployeeRequest {
    fn collection_name() -> &'static str {
        "CompanyEmployeeRequest"
    }

    fn get_id(&self) -> Result<&DocumentId, AppError> {
        if self.id.is_some() {
            Ok(self.id.as_ref().unwrap())
        } else {
            Err(AppError::InternalServerError(anyhow!(
                "Requested ObjectId for Document but it is None"
            )))
        }
    }
}
