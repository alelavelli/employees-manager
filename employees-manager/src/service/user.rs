use anyhow::anyhow;
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    enums::CompanyRole,
    error::{AppError, AuthError},
    model::db_entities,
    DocumentId,
};

use super::db::{get_database_service, DatabaseDocument};

pub async fn login(username: &str, password: &str) -> Result<db_entities::User, AppError> {
    let query_result: Option<db_entities::User> =
        db_entities::User::find_one(doc! {"username": username}).await?;
    if let Some(user_document) = query_result {
        if bcrypt::verify(password, &user_document.password_hash).map_err(|e| {
            AppError::InternalServerError(anyhow!(format!(
                "Error in password hash verification. Got {e}"
            )))
        })? {
            Ok(user_document)
        } else {
            Err(AuthError::WrongCredentials)?
        }
    } else {
        Err(AuthError::WrongCredentials)?
    }
}

pub async fn get_user(user_id: &DocumentId) -> Result<db_entities::User, AppError> {
    let query_result: Option<db_entities::User> =
        db_entities::User::find_one(doc! {"_id": user_id}).await?;
    if let Some(user_document) = query_result {
        Ok(user_document)
    } else {
        Err(AppError::DoesNotExist(anyhow!(
            "User with id {user_id} does not exist"
        )))
    }
}

/// Create new user in database and returns his identifier
/// Attribute `username` is unique therefore, before creating a user we verify it
pub async fn create_user(
    username: String,
    password: String,
    email: String,
    name: String,
    surname: String,
) -> Result<String, AppError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        username: String,
    }

    let usernames = db_entities::User::find_many_projection::<db_entities::User, QueryResult>(
        doc! {},
        doc! {"username": 1},
    )
    .await?;

    for document in usernames {
        if username.to_lowercase() == document.username.to_lowercase() {
            return Err(AppError::ManagedError(
                "Username {username} already exist.".into(),
            ));
        }
    }
    let mut user_model = db_entities::User {
        id: None,
        username,
        password_hash: hash_password(&password)?,
        api_key: None,
        email,
        name,
        surname,
        // by default users are always not platform admin
        platform_admin: false,
        active: true,
    };
    user_model.save(None).await
}

/// Deactivate user
/// Instead of deleting permanently from the application, a deactivated user cannot perform any operation
/// but he still exist in the database and can be activated by admins.
/// Deactivating a user determine the deactivation of all companies for which he is owner.
/// It returns an error if the user is already not active
pub async fn deactivate_user(user_id: &DocumentId) -> Result<(), AppError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct UserQueryResult {
        active: bool,
    }
    let user = db_entities::User::find_one_projection::<db_entities::User, UserQueryResult>(
        doc! {"_id": user_id},
        doc! { "active": 1 },
    )
    .await?;
    if let Some(user) = user {
        if user.active {
            #[derive(Serialize, Deserialize, Debug)]
            struct QueryResult {
                company_id: ObjectId,
            }
            let companies: Vec<ObjectId> =
                db_entities::UserCompanyAssignment::find_many_projection::<
                    db_entities::UserCompanyAssignment,
                    QueryResult,
                >(
                    doc! {"user_id": user_id, "role": CompanyRole::Owner},
                    doc! {"company_id": 1},
                )
                .await?
                .iter()
                .map(|doc| doc.company_id)
                .collect::<Vec<ObjectId>>();

            if !companies.is_empty() {
                let db_service = get_database_service().await;
                let mut transaction = db_service.new_transaction().await?;
                transaction.start_transaction().await?;
                let result = db_entities::User::update_one(
                    doc! {"_id": user_id},
                    doc! { "$set": {"active": false} },
                    Some(&mut transaction),
                )
                .await;
                if result.is_err() {
                    transaction.abort_transaction().await?;
                    return Err(AppError::InternalServerError(anyhow!(
                        "Got an error during User update"
                    )));
                }
                let result = db_entities::Company::update_many(
                    doc! { "_id": {"$in": companies}},
                    doc! {"$set": {"active": false}},
                    Some(&mut transaction),
                )
                .await;
                if result.is_err() {
                    info!("Aborting transaction due to an error in Company update");
                    let result = transaction.abort_transaction().await;
                    println!("{:?}", result);
                    info!("Transaction aborted");
                    return Err(AppError::InternalServerError(anyhow!(
                        "Got an error during Company update"
                    )));
                }
                transaction.commit_transaction().await?;
            } else {
                // Since the user does not have companies with Owner role we do not create a transaction
                // and we just update it
                db_entities::User::update_one(
                    doc! {"_id": user_id},
                    doc! { "$set": {"active": false} },
                    None,
                )
                .await?;
            }
            Ok(())
        } else {
            Err(AppError::ManagedError(
                "The user with id {user_id} not active.".to_string(),
            ))
        }
    } else {
        Err(AppError::DoesNotExist(anyhow!(
            "User with id {user_id} does not exist"
        )))
    }
}

