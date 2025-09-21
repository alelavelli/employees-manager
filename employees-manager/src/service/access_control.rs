use bson::doc;

use crate::{
    auth::AuthInfo,
    enums::{CompanyRole, CorporateGroupRole},
    error::{AppError, ServiceAppError},
    model::db_entities,
    service::{
        db::document::{DatabaseDocument, SmartDocumentReference},
        user::UserService,
    },
};

/// Access control struct that validate and verify the
/// role of the user
pub struct AccessControl<T: AuthInfo> {
    auth_info: T,
    user_service: UserService,
}

impl<T: AuthInfo> AccessControl<T> {
    pub async fn new(auth_info: T) -> Result<AccessControl<T>, AppError> {
        let user_service =
            UserService::new(SmartDocumentReference::Id(auth_info.user_id().clone()));
        let user = user_service.get().await.map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })?;
        if *user.active() {
            Ok(AccessControl {
                auth_info,
                user_service,
            })
        } else {
            Err(AppError::AccessControlError(
                "You are not allowed to do this operation".into(),
            ))
        }
    }

    pub fn auth_info(&self) -> &T {
        &self.auth_info
    }

    /// Verify that the user has ADMIN role, otherwise it
    /// returns AccessControlError
    pub async fn is_platform_admin(self) -> Result<Self, AppError> {
        let user = self.user_service.get().await.map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })?;
        if *user.platform_admin() {
            Ok(self)
        } else {
            Err(AppError::AccessControlError(
                "You are not allowed to do this operation".into(),
            ))
        }
    }

    /// Verifies that the user has the right permissions to operate on the Corporate Group.
    ///
    /// When must_be_active parameter is true then Err is returned when the corporate group is not.
    /// This forces the users to operate only on active corporate groups when specified
    pub async fn has_corporate_group_role_or_higher(
        self,
        corporate_group_id: &SmartDocumentReference<db_entities::CorporateGroup>,
        role: CorporateGroupRole,
        must_be_active: bool,
    ) -> Result<Self, AppError> {
        let entry = db_entities::UserCorporateGroupRole::find_one(
            doc! {"user_id": self.auth_info.user_id(),
            "corporate_group_id": corporate_group_id.as_ref_id()},
        )
        .await
        .map_err(|_| {
            AppError::AccessControlError("You are not allowed to perform this operation.".into())
        })?;
        if let Some(cg_role) = entry {
            let corporate_group = corporate_group_id
                .clone()
                .to_document()
                .await
                .map_err(|_| {
                    AppError::AccessControlError(
                        "You are not allowed to perform this operation".into(),
                    )
                })?;
            if (*cg_role.role() >= role) && (!must_be_active || *corporate_group.active()) {
                Ok(self)
            } else {
                Err(AppError::AccessControlError(
                    "You are not allowed to perform this operation.".into(),
                ))
            }
        } else {
            Err(AppError::AccessControlError(
                "You are not allowed to perform this operation.".into(),
            ))
        }
    }

    /// Verify that the user has the role indicated by the parameter
    ///
    /// When must_be_active parameter is true then Err is returned when the company is not.
    /// This forces the users to operate only on active corporate groups when specified.
    pub async fn has_company_role_or_higher(
        self,
        company_id: &SmartDocumentReference<db_entities::Company>,
        role: CompanyRole,
        must_be_active: bool,
    ) -> Result<Self, AppError> {
        let entry = db_entities::UserCompanyRole::find_one(doc! {"_id": self.auth_info.user_id(),
        "company_id": company_id.as_ref_id()})
        .await
        .map_err(|_| {
            AppError::AccessControlError("You are not allowed to perform this operation.".into())
        })?;
        if let Some(cmp_role) = entry {
            let company = company_id.clone().to_document().await.map_err(|_| {
                AppError::AccessControlError(
                    "You are not allowed to perform this operation.".into(),
                )
            })?;

            if (*cmp_role.role() >= role) && (!must_be_active || *company.active()) {
                Ok(self)
            } else {
                Err(AppError::AccessControlError(
                    "You are not allowed to perform this operation.".into(),
                ))
            }
        } else {
            Err(AppError::AccessControlError(
                "You are not allowed to perform this operation.".into(),
            ))
        }
    }
}

/*

#[cfg(test)]
mod tests {
    use crate::{
        auth::APIKeyAuthClaim,
        model::db_entities,
        service::{
            access_control::AccessControl,
            db::{document::DatabaseDocument, get_database_service},
        },
    };

    #[tokio::test]
    async fn has_company_role_or_higher_test() {
        let mut user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            true,
        );
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        let mut company = db_entities::Company::new("Company".into(), true);
        company.save(None).await.unwrap();

        for (role, expected) in vec![
            (crate::enums::CompanyRole::User, false),
            (crate::enums::CompanyRole::Admin, true),
            (crate::enums::CompanyRole::Owner, true),
        ] {
            let mut company_assignment = db_entities::UserEmploymentContract::new(
                user_id.clone(),
                company.get_id().unwrap().clone(),
                role,
                "CEO".into(),
                vec![],
            );
            company_assignment.save(None).await.unwrap();

            let auth_info = AccessControl {
                auth_info: &APIKeyAuthClaim {
                    key: "api_key".into(),
                    user_id: user_id.clone(),
                },
            };

            let access_control = auth_info
                .has_company_role_or_higher(
                    &company.get_id().unwrap(),
                    crate::enums::CompanyRole::Admin,
                )
                .await;
            if expected {
                assert!(access_control.is_ok());
            } else {
                assert!(access_control.is_err());
            }

            company_assignment.delete(None).await.unwrap();
        }

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
*/
