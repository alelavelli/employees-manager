use crate::{
    enums::{
        CompanyRole, EmployeeRequest, FileType, NotificationType, ObjectSourceType, WorkingDayType,
    },
    error::DatabaseError,
    service::db::DatabaseDocument,
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
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    project_ids: Vec<DocumentId>
);

database_document!(
    #[doc = "Management Team is a list of Company Employees that has special permissions"]
    CompanyManagementTeam,
    "company_management_team",
    company_id: DocumentId,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
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
    #[serde(skip_serializing_if="Option::is_none")]
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
    #[doc = "Define a type of activity that can be done in the Project."]
    #[doc  = "It is defined at Company level and can be associated to any Project."]
    #[doc = "The User specify it during the timesheet compilation."]
    ProjectActivity,
    "project_activity",
    name: String,
    description: String,
    company_id: DocumentId
);

database_document!(
    #[doc = "Assigns the activity to the Project."]
    ProjectActivityAssignment,
    "project_activity_assignment",
    project_id: DocumentId,
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    activity_ids: Vec<DocumentId>
);

embedded_document!(
    #[doc = "Struct that contains a single activity hour amount inside a timesheet day"]
    #[doc = "It is a document inside the TimesheetDay."]
    TimesheetActivityHours,
    company_id: DocumentId,
    project_id: DocumentId,
    activity_id: DocumentId,
    notes: String,
    hours: u32
);

impl From<TimesheetActivityHours> for Bson {
    fn from(value: TimesheetActivityHours) -> Self {
        Bson::Document(doc! {
            "company_id": value.company_id,
            "project_id": value.project_id,
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
    #[serde(skip_serializing_if="Vec::is_empty", default)]
    company_ids: Vec<DocumentId>,
    #[doc = "The user that created the corporate group"]
    owner: DocumentId
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

#[cfg(test)]
mod tests {
    use bson::oid::ObjectId;

    use crate::{
        model::db_entities::CorporateGroup,
        service::db::{get_database_service, DatabaseDocument},
    };

    #[tokio::test]
    async fn test_create_corporate_group() {
        let mut first_group =
            CorporateGroup::new("ciao".to_string(), vec![ObjectId::new()], ObjectId::new());
        first_group.save(None).await.unwrap();
        first_group.reload().await.unwrap();
        println!("{:?}", first_group);
        let mut second_group = CorporateGroup::new("ciao".to_string(), vec![], ObjectId::new());
        second_group.save(None).await.unwrap();
        second_group.reload().await.unwrap();
        println!("{:?}", second_group);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
