use crate::{
    enums::{
        CompanyRole, CorporateGroupRole, EmployeeRequest, FileType, NotificationType,
        ObjectSourceType, WorkingDayType,
    },
    error::DatabaseError,
    service::db::document::DatabaseDocument,
    DocumentId,
};
use bson::{self, doc, Bson};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::internal;

/// The macro generates struct that implements DatabaseDocument trait
///
/// You need to provide struct level docstring, the name of the struct, the name of the mongodb collection and the fields with their type
macro_rules! database_document {
    ( $(#[doc = $doc:expr])* $struct_name:ident, $collection_name:expr, $(
        $(#[$field_attr:meta])*
        $field_name:ident : $field_type:ty
    ),* $(,)? ) => {
        $( #[doc = $doc] )*
        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct $struct_name {
            #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
            id: Option<DocumentId>,
            $(
                $(#[$field_attr])*
                $field_name: $field_type,
            )*
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

/// The macro generates struct used as an object inside the database document
///
/// You need to provide struct level docstring, the name of the struct
macro_rules! embedded_document {
    ( $(#[doc = $doc:expr])* $struct_name:ident, $ ( $field_name:ident : $field_type:ty ),* ) => {
        $( #[doc = $doc] )*
        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct $struct_name {
            $ ($field_name: $field_type, )*
        }

        impl $struct_name {
            #[allow(dead_code)]
            #[allow(clippy::too_many_arguments)]
            pub fn new($($field_name: $field_type),*) -> Self {
                Self { $($field_name),*}
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
    #[serde(skip_serializing_if="Option::is_none")]
    api_key: Option<String>,
    #[doc = "true if the user is administrator of the entire platform applications."]
    platform_admin: bool,
    active: bool,
);

database_document!(
    #[doc = "This document links a user with a corporate group "]
    #[doc = "defining his role in it."]
    UserCorporateGroupRole,
    "user_corporate_group_assignment",
    user_id: DocumentId,
    corporate_group_id: DocumentId,
    role: CorporateGroupRole,
);

database_document!(
    #[doc = "Employment contract between the user and the company"]
    #[doc = ""]
    #[doc = "For now a contract only contains the job title but in future it "]
    #[doc = "will contain every contract information."]
    UserEmploymentContract,
    "user_employment_contract",
    user_id: DocumentId,
    company_id: DocumentId,
    job_title: String,
);

database_document!(
    #[doc = "Role the user has for a specific company"]
    #[doc = ""]
    #[doc = "If the document is not present in database then it defaults "]
    #[doc = "to standard user"]
    UserCompanyRole,
    "user_company_role",
    user_id: DocumentId,
    company_id: DocumentId,
    role: CompanyRole
);

database_document!(
    #[doc = "Assignment of company projects to the user"]
    #[doc = "Projects can belong to any company in the CorporateGroup"]
    UserProjects,
    "user_projects",
    user_id: DocumentId,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    project_ids: Vec<DocumentId>,
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
    #[serde(skip_serializing_if="Option::is_none")]
    entity_id: Option<DocumentId>
);

// TODO: remote this collection
database_document!(
    #[doc = "Invite for the user in the app"]
    InviteAddCompany,
    "invite_add_company",
    inviting_user_id: DocumentId,
    invited_user_id: DocumentId,
    company_id: DocumentId,
    company_role: CompanyRole,
    job_title: String,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    project_ids: Vec<DocumentId>,
    #[serde(skip_serializing_if="Option::is_none")]
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

database_document!(
    #[doc = "Work Package inside a project to which a set of activities is associated"]
    WorkPackage,
    "work_package",
    project_id: DocumentId,
    name: String,
    description: String
);

database_document!(
    #[doc = "Define a type of activity that can be done in the Project."]
    #[doc  = "It is defined at Company level and can be associated to any Project."]
    #[doc = "The User specify it during the timesheet compilation."]
    ProjectActivity,
    "project_activity",
    name: String,
    description: String,
    corporate_group_id: DocumentId,
);

database_document!(
    #[doc = "Assigns the activity to the Project."]
    WPActivityAssignment,
    "wp_activity_assignment",
    work_package_id: DocumentId,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    activity_ids: Vec<DocumentId>
);

embedded_document!(
    #[doc = "Struct that contains a single activity hour amount inside a timesheet day"]
    #[doc = "It is a document inside the TimesheetDay."]
    TimesheetActivityHours,
    company_id: DocumentId,
    project_id: DocumentId,
    work_package_id: DocumentId,
    activity_id: DocumentId,
    notes: String,
    hours: u32
);

impl From<TimesheetActivityHours> for Bson {
    fn from(value: TimesheetActivityHours) -> Self {
        Bson::Document(doc! {
            "company_id": value.company_id,
            "project_id": value.project_id,
            "work_package_id": value.work_package_id,
            "activity_id": value.activity_id,
            "notes": value.notes,
            "hours": value.hours
        })
    }
}

impl From<internal::TimesheetActivityHours> for TimesheetActivityHours {
    fn from(value: internal::TimesheetActivityHours) -> Self {
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

database_document!(
    #[doc = "A Timesheet day of the User in the Company."]
    #[doc = "A day is composed of number of permit hours, the type, i.e., work at the office, remote or holiday and the list of activities."]
    #[doc = "An activity is specified by company id, project id, activity id and number of hours"]
    TimesheetDay,
    "timesheet_day",
    user_id: DocumentId,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    date: DateTime<Utc>,
    permit_hours: u32,
    working_type: WorkingDayType,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    activities: Vec<TimesheetActivityHours>
);

database_document!(
    #[doc = "Corporate Group groups together a set of Companies letting managers to have a global view."]
    #[doc = "Admins of each company are automatically admins of the group."]
    CorporateGroup,
    "corporate_group",
    name: String,
    active: bool,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    company_ids: Vec<DocumentId>,
);

database_document!(
    #[doc = "Object endpoint document that contains information to retrieve it."]
    #[doc = "Path must contain a prefix with the scheme that indicates the source."]
    #[doc = "Examples:"]
    #[doc = "- local filesystem: file:///folder/file.csv"]
    #[doc = "- Google Cloud Storage: gs://bucket/path/file.csv"]
    #[doc = "- AWS S3: s3://bucket/path/file.csv"]
    ObjectEndpoint,
    "object_endpoint",
    path: String,
    file_type: FileType,
    source_type: ObjectSourceType
);
