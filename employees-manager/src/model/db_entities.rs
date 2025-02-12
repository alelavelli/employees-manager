use mongodb::bson::oid::ObjectId;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::{
    enums::{CompanyRole, EmployeeRequest, NotificationType},
    error::DatabaseError,
    service::db::DatabaseDocument,
    DocumentId,
};

/// The macro generates struct that implements DatabaseDocument trait
///
/// You need to provide struct level docstring, the name of the struct, the name of the mongodb collection and the fields with their type
macro_rules! database_document {
    ( $(#[doc = $doc:expr])* $struct_name:ident, $collection_name:expr, $ ( $field_name:ident : $field_type:ty ),* ) => {
        $( #[doc = $doc] )*
        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct $struct_name {
            #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
            id: Option<DocumentId>,
            $ ($field_name: $field_type, )*
        }

        impl $struct_name {
            #[allow(dead_code)]
            #[allow(clippy::too_many_arguments)]
            pub fn new($($field_name: $field_type),*) -> Self {
                Self { id: None, $($field_name),*}
            }

            paste!{
                $(
                    #[allow(dead_code)]
                    pub fn $field_name(&self) -> &$field_type {
                        &self.$field_name
                    }
                    #[allow(dead_code)]
                    pub fn [<$field_name _mut>](&mut self) -> &mut $field_type {
                        &mut self.$field_name
                    }
                    #[allow(dead_code)]
                    pub fn [<set_ $field_name>](&mut self, value: $field_type) {
                        self.$field_name = value;
                    }
                )*
            }
        }


        impl DatabaseDocument for $struct_name {
            fn collection_name() -> &'static str {
                $collection_name
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
    };
}

database_document!(
    #[doc = "User inside the Platform"]
    #[doc = ""]
    #[doc = "It must have unique username and email"]
    User,
    "user",
    email: String,
    username: String,
    password_hash: String,
    name: String,
    surname: String,
    api_key: Option<String>,
    platform_admin: bool,
    active: bool
);

database_document!(
    #[doc = "Assignment of a user to a company"]
    #[doc = ""]
    #[doc = "A User has a CompanyRole in the Company and a Job Title"]
    #[doc = "the user has a list of projects that he is assigned to that"]
    #[doc = "he can select in the timesheet"]
    UserCompanyAssignment,
    "user_company_assignment",
    user_id: DocumentId,
    company_id: DocumentId,
    role: CompanyRole,
    job_title: String,
    project_ids: Vec<DocumentId>
);

database_document!(
    #[doc = "Management Team is a list of Company Employees that has special permissions"]
    CompanyManagementTeam,
    "company_management_team",
    company_id: DocumentId,
    user_ids: Vec<DocumentId>
);

database_document!(
    #[doc = "Struct representing a company that has some employees"]
    Company,
    "company",
    name: String,
    active: bool
);

database_document!(
    #[doc = "Employee request in a company"]
    CompanyEmployeeRequest,
    "company_employee_request",
    user_id: DocumentId,
    company_id: DocumentId,
    request: EmployeeRequest
);

database_document!(
    #[doc = "Generic notification for the user in the app"]
    AppNotification,
    "app_notification",
    user_id: DocumentId,
    notification_type: NotificationType,
    message: String,
    read: bool,
    entity_id: Option<DocumentId>
);

database_document!(
    #[doc = "Invite for the user in the app"]
    InviteAddCompany,
    "invite_add_company",
    inviting_user_id: DocumentId,
    invited_user_id: DocumentId,
    company_id: DocumentId,
    company_role: CompanyRole,
    job_title: String,
    project_ids: Vec<DocumentId>,
    answer: Option<bool>
);

database_document!(
    #[doc = "Project inside the company"]
    CompanyProject,
    "company_project",
    name: String,
    code: String,
    company_id: DocumentId,
    active: bool
);