/// Activate user
///
/// Activate a deactivated User. It returns a ManagedError if the user is not active
pub async fn activate_user(user_id: &DocumentId) -> Result<(), AppError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct UserQueryResult {
        active: bool,
    }
    let user = db_entities::User::find_one_projection::<db_entities::User, UserQueryResult>(
        doc! {"_id": user_id},
        doc! { "active": 1 },
    )
    .await?;
    if let Some(user) = user {
        if !user.active {
            #[derive(Serialize, Deserialize, Debug)]
            struct QueryResult {
                company_id: ObjectId,
            }
            let companies: Vec<ObjectId> =
                db_entities::UserCompanyAssignment::find_many_projection::<
                    db_entities::UserCompanyAssignment,
                    QueryResult,
                >(
                    doc! {"user_id": user_id, "role": CompanyRole::Owner},
                    doc! {"company_id": 1},
                )
                .await?
                .iter()
                .map(|doc| doc.company_id)
                .collect::<Vec<ObjectId>>();

            if !companies.is_empty() {
                let db_service = get_database_service().await;
                let mut transaction = db_service.new_transaction().await?;
                transaction.start_transaction().await?;
                let result = db_entities::User::update_one(
                    doc! {"_id": user_id},
                    doc! { "$set": {"active": true} },
                    Some(&mut transaction),
                )
                .await;
                if result.is_err() {
                    transaction.abort_transaction().await?;
                    return Err(AppError::InternalServerError(anyhow!(
                        "Got an error during User update"
                    )));
                }
                let result = db_entities::Company::update_many(
                    doc! { "_id": {"$in": companies}},
                    doc! {"$set": {"active": true}},
                    Some(&mut transaction),
                )
                .await;
                if result.is_err() {
                    transaction.abort_transaction().await?;
                    return Err(AppError::InternalServerError(anyhow!(
                        "Got an error during Company update"
                    )));
                }
                transaction.commit_transaction().await?;
            } else {
                // Since the user does not have companies with Owner role we do not create a transaction
                // and we just update it
                db_entities::User::update_one(
                    doc! {"_id": user_id},
                    doc! { "$set": {"active": true} },
                    None,
                )
                .await?;
            }
            Ok(())
        } else {
            Err(AppError::ManagedError(
                "The user with id {user_id} active.".to_string(),
            ))
        }
    } else {
        Err(AppError::DoesNotExist(anyhow!(
            "User with id {user_id} does not exist"
        )))
    }
}

/// Delete user from the database.
///
/// Each Company the User is owner is deleted as well.
///
/// This operation is not reversible.
pub async fn delete_user(user_id: &DocumentId) -> Result<(), AppError> {
    let user = db_entities::User::find_one::<db_entities::User>(doc! {"_id": user_id}).await?;
    if let Some(user) = user {
        #[derive(Serialize, Deserialize, Debug)]
        struct QueryResult {
            company_id: ObjectId,
        }
        let companies: Vec<ObjectId> = db_entities::UserCompanyAssignment::find_many_projection::<
            db_entities::UserCompanyAssignment,
            QueryResult,
        >(
            doc! {"user_id": user_id, "role": CompanyRole::Owner},
            doc! {"company_id": 1},
        )
        .await?
        .iter()
        .map(|doc| doc.company_id)
        .collect::<Vec<ObjectId>>();

        if !companies.is_empty() {
            let db_service = get_database_service().await;
            let mut transaction = db_service.new_transaction().await?;
            transaction.start_transaction().await?;
            let result = user.delete(Some(&mut transaction)).await;
            if result.is_err() {
                transaction.abort_transaction().await?;
                return Err(AppError::InternalServerError(anyhow!(
                    "Got an error during User delete"
                )));
            }

            let result = db_entities::Company::delete_many(
                doc! { "_id": {"$in": &companies}},
                Some(&mut transaction),
            )
            .await;
            if result.is_err() {
                transaction.abort_transaction().await?;
                return Err(AppError::InternalServerError(anyhow!(
                    "Got an error during Company delete"
                )));
            }

            // We delete any UserCompanyAssignment for the deleted companies
            #[derive(Serialize, Deserialize, Debug)]
            struct QueryResult {
                user_id: ObjectId,
            }
            let other_assignments: Vec<ObjectId> =
                db_entities::UserCompanyAssignment::find_many_projection::<
                    db_entities::UserCompanyAssignment,
                    QueryResult,
                >(
                    doc! {"company_id": {"$in": &companies}},
                    doc! {"company_id": 1},
                )
                .await?
                .iter()
                .map(|doc| doc.user_id)
                .collect::<Vec<ObjectId>>();
            let result = db_entities::User::delete_many(
                doc! {"_id": {"$in": &other_assignments}},
                Some(&mut transaction),
            )
            .await;
            if result.is_err() {
                transaction.abort_transaction().await?;
                return Err(AppError::InternalServerError(anyhow!(
                    "Got an error during Company assignments delete"
                )));
            }

            transaction.commit_transaction().await?;
        } else {
            // Since the user does not have companies with Owner role we do not create a transaction
            // and we just delete it
            user.delete(None).await?;
        }
        Ok(())
    } else {
        Err(AppError::DoesNotExist(anyhow!(
            "User with id {user_id} does not exist"
        )))
    }
}

