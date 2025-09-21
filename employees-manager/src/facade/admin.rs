use crate::{
    auth::AuthInfo,
    dtos::{
        web_app_request,
        web_app_response::{self},
    },
    error::{AppError, ServiceAppError},
    model::db_entities,
    service::{
        access_control::AccessControl,
        admin,
        corporate_group::CorporateGroupService,
        db::document::{DatabaseDocument, SmartDocumentReference},
        user::UserService,
    },
    DocumentId,
};

/// Struct containing information of the logged user and
/// that perform access control during initialization
pub struct AdminFacade<T>
where
    T: AuthInfo,
{
    access_control: AccessControl<T>,
}

impl<T: AuthInfo> AdminFacade<T> {
    /// Creates a AdminFacade object verifying that the user has the required permissions.
    ///
    /// The user needs to be Platform Administrator
    pub async fn new(auth_info: T) -> Result<AdminFacade<T>, AppError> {
        let access_control = AccessControl::new(auth_info)
            .await?
            .is_platform_admin()
            .await?;
        Ok(AdminFacade { access_control })
    }

    pub async fn create_corporate_group(&self, name: String) -> Result<(), AppError> {
        // Do not provide the user id because we are creating the corporate group
        // from the admin panel and we don't want that this user will become the owner
        CorporateGroupService::create_corporate_group(None, name)
            .await
            .map_err(|e| match e {
                ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }

    pub async fn get_admin_panel_overview(
        &self,
    ) -> Result<web_app_response::admin_panel::AdminPanelOverview, AppError> {
        let users_info = admin::get_admin_panel_overview_users_info()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let companies_info = admin::get_admin_panel_overview_companies_info()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(web_app_response::admin_panel::AdminPanelOverview::from((
            users_info,
            companies_info,
        )))
    }

    pub async fn get_admin_panel_users_info(
        &self,
    ) -> Result<Vec<web_app_response::admin_panel::AdminPanelUserInfo>, AppError> {
        admin::get_admin_panel_users_info()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
            .map(|info| info.into_iter().map(|user_info| user_info.into()).collect())
    }

    pub async fn get_admin_panel_corporate_groups_info(
        &self,
    ) -> Result<Vec<web_app_response::admin_panel::AdminPanelCorporateGroupInfo>, AppError> {
        admin::get_admin_corporate_groups_info()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
            .map(|doc_vec| {
                doc_vec
                    .into_iter()
                    .map(
                        |doc| web_app_response::admin_panel::AdminPanelCorporateGroupInfo {
                            name: doc.name().into(),
                            id: doc
                                .get_id()
                                .expect("Document id must be present after a query")
                                .to_string(),
                            active: *doc.active(),
                        },
                    )
                    .collect()
            })
    }

    /// Set the user as corporate group owner
    ///
    /// Since this operation can happen at the creation of
    /// the corporate group, is it possible that the user is
    /// not inside the corporate group.
    ///
    /// If not, then he is added and the role created.
    /// If an owner is already present then he is changed to Admin
    pub async fn set_corporate_group_owner(
        &self,
        corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
        user_id: SmartDocumentReference<db_entities::User>,
    ) -> Result<(), AppError> {
        admin::set_corporate_group_owner(&corporate_group_id, &user_id)
            .await
            .map_err(|e| match e {
                ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }

    pub async fn set_platform_admin(&self, user_id: DocumentId) -> Result<(), AppError> {
        if *self.access_control.auth_info().user_id() == user_id {
            return Err(AppError::InvalidRequest(
                "You cannot set yourself as platform admin".into(),
            ));
        }

        UserService::new(SmartDocumentReference::Id(user_id))
            .set_platform_admin()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    pub async fn unset_platform_admin(&self, user_id: DocumentId) -> Result<(), AppError> {
        if *self.access_control.auth_info().user_id() == user_id {
            return Err(AppError::InvalidRequest(
                "You cannot unset yourself as platform admin".into(),
            ));
        }

        UserService::new(SmartDocumentReference::Id(user_id))
            .unset_platform_admin()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    pub async fn activate_user(&self, user_id: DocumentId) -> Result<(), AppError> {
        if *self.access_control.auth_info().user_id() == user_id {
            return Err(AppError::InvalidRequest(
                "You cannot activate yourself".into(),
            ));
        }

        UserService::new(SmartDocumentReference::Id(user_id))
            .activate()
            .await
            .map_err(|e| match e {
                ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }

    pub async fn deactivate_user(&self, user_id: DocumentId) -> Result<(), AppError> {
        if *self.access_control.auth_info().user_id() == user_id {
            return Err(AppError::InvalidRequest(
                "You cannot deactivate yourself".into(),
            ));
        }

        UserService::new(SmartDocumentReference::Id(user_id))
            .deactivate()
            .await
            .map_err(|e| match e {
                ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
                ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }

    pub async fn set_user_password(
        &self,
        user_id: DocumentId,
        password: String,
    ) -> Result<(), AppError> {
        if *self.access_control.auth_info().user_id() == user_id {
            return Err(AppError::InvalidRequest(
                "You cannot set password of yourself via this method. You must use reset password."
                    .into(),
            ));
        }

        UserService::new(SmartDocumentReference::Id(user_id))
            .update(None, Some(password), None, None)
            .await
            .map_err(|e| match e {
                ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }

    pub async fn get_user(&self, user_id: DocumentId) -> Result<web_app_response::User, AppError> {
        let user_model = UserService::new(SmartDocumentReference::Id(user_id))
            .get()
            .await
            .map_err(|e| match e {
                ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                _ => AppError::InternalServerError(e.to_string()),
            })?;

        web_app_response::User::try_from(user_model).map_err(|_| {
            AppError::InternalServerError(
                "Error in building the response from User document".into(),
            )
        })
    }

    pub async fn create_user(
        &self,
        payload: web_app_request::CreateUser,
    ) -> Result<String, AppError> {
        UserService::create(
            payload.username,
            payload.password,
            payload.email,
            payload.name,
            payload.surname,
        )
        .await
        .map_err(|e| match e {
            ServiceAppError::InvalidRequest(message) => AppError::InvalidRequest(message),
            _ => AppError::InternalServerError(e.to_string()),
        })
    }

    pub async fn delete_user(&self, user_id: DocumentId) -> Result<(), AppError> {
        if *self.access_control.auth_info().user_id() == user_id {
            return Err(AppError::InvalidRequest(
                "You cannot delete yourself".into(),
            ));
        }
        UserService::new(SmartDocumentReference::Id(user_id))
            .delete()
            .await
            .map_err(|e| match e {
                ServiceAppError::EntityDoesNotExist(message) => AppError::DoesNotExist(message),
                _ => AppError::InternalServerError(e.to_string()),
            })
    }

    pub async fn activate_corporate_group(
        &self,
        corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
    ) -> Result<(), AppError> {
        CorporateGroupService::new(corporate_group_id)
            .activate()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    pub async fn deactivate_corporate_group(
        &self,
        corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
    ) -> Result<(), AppError> {
        CorporateGroupService::new(corporate_group_id)
            .deactivate()
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    }
}
