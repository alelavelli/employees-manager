use std::str::FromStr;

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    enums::{CompanyRole, EmployeeRequest},
    error::DatabaseError,
    service::db::DatabaseDocument,
    DocumentId,
};

/// Struct representing user model
#[derive(Debug, Serialize, Deserialize, Clone)]
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
    /// if the user is active and can operate on application
    pub active: bool,
}

/// Assignment of a user to a company
///
/// A User has a CompanyRole in the Company and a Job Title
#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyManagementTeam {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub company_id: DocumentId,
    pub user_ids: Vec<DocumentId>,
}

/// Struct representing a company that has some employees
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Company {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub name: String,
    /// If the company is active, it is automatically deactivated when the
    /// owner is deactivated
    pub active: bool,
}

/// Employee request in a company
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyEmployeeRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<DocumentId>,
    pub user_id: DocumentId,
    pub company_id: DocumentId,
    pub request: EmployeeRequest,
}

// Impl blocks

impl DatabaseDocument for User {
    fn collection_name() -> &'static str {
        "user"
    }

    fn get_id(&self) -> Option<&DocumentId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, document_id: &str) -> Result<(), DatabaseError> {
        if self.id.is_some() {
            Err(DatabaseError::DocumentHasAlreadyAnId)
        } else if let Ok(parsed_id) = ObjectId::from_str(document_id) {
            self.id = Some(parsed_id);
            Ok(())
        } else {
            Err(DatabaseError::InvalidObjectId)
        }
    }
}

impl DatabaseDocument for UserCompanyAssignment {
    fn collection_name() -> &'static str {
        "user_company_assignment"
    }

    fn get_id(&self) -> Option<&DocumentId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, document_id: &str) -> Result<(), DatabaseError> {
        if self.id.is_some() {
            Err(DatabaseError::DocumentHasAlreadyAnId)
        } else if let Ok(parsed_id) = ObjectId::from_str(document_id) {
            self.id = Some(parsed_id);
            Ok(())
        } else {
            Err(DatabaseError::InvalidObjectId)
        }
    }
}

impl DatabaseDocument for CompanyManagementTeam {
    fn collection_name() -> &'static str {
        "company_management_team"
    }

    fn get_id(&self) -> Option<&DocumentId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, document_id: &str) -> Result<(), DatabaseError> {
        if self.id.is_some() {
            Err(DatabaseError::DocumentHasAlreadyAnId)
        } else if let Ok(parsed_id) = ObjectId::from_str(document_id) {
            self.id = Some(parsed_id);
            Ok(())
        } else {
            Err(DatabaseError::InvalidObjectId)
        }
    }
}

impl DatabaseDocument for Company {
    fn collection_name() -> &'static str {
        "company"
    }

    fn get_id(&self) -> Option<&DocumentId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, document_id: &str) -> Result<(), DatabaseError> {
        if self.id.is_some() {
            Err(DatabaseError::DocumentHasAlreadyAnId)
        } else if let Ok(parsed_id) = ObjectId::from_str(document_id) {
            self.id = Some(parsed_id);
            Ok(())
        } else {
            Err(DatabaseError::InvalidObjectId)
        }
    }
}

impl DatabaseDocument for CompanyEmployeeRequest {
    fn collection_name() -> &'static str {
        "company_employee_request"
    }

    fn get_id(&self) -> Option<&DocumentId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, document_id: &str) -> Result<(), DatabaseError> {
        if self.id.is_some() {
            Err(DatabaseError::DocumentHasAlreadyAnId)
        } else if let Ok(parsed_id) = ObjectId::from_str(document_id) {
            self.id = Some(parsed_id);
            Ok(())
        } else {
            Err(DatabaseError::InvalidObjectId)
        }
    }
}
