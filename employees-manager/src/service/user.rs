use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AuthError, ServiceAppError},
    model::db_entities,
    service::db::document::SmartDocumentReference,
    DocumentId,
};

use super::db::document::DatabaseDocument;

/// Struct containing user information and performing operations for it
pub struct UserService {
    user_id: SmartDocumentReference<db_entities::User>,
}

impl UserService {
    pub fn new(user_id: SmartDocumentReference<db_entities::User>) -> UserService {
        UserService { user_id }
    }

    pub async fn login(username: &str, password: &str) -> Result<db_entities::User, AppError> {
        let query_result: Option<db_entities::User> =
            db_entities::User::find_one(doc! {"username": username})
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        if let Some(user_document) = query_result {
            if bcrypt::verify(password, user_document.password_hash()).map_err(|e| {
                AppError::InternalServerError(format!(
                    "Error in password hash verification. Got {e}"
                ))
            })? {
                if *user_document.active() {
                    Ok(user_document)
                } else {
                    Err(AuthError::WrongCredentials)?
                }
            } else {
                Err(AuthError::WrongCredentials)?
            }
        } else {
            Err(AuthError::WrongCredentials)?
        }
    }

    /// Create new user in database and returns his identifier
    /// Attribute `username` is unique therefore, before creating a user we verify it
    ///
    /// It is a struct function and not method because we are creating the user
    pub async fn create(
        username: String,
        password: String,
        email: String,
        name: String,
        surname: String,
    ) -> Result<String, ServiceAppError> {
        #[derive(Serialize, Deserialize, Debug)]
        struct QueryResult {
            username: String,
            email: String,
        }

        let usernames = db_entities::User::find_many_projection::<QueryResult>(
            doc! {},
            doc! {"username": 1, "email": 1},
        )
        .await?;

        for document in usernames {
            if username.to_lowercase().trim() == document.username.to_lowercase().trim() {
                return Err(ServiceAppError::InvalidRequest(format!(
                    "Username {} already exists.",
                    username
                )));
            }
            if email.to_lowercase().trim() == document.email.to_lowercase().trim() {
                return Err(ServiceAppError::InvalidRequest(format!(
                    "Email {} already exists.",
                    email
                )));
            }
        }
        let mut user_model = db_entities::User::new(
            email.trim().into(),
            username.trim().into(),
            hash_password(&password)?,
            name.trim().into(),
            surname.trim().into(),
            None,
            false,
            true,
        );

        user_model.save().await
    }

    pub async fn get(&self) -> Result<db_entities::User, ServiceAppError> {
        let query_result: Option<db_entities::User> =
            db_entities::User::find_one(doc! {"_id": self.user_id.as_ref_id()}).await?;
        if let Some(user_document) = query_result {
            Ok(user_document)
        } else {
            Err(ServiceAppError::EntityDoesNotExist(format!(
                "User with id {} does not exist",
                self.user_id
            )))
        }
    }

    /// Delete user from the database.
    ///
    /// Any entity that is related with the user will be deleted:
    /// - User
    /// - UserCorporateGroupRole
    /// - UserEmploymentContract
    /// - UserCompanyRole
    /// - UserProjects
    /// - CompanyEmployeeRequest
    /// - AppNotification
    /// - InviteAddCompany
    /// - TimesheetDay
    ///
    /// This operation is not reversible.
    pub async fn delete(&self) -> Result<(), ServiceAppError> {
        let user = db_entities::User::find_one(doc! {"_id": self.user_id.as_ref_id()}).await?;
        if let Some(user) = user {
            // Delete user document
            user.delete().await?;

            // Delete user corporate group role
            db_entities::UserCorporateGroupRole::delete_many(
                doc! {"user_id": self.user_id.as_ref_id()},
            )
            .await?;

            // Delete UserEmploymentContract
            db_entities::UserEmploymentContract::delete_many(
                doc! {"user_id": self.user_id.as_ref_id()},
            )
            .await?;

            // Delete UserCompanyRole
            db_entities::UserCompanyRole::delete_many(doc! {"user_id": self.user_id.as_ref_id()})
                .await?;

            // Delete UserProjects
            db_entities::UserProjects::delete_many(doc! {"user_id": self.user_id.as_ref_id()})
                .await?;

            // Delete CompanyEmploymentContract
            db_entities::UserEmploymentContract::delete_many(
                doc! {"user_id": self.user_id.as_ref_id()},
            )
            .await?;

            // Delete AppNotification
            db_entities::AppNotification::delete_many(doc! {"user_id": self.user_id.as_ref_id()})
                .await?;

            // Delete InviteAddCompany
            db_entities::InviteAddCompany::delete_many(doc! {"user_id": self.user_id.as_ref_id()})
                .await?;

            // Delete TimesheetDay
            db_entities::TimesheetDay::delete_many(doc! {"user_id": self.user_id.as_ref_id()})
                .await?;

            Ok(())
        } else {
            Err(ServiceAppError::EntityDoesNotExist(format!(
                "User with id {} does not exist",
                self.user_id
            )))
        }
    }