pub async fn update_user(
    user_id: &DocumentId,
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
    surname: Option<String>,
) -> Result<(), AppError> {
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
    db_entities::User::update_one(doc! {"_id": user_id}, update, None).await
}

pub async fn set_platform_admin(user_id: &DocumentId) -> Result<(), AppError> {
    db_entities::User::update_one(
        doc! {"_id": user_id},
        doc! {"$set": doc! { "platform_admin": true }},
        None,
    )
    .await
}

pub async fn unset_platform_admin(user_id: &DocumentId) -> Result<(), AppError> {
    db_entities::User::update_one(
        doc! {"_id": user_id},
        doc! {"$set": doc! { "platform_admin": false }},
        None,
    )
    .await
}

fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| {
        AppError::InternalServerError(anyhow!(format!("Error in hashing password. Got {e}")))
    })
}

#[cfg(test)]
mod tests {

    use mongodb::bson::doc;

    use crate::{
        model::db_entities,
        service::{
            db::{get_database_service, DatabaseDocument},
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
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: true,
        };
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
        assert_eq!(loaded_user.name, new_name);
        assert_eq!(loaded_user.surname, new_surname);
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn delete_user_test() {
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: true,
        };
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
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: true,
        };
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();
        let deleted_user_result = deactivate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| !user.active));
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn deactivate_user_with_company_test() {
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
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
        let company_id = company.get_id().unwrap();

        let mut user_company_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: user_id.clone(),
            company_id: company_id.clone(),
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        user_company_assignment.save(None).await.unwrap();

        let deleted_user_result = deactivate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| !user.active));

        let collection =
            db.collection::<db_entities::Company>(db_entities::Company::collection_name());
        let filter = doc! {"_id": company_id};
        let loaded_company = collection.find_one(filter).await.unwrap();
        assert!(loaded_company.is_some_and(|company| !company.active));

        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn activate_user_test() {
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: false,
        };
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();
        let deleted_user_result = activate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| user.active));
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn activate_user_with_company_test() {
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: false,
        };
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        let mut company = db_entities::Company {
            id: None,
            name: "Company".into(),
            active: false,
        };
        company.save(None).await.unwrap();
        let company_id = company.get_id().unwrap();

        let mut user_company_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: user_id.clone(),
            company_id: company_id.clone(),
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        user_company_assignment.save(None).await.unwrap();

        let deleted_user_result = activate_user(&user_id).await;
        assert!(deleted_user_result.is_ok());

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap();
        assert!(loaded_user.is_some_and(|user| user.active));

        let collection =
            db.collection::<db_entities::Company>(db_entities::Company::collection_name());
        let filter = doc! {"_id": company_id};
        let loaded_company = collection.find_one(filter).await.unwrap();
        assert!(loaded_company.is_some_and(|company| company.active));

        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn set_platform_admin_test() {
        // prepare the test by creating a User who is not admin
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: true,
        };
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        set_platform_admin(&user_id).await.unwrap();

        let db = &get_database_service().await.db;
        let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
        let filter = doc! {"_id": user_id};
        let loaded_user = collection.find_one(filter).await.unwrap().unwrap();
        assert!(loaded_user.platform_admin);
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn unset_platform_admin_test() {
        // prepare the test by creating a User who is not admin
        let mut user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: true,
            active: true,
        };
        user.save(None).await.unwrap();
        let user_id = user.get_id().unwrap();

        unset_platform_admin(&user_id).await.unwrap();

        let db = &get_database_service().await.db;
        user.reload().await.unwrap();

        assert!(!user.platform_admin);
        let drop_result = db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn login_test() {
        let username = "John";
        let password = "Smith";
        let name = "John".into();
        let surname = "Smith".into();
        let email = "john@smith.com".into();

        // No users
        let result = login(username, password).await;
        assert!(result.is_err());

        // Add users and retrieve them
        let mut user = db_entities::User {
            id: None,
            username: username.into(),
            password_hash: hash_password(password).unwrap(),
            api_key: None,
            name,
            email,
            surname,
            platform_admin: false,
            active: true,
        };
        user.save(None).await.unwrap();

        // Remake the query
        let result = login(username, password).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(username, user.username);
        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
