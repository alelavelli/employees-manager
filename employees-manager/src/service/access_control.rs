use crate::{
    auth::AuthInfo, enums::CompanyRole, error::AppError, service::user::get_user, DocumentId,
};

use super::company::get_user_company_role;

/// Access control struct that validate and verify the
/// role of the user
pub struct AccessControl<T: AuthInfo> {
    auth_info: T,
}

impl<T: AuthInfo> AccessControl<T> {
    pub async fn new(auth_info: T) -> Result<AccessControl<T>, AppError> {
        let user = get_user(auth_info.user_id()).await?;
        if user.active {
            Ok(AccessControl { auth_info })
        } else {
            Err(AppError::AccessControlError)
        }
    }

    /// Verify that the user has ADMIN role, otherwise it
    /// returns AccessControlError
    pub async fn is_platform_admin(self) -> Result<Self, AppError> {
        let user = get_user(self.auth_info.user_id()).await?;
        if user.platform_admin {
            Ok(self)
        } else {
            Err(AppError::AccessControlError)
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
            .map_err(|_| AppError::AccessControlError)?;
        if assignment.role >= role {
            Ok(self)
        } else {
            Err(AppError::AccessControlError)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::APIKeyAuthClaim,
        model::db_entities,
        service::{access_control::AccessControl, db::DatabaseDocument},
    };

    #[tokio::test]
    async fn has_company_role_or_higher_test() {
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: Some("api_key".into()),
            platform_admin: false,
            active: true,
        };
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        let mut company = db_entities::Company {
            id: None,
            name: "Company".into(),
            active: true,
        };
        company.save(None).await.unwrap();

        for (role, expected) in vec![
            (crate::enums::CompanyRole::User, false),
            (crate::enums::CompanyRole::Admin, true),
            (crate::enums::CompanyRole::Owner, true),
        ] {
            let mut company_assignment = db_entities::UserCompanyAssignment {
                id: None,
                user_id: user_id.clone(),
                company_id: company.get_id().unwrap().clone(),
                role,
                job_title: "CEO".into(),
            };
            company_assignment.save(None).await.unwrap();

            let auth_info = AccessControl {
                auth_info: APIKeyAuthClaim {
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
    }
}