    pub async fn set_platform_admin(&self) -> Result<(), ServiceAppError> {
        db_entities::User::update_one(
            doc! {"_id": self.user_id.as_ref_id()},
            doc! {"$set": doc! { "platform_admin": true }},
        )
        .await
    }

    pub async fn unset_platform_admin(&self) -> Result<(), ServiceAppError> {
        db_entities::User::update_one(
            doc! {"_id": self.user_id.as_ref_id()},
            doc! {"$set": doc! { "platform_admin": false }},
        )
        .await
    }

    /// Activate user
    pub async fn activate(&self) -> Result<(), ServiceAppError> {
        #[derive(Serialize, Deserialize, Debug)]
        struct UserQueryResult {
            active: bool,
        }
        let user = db_entities::User::find_one_projection::<UserQueryResult>(
            doc! {"_id": self.user_id.as_ref_id()},
            doc! { "active": 1 },
        )
        .await?;
        if user.is_some() {
            db_entities::User::update_one(
                doc! {"_id": self.user_id.as_ref_id()},
                doc! { "$set": {"active": true} },
            )
            .await?;
            Ok(())
        } else {
            Err(ServiceAppError::EntityDoesNotExist(format!(
                "User with id {} does not exist",
                self.user_id
            )))
        }
    }

    /// Deactivate user
    pub async fn deactivate(&self) -> Result<(), ServiceAppError> {
        #[derive(Serialize, Deserialize, Debug)]
        struct UserQueryResult {
            active: bool,
        }
        let user = db_entities::User::find_one_projection::<UserQueryResult>(
            doc! {"_id": self.user_id.as_ref_id()},
            doc! { "active": 1 },
        )
        .await?;
        if user.is_some() {
            db_entities::User::update_one(
                doc! {"_id": self.user_id.as_ref_id()},
                doc! { "$set": {"active": false} },
            )
            .await?;
            Ok(())
        } else {
            Err(ServiceAppError::EntityDoesNotExist(format!(
                "User with id {} does not exist",
                self.user_id
            )))
        }
    }

    pub async fn update(
        &self,
        email: Option<String>,
        password: Option<String>,
        name: Option<String>,
        surname: Option<String>,
    ) -> Result<(), ServiceAppError> {
        let user = db_entities::User::find_one(doc! {"_id": self.user_id.as_ref_id()}).await?;
        if user.is_some() {
            let mut update = doc! {};
            if let Some(email_str) = email {
                update.insert("email", email_str);
            }
            if let Some(password_str) = password {
                update.insert("password_hash", hash_password(&password_str)?);
            }
            if let Some(name_str) = name {
                update.insert("name", name_str);
            }
            if let Some(surname_str) = surname {
                update.insert("surname", surname_str);
            }
            let update = doc! {"$set": update};
            db_entities::User::update_one(doc! {"_id": self.user_id.as_ref_id()}, update).await
        } else {
            Err(ServiceAppError::EntityDoesNotExist(format!(
                "User with id {} does not exist",
                self.user_id
            )))
        }
    }

    /// Returns the list of the companies for which the user belongs to
    ///
    /// We know this by looking at the company roles because in a corporate group
    /// the user has only one contract with a company but belongs to all of them
    pub async fn get_companies(&self) -> Result<Vec<db_entities::Company>, ServiceAppError> {
        let company_ids: Vec<DocumentId> =
            db_entities::UserCompanyRole::find_many(doc! { "user_id": self.user_id.as_ref_id() })
                .await?
                .into_iter()
                .map(|doc| *doc.company_id())
                .collect();

        let companies =
            db_entities::Company::find_many(doc! { "_id": {"$in": company_ids}}).await?;

        Ok(companies)
    }
}

fn hash_password(password: &str) -> Result<String, ServiceAppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| {
        ServiceAppError::InternalServerError(format!("Error in hashing password. Got {e}"))
    })
}

