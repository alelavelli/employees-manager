use serde::{Deserialize, Serialize};

use crate::{model::db_entities, DocumentId};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetActivityHours {
    pub company_id: DocumentId,
    pub project_id: DocumentId,
    pub activity_id: DocumentId,
    pub description: String,
    pub hours: u32,
}

impl From<&db_entities::TimesheetActivityHours> for TimesheetActivityHours {
    fn from(value: &db_entities::TimesheetActivityHours) -> Self {
        Self {
            company_id: *value.company_id(),
            project_id: *value.project_id(),
            activity_id: *value.activity_id(),
            description: value.description().into(),
            hours: *value.hours(),
        }
    }
}
