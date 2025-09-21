use bson::doc;

use crate::{
    auth::AuthInfo,
    dtos::web_app_response,
    error::AppError,
    model::db_entities,
    service::{
        access_control::AccessControl,
        db::document::{DatabaseDocument, SmartDocumentReference},
        user::UserService,
    },
    DocumentId,
};

/// Facade with operations for generic user that do not require any specific
/// permission over other entities
pub struct UserFacade<T>
where
    T: AuthInfo,
{
    user_id: SmartDocumentReference<db_entities::User>,
    access_control: AccessControl<T>,
}

impl<T: AuthInfo> UserFacade<T> {
    /// Create a new UserFacade object verifying that is active
    pub async fn new(
        user_id: SmartDocumentReference<db_entities::User>,
        auth_info: T,
    ) -> Result<UserFacade<T>, AppError> {
        let access_control = AccessControl::new(auth_info).await?;

        Ok(UserFacade {
            user_id,
            access_control,
        })
    }

    pub async fn get_user_data(&self) -> Result<web_app_response::AuthUserData, AppError> {
        let user_model = self
            .user_id
            .clone()
            .to_document()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        web_app_response::AuthUserData::try_from(user_model).map_err(|_| {
            AppError::InternalServerError("Error in building response from User document".into())
        })
    }

    pub async fn get_corporate_groups(
        &self,
    ) -> Result<Vec<web_app_response::corporate_group::CorporateGroupInfo>, AppError> {
        let corporate_groups_roles = db_entities::UserCorporateGroupRole::find_many(
            doc! {"user_id": self.user_id.as_ref_id()},
        )
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut result = vec![];

        let corporate_groups = db_entities::CorporateGroup::find_many(
        doc! {
            "_id": corporate_groups_roles.iter().map(|doc| doc.corporate_group_id()).collect::<Vec<&DocumentId>>()
    },
    )
    .await
    .map_err(|e| AppError::InternalServerError(format!("Got error {e}")))?;

        for group in corporate_groups.into_iter() {
            let group_id = group
                .get_id()
                .expect("Document id must exist after a query");

            /* let company_names_mapping = company::get_company_names(group.company_ids())
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

            let mut company_names: Vec<String> = vec![];
            for company_id in group.company_ids() {
                if let Some(name) = company_names_mapping.get(company_id) {
                    company_names.push(name.into());
                } else {
                    return Err(AppError::InternalServerError(format!("Missing company name entry in company name hashmap for company with id {company_id}")));
                }
            }

            let company_names = group
                .company_ids()
                .iter()
                .map(|elem| company_names_mapping.get(elem).unwrap().into())
                .collect();

            result.push(web_app_response::corporate_group::CorporateGroupInfo {
                group_id: group_id.to_hex(),
                name: group.name().clone(),
                company_ids: group
                    .company_ids()
                    .iter()
                    .map(|elem| elem.to_hex())
                    .collect(),
                company_names,
            }) */
            todo!()
        }

        Ok(result)
    }

    pub async fn get_companies(&self) -> Result<Vec<web_app_response::CompanyInfo>, AppError> {
        let companies = UserService::new(self.user_id.clone())
            .get_companies()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let mut to_return = vec![];
        for doc in companies {
            let id = *doc
                .get_id()
                .expect("expecting document id since it has been loaded from db.");
            to_return.push(
                web_app_response::CompanyInfoBuilder::default()
                    .id(id.to_string())
                    .name(doc.name().clone())
                    .active(*doc.active())
                    .build()
                    .map_err(|_| {
                        AppError::InternalServerError(
                            "Error in building response for companies of user".into(),
                        )
                    })?,
            );
        }
        Ok(to_return)
    }
}
