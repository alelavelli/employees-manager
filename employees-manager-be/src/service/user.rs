use anyhow::anyhow;
use futures::TryStreamExt;
use mongodb::{bson::doc, options::FindOptions};
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AuthError},
    model::db_entities,
    DocumentId,
};

use super::db::{get_database_service, DatabaseDocument};
use base64ct::{Base64, Encoding};

pub async fn login(username: &str, password: &str) -> Result<db_entities::User, AppError> {
    let db = &get_database_service().await.db;
    let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
    let hashed_password = hash_password(password);
    let filter = doc! {
        "username": username,
        "password_hash": hashed_password
    };
    let query_result = collection.find_one(filter).await?;
    if let Some(user_document) = query_result {
        Ok(user_document)
    } else {
        Err(AuthError::WrongCredentials)?
    }
}

pub async fn get_user(user_id: &DocumentId) -> Result<db_entities::User, AppError> {
    let db = &get_database_service().await.db;
    let collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
    let filter = doc! { "_id": user_id };
    let query_result = collection.find_one(filter).await?;
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
    // before to create the user we check if the username is unique or already present in the database
    // we create a temporary struct to get retrieve only user names from database
    let db = &get_database_service().await.db;
    let user_collection = db.collection::<db_entities::User>(db_entities::User::collection_name());
    let options = FindOptions::builder()
        .projection(doc! {"username": 1})
        .build();
    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        username: String,
    }
    let usernames: Vec<QueryResult> = user_collection
        .clone_with_type::<QueryResult>()
        .find(doc! {})
        .with_options(options)
        .await?
        .try_collect()
        .await?;
    for document in usernames {
        if username.to_lowercase() == document.username.to_lowercase() {
            return Err(AppError::ManagedError(
                "Username {username} already exist.".into(),
            ));
        }
    }
    let user_model = db_entities::User {
        id: None,
        username,
        password_hash: hash_password(&password),
        api_key: None,
        email,
        name,
        surname,
        // by default users are always not platform admin
        platform_admin: false,
    };
    user_model.save().await
}

/// Delete the user and all his the company assignments
pub async fn delete_user(user_id: &DocumentId) -> Result<(), AppError> {
    let db_service = get_database_service().await;
    db_service
        .delete_document::<db_entities::User>(user_id)
        .await?;
    let assignments_collection = db_service
        .db
        .collection::<db_entities::UserCompanyAssignment>(
            db_entities::UserCompanyAssignment::collection_name(),
        );

    let filter = doc! { "user_id": user_id};
    let query_result: Vec<db_entities::UserCompanyAssignment> = assignments_collection
        .find(filter)
        .await?
        .try_collect()
        .await?;
    for assignment_doc in query_result {
        db_service
            .delete_document::<db_entities::UserCompanyAssignment>(&assignment_doc.id.unwrap())
            .await?;
    }
    Ok(())
}

pub async fn update_user(
    user_id: &DocumentId,
    email: Option<String>,
    password: Option<String>,
    name: Option<String>,
    surname: Option<String>,
) -> Result<(), AppError> {
    let db_service = get_database_service().await;
    let mut update = doc! {};
    if let Some(email_str) = email {
        update.insert("email", email_str);
    }
    if let Some(password_str) = password {
        update.insert("password_hash", hash_password(&password_str));
    }
    if let Some(name_str) = name {
        update.insert("name", name_str);
    }
    if let Some(surname_str) = surname {
        update.insert("surname", surname_str);
    }
    let update = doc! {"$set": update};
    db_service
        .update_document::<db_entities::User>(user_id, update)
        .await
}

pub async fn set_platform_admin(user_id: &DocumentId) -> Result<(), AppError> {
    let db_service = get_database_service().await;
    db_service
        .update_document::<db_entities::User>(user_id, doc! {"$set": {"platform_admin": true}})
        .await
}

fn hash_password(password: &str) -> String {
    Base64::encode_string(password.as_bytes())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mongodb::bson::{doc, oid::ObjectId};

    use crate::{
        model::db_entities,
        service::{
            db::{get_database_service, DatabaseDocument},
            user::{create_user, delete_user, hash_password, set_platform_admin, update_user},
        },
    };

    use super::login;

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
        let user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
        };
        let user_id = ObjectId::from_str(&user.save().await.unwrap()).unwrap();

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
        let user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
        };
        let user_id = ObjectId::from_str(&user.save().await.unwrap()).unwrap();
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
    async fn set_platform_admin_test() {
        // prepare the test by creating a User who is not admin
        let user = db_entities::User {
            username: "johnsmith".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
        };
        let user_id = ObjectId::from_str(&user.save().await.unwrap()).unwrap();

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
        let user_id_result = db_entities::User {
            id: None,
            username: username.into(),
            password_hash: hash_password(password),
            api_key: None,
            name,
            email,
            surname,
            platform_admin: false,
        }
        .save()
        .await;
        assert!(user_id_result.is_ok());

        // Remake the query
        let result = login(username, password).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(username, user.username);
        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
