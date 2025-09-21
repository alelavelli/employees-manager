use crate::{
    auth::AuthInfo,
    enums::CompanyRole,
    error::{AppError, ServiceAppError},
    service::user::get_user,
    DocumentId,
};

use super::company::get_user_company_role;

/// Access control struct that validate and verify the
/// role of the user
pub struct AccessControl<'a, T: AuthInfo> {
    auth_info: &'a T,
}

impl<T: AuthInfo> AccessControl<'_, T> {
    pub async fn new(auth_info: &T) -> Result<AccessControl<T>, AppError> {
        let user = get_user(auth_info.user_id()).await.map_err(|e| match e {
            ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
            _ => AppError::InternalServerError(e.to_string()),
        })?;
        if *user.active() {
            Ok(AccessControl { auth_info })
        } else {
            Err(AppError::AccessControlError(
                "You are not allowed to do this operation".into(),
            ))
        }
    }

    /// Verify that the user has ADMIN role, otherwise it
    /// returns AccessControlError
    pub async fn is_platform_admin(self) -> Result<Self, AppError> {
        let user = get_user(self.auth_info.user_id())
            .await
            .map_err(|e| match e {
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

    /// Verify that the user has the role indicated by the parameter
    pub async fn has_company_role_or_higher(
        self,
        company_id: &DocumentId,
        role: CompanyRole,
    ) -> Result<Self, AppError> {
        let assignment = get_user_company_role(self.auth_info.user_id(), company_id)
            .await
            .map_err(|_| {
                AppError::AccessControlError("You are not allowed to do this operation".into())
            })?;
        if *assignment.role() >= role {
            Ok(self)
        } else {
            Err(AppError::AccessControlError(
                "You are not allowed to do this operation".into(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::APIKeyAuthClaim,
        model::db_entities,
        service::{
            access_control::AccessControl,
            db::{get_database_service, DatabaseDocument},
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
            let mut company_assignment = db_entities::UserCompanyAssignment::new(
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
