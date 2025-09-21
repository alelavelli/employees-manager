use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateGroupInfo {
    pub group_id: String,
    pub name: String,
    pub company_ids: Vec<String>,
    pub company_names: Vec<String>,
}
