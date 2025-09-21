use crate::{
    auth::AuthInfo,
    dtos::web_app_request::{self, corporate_group::EmploymentContractRequest},
    enums::CorporateGroupRole,
    error::{AppError, ServiceAppError},
    model::db_entities,
    service::{
        access_control::AccessControl, corporate_group::CorporateGroupService,
        db::document::SmartDocumentReference,
    },
};

/// Facade with operations allowed by users with Corporate Group Admin role
///
/// Note that the corporate group is not forced to be active. Therefore, it is
/// important to apply this check manually when needed
pub struct CorporateGroupAdminFacade<T>
where
    T: AuthInfo,
{
    corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
    access_control: AccessControl<T>,
}

impl<T: AuthInfo> CorporateGroupAdminFacade<T> {
    /// Creates a CorporateGroupFacade verifying that the user has Admin role
    /// for the corporate group
    pub async fn new(
        corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
        auth_info: T,
    ) -> Result<CorporateGroupAdminFacade<T>, AppError> {
        let access_control = AccessControl::new(auth_info)
            .await?
            .has_corporate_group_role_or_higher(
                &corporate_group_id,
                CorporateGroupRole::Admin,
                false,
            )
            .await?;

        Ok(CorporateGroupAdminFacade {
            corporate_group_id,
            access_control,
        })
    }

    async fn is_active(&self) -> Result<bool, AppError> {
        Ok(*self
            .corporate_group_id
            .clone()
            .to_document()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .active())
    }

    pub async fn delete(&self) -> Result<(), AppError> {
        CorporateGroupService::new(self.corporate_group_id.clone())
            .delete()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    pub async fn update(
        &self,
        payload: web_app_request::corporate_group::EditCorporateGroup,
    ) -> Result<(), AppError> {
        if !self.is_active().await? {
            Err(AppError::InvalidRequest(format!(
                "Corporate Group with id {} must be active to be updated.",
                self.corporate_group_id.as_ref_id()
            )))
        } else {
            CorporateGroupService::new(self.corporate_group_id.clone())
                .update(
                    Some(payload.name),
                    Some(
                        payload
                            .company_ids
                            .into_iter()
                            .map(SmartDocumentReference::from)
                            .collect(),
                    ),
                )
                .await
                .map_err(|e| match e {
                    ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                    ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                    _ => AppError::InternalServerError(e.to_string()),
                })
        }
    }

    /// Add a user to the corporate group
    pub async fn add_user(
        &self,
        user_id: SmartDocumentReference<db_entities::User>,
        role: CorporateGroupRole,
        employment_contract: Option<EmploymentContractRequest>,
    ) -> Result<(), AppError> {
        if !self.is_active().await? {
            Err(AppError::InvalidRequest(format!(
                "Corporate Group with id {} must be active to be updated.",
                self.corporate_group_id.as_ref_id()
            )))
        } else {
            let (company_id, job_title) = if let Some(employment_contract) = employment_contract {
                (
                    Some(SmartDocumentReference::from(employment_contract.company_id)),
                    Some(employment_contract.job_title),
                )
            } else {
                (None, None)
            };

            CorporateGroupService::new(self.corporate_group_id.clone())
                .add_user(user_id, role, company_id, job_title)
                .await
                .map_err(|e| match e {
                    ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                    _ => AppError::InternalServerError(e.to_string()),
                })
        }
    }

    /// Remove a user from the corporate group
    pub async fn remove_user(
        &self,
        user_id: SmartDocumentReference<db_entities::User>,
    ) -> Result<(), AppError> {
        if !self.is_active().await? {
            Err(AppError::InvalidRequest(format!(
                "Corporate Group with id {} must be active to be updated.",
                self.corporate_group_id.as_ref_id()
            )))
        } else {
            CorporateGroupService::new(self.corporate_group_id.clone())
                .remove_user(user_id)
                .await
                .map_err(|e| match e {
                    ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                    _ => AppError::InternalServerError(e.to_string()),
                })
        }
    }

    pub async fn update_user(
        &self,
        user_id: SmartDocumentReference<db_entities::User>,
        role: Option<CorporateGroupRole>,
        employment_contract: Option<EmploymentContractRequest>,
    ) -> Result<(), AppError> {
        if !self.is_active().await? {
            Err(AppError::InvalidRequest(format!(
                "Corporate Group with id {} must be active to be updated.",
                self.corporate_group_id.as_ref_id()
            )))
        } else {
            let (company_id, job_title) = if let Some(employment_contract) = employment_contract {
                (
                    Some(SmartDocumentReference::from(employment_contract.company_id)),
                    Some(employment_contract.job_title),
                )
            } else {
                (None, None)
            };

            CorporateGroupService::new(self.corporate_group_id.clone())
                .update_user(user_id, role, company_id, job_title)
                .await
                .map_err(|e| match e {
                    ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                    _ => AppError::InternalServerError(e.to_string()),
                })
        }
    }
}