pub async fn get_company_project_of_user(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<Vec<DocumentId>, ServiceAppError> {
    todo!();
    /*
    if let Some(doc) = db_entities::UserEmploymentContract::find_one(
        doc! {"user_id": user_id, "company_id": company_id},
    )
    .await?
    {
        Ok(doc.project_ids().clone())
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "User with id {user_id} is not in company with id {company_id}"
        )))
    }
    */
}

/*
#[cfg(test)]
mod tests {

    use mongodb::bson::doc;

    use crate::{
        model::db_entities,
        service::{
            db::{document::DatabaseDocument, get_database_service},
            user::{
                activate_user, create_user, delete_user, hash_password, set_platform_admin,
                unset_platform_admin, update_user,
            },
        },
    };

    use super::{deactivate_user, login};

    #[tokio::test]
    async fn create_user_test() {
        let username = "johnsmith".into();
        let password = "dfsf".into();
        let name = "John".into();
        let surname = "Smith".into();
        let email = "john@smith.com".into();
        let created_user_result = create_user(username, password, email, name, surname).await;
        assert!(created_user_result.is_ok());

        let username = "johnsmith".into();
        let password = "ollol".into();
        let name = "John".into();
        let surname = "Smith".into();
        let email = "john@smith.com".into();
        let created_user_result = create_user(username, password, email, name, surname).await;
        assert!(created_user_result.is_err());
        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn update_user_test() {
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

        let new_name: String = "Alfredo".into();
        let new_surname: String = "Mini".into();
        let updated_result = update_user(
            &user_id,
            None,
            None,
            Some(new_name.clone()),
            Some(new_surname.clone()),
        )
        .await;
        assert!(updated_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap().unwrap();
        assert_eq!(*loaded_user.name(), new_name);
        assert_eq!(*loaded_user.surname(), new_surname);
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn delete_user_test() {
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
        let deleted_user_result = delete_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_none());
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn deactivate_user_test() {
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
        let deleted_user_result = deactivate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| !user.active()));
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    /*
    #[tokio::test]
    async fn deactivate_user_with_company_test() {
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
        let company_id = company.get_id().unwrap();

        let mut user_company_assignment = db_entities::UserEmploymentContract::new(
            user_id.clone(),
            company_id.clone(),
            crate::enums::CompanyRole::Owner,
            "CEO".into(),
            vec![],
        );
        user_company_assignment.save(None).await.unwrap();

        let deleted_user_result = deactivate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| !user.active()));

        let collection =
            db.collection::<db_entities::Company>(db_entities::Company::collection_name());
        let filter = doc! {"_id": company_id};
        let loaded_company = collection.find_one(filter).await.unwrap();
        assert!(loaded_company.is_some_and(|company| !company.active()));

        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn activate_user_test() {
        let mut user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            false,
        );
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();
        let deleted_user_result = activate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| *user.active()));
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn activate_user_with_company_test() {
        let mut user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            false,
        );
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        let mut company = db_entities::Company::new("Company".into(), false);
        company.save(None).await.unwrap();
        let company_id = company.get_id().unwrap();

        let mut user_company_assignment = db_entities::UserEmploymentContract::new(
            user_id.clone(),
            company_id.clone(),
            crate::enums::CompanyRole::Owner,
            "CEO".into(),
            vec![],
        );

        user_company_assignment.save(None).await.unwrap();

        let deleted_user_result = activate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| *user.active()));

        let collection =
            db.collection::<db_entities::Company>(db_entities::Company::collection_name());
        let filter = doc! {"_id": company_id};
        let loaded_company = collection.find_one(filter).await.unwrap();
        assert!(loaded_company.is_some_and(|company| *company.active()));

        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn set_platform_admin_test() {
        // prepare the test by creating a User who is not admin
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

        set_platform_admin(&user_id).await.unwrap();

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap().unwrap();
        assert!(loaded_user.platform_admin());
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn unset_platform_admin_test() {
        // prepare the test by creating a User who is not admin
        let mut user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            true,
            true,
        );
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        unset_platform_admin(&user_id).await.unwrap();

        let db = &get_database_service().await.db;
        user.reload().await.unwrap();

        assert!(!user.platform_admin());
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn login_test() {
        let username = "John";
        let password = "Smith".into();
        let name = "John".into();
        let surname = "Smith".into();
        let email = "john@smith.com".into();

        // No users
        let result = login(username, password).await;
        assert!(result.is_err());

        // Add users and retrieve them
        let mut user = db_entities::User::new(
            email,
            username.into(),
            hash_password(password).unwrap(),
            name,
            surname,
            None,
            false,
            true,
        );
        user.save(None).await.unwrap();

        // Remake the query
        let result = login(username, password).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(username, user.username());
        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
    */
}
 */
