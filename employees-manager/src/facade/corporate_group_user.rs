use crate::dtos::web_app_response;
use crate::error::ServiceAppError;
//use crate::service::company;
use crate::service::db::document::DatabaseDocument;
use crate::{
    auth::AuthInfo,
    enums::CorporateGroupRole,
    error::AppError,
    model::db_entities,
    service::{access_control::AccessControl, db::document::SmartDocumentReference},
};

/// Facade with operations allowed by users with Corporate Group User role
///
/// Note that the corporate group is not forced to be active. Therefore, it is
/// important to apply this check manually when needed
pub struct CorporateGroupUserFacade<T>
where
    T: AuthInfo,
{
    corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
    access_control: AccessControl<T>,
}

impl<T: AuthInfo> CorporateGroupUserFacade<T> {
    /// Creates a CorporateGroupFacade verifying that the user has Admin role
    /// for the corporate group
    pub async fn new(
        corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
        auth_info: T,
    ) -> Result<CorporateGroupUserFacade<T>, AppError> {
        let access_control = AccessControl::new(auth_info)
            .await?
            .has_corporate_group_role_or_higher(
                &corporate_group_id,
                CorporateGroupRole::User,
                false,
            )
            .await?;

        Ok(CorporateGroupUserFacade {
            corporate_group_id,
            access_control,
        })
    }

    pub async fn get_corporate_group_info(
        &self,
    ) -> Result<web_app_response::corporate_group::CorporateGroupInfo, AppError> {
        let corporate_group_id_ref = self.corporate_group_id.as_ref_id().clone();
        let corporate_group = self
            .corporate_group_id
            .clone()
            .to_document()
            .await
            .map_err(|e| match e {
                ServiceAppError::EntityDoesNotExist(_) => AppError::DoesNotExist(format!(
                    "There is not corporate group with id {corporate_group_id_ref}"
                )),
                _ => AppError::InternalServerError(e.to_string()),
            })?;
        let company_ids = corporate_group.company_ids();
        todo!()
        /* let company_names_map = company::get_company_names(corporate_group.company_ids())
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        return Ok(web_app_response::corporate_group::CorporateGroupInfo {
            group_id: corporate_group
                .get_id()
                .expect("Document must have object id")
                .to_hex(),
            name: corporate_group.name().into(),
            company_ids: company_ids.iter().map(|elem| elem.to_hex()).collect(),
            company_names: company_ids
                .iter()
                .map(|elem| {
                    company_names_map
                        .get(elem)
                        .expect("Expecting to have company id entry from the database call.")
                        .clone()
                })
                .collect(),
        }); */
    }
}
