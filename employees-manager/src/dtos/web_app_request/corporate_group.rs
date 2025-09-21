use serde::Deserialize;

use crate::{enums::CorporateGroupRole, DocumentId};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCorporateGroup {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditCorporateGroup {
    pub name: String,
    pub company_ids: Vec<DocumentId>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmploymentContractRequest {
    pub company_id: DocumentId,
    pub job_title: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUserToCorporateGroup {
    pub role: CorporateGroupRole,
    pub employment_contract: Option<EmploymentContractRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserInCorporateGroup {
    pub role: Option<CorporateGroupRole>,
    pub employment_contract: Option<EmploymentContractRequest>,
}
