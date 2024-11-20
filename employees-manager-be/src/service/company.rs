use std::str::FromStr;

use anyhow::anyhow;
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Bson};

use super::db::{get_database_service, DatabaseDocument};
use crate::{
    enums::CompanyRole,
    error::AppError,
    model::db_entities::{self, UserCompanyAssignment},
    DocumentId,
};

/// creates a Company and assigns it to the User creating an entry
/// in UserCompanyAssignment
pub async fn create_company(
    user_id: &DocumentId,
    name: String,
    job_title: String,
) -> Result<String, AppError> {
    let company_model = db_entities::Company {
        id: None,
        name,
        active: true,
    };
    let company_id = company_model.save().await?;
    let company_id_object_id = ObjectId::from_str(&company_id);
    if company_id_object_id.is_err() {
        return Err(AppError::InternalServerError(anyhow!(
            "Unexpected failed conversion of ObjectId"
        )));
    }
    let user_company_assignment = db_entities::UserCompanyAssignment {
        id: None,
        user_id: *user_id,
        company_id: company_id_object_id.unwrap(),
        role: CompanyRole::Owner,
        job_title,
    };
    // If for some reasons we fail to dump the assignment we need to rollback
    if user_company_assignment.save().await.is_err() {
        company_model.delete().await?;
    }
    Ok(company_id)
}

/// Get all the Companies the User is in by looking at the UserCompanyAssignment
pub async fn get_user_companies(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::Company>, AppError> {
    let db = &get_database_service().await.db;
    let assignments_collection = db.collection::<db_entities::UserCompanyAssignment>(
        db_entities::UserCompanyAssignment::collection_name(),
    );

    let filter = doc! { "user_id": user_id};
    let query_result: Vec<UserCompanyAssignment> = assignments_collection
        .find(filter)
        .await?
        .try_collect()
        .await?;

    let mut company_ids = vec![];
    for doc in query_result {
        company_ids.push(Bson::ObjectId(doc.company_id));
    }
    if company_ids.is_empty() {
        return Err(AppError::DoesNotExist(anyhow!(
            "User with id {user_id} does not have Companies.",
        )));
    }

    let company_collection =
        db.collection::<db_entities::Company>(db_entities::Company::collection_name());
    let filter = doc! { "_id": {"$in": company_ids}};
    let query_result: Vec<db_entities::Company> =
        company_collection.find(filter).await?.try_collect().await?;
    Ok(query_result)
}

/// Verifies that the entry in UserCompanyAssignment exists and then
/// returns the Company
pub async fn get_user_company(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<db_entities::Company, AppError> {
    let db = &get_database_service().await.db;
    let assignments_collection = db.collection::<db_entities::UserCompanyAssignment>(
        db_entities::UserCompanyAssignment::collection_name(),
    );
    let filter = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = assignments_collection.find_one(filter).await?;

    if query_result.is_some() {
        let company_collection =
            db.collection::<db_entities::Company>(db_entities::Company::collection_name());
        let filter = doc! {"_id": company_id};
        let query_result = company_collection.find_one(filter).await?;
        if let Some(company) = query_result {
            Ok(company)
        } else {
            Err(AppError::InternalServerError(anyhow!(
                "Something went wrong in retrieving company {company_id} for user {user_id}"
            )))
        }
    } else {
        Err(AppError::DoesNotExist(anyhow!(
            "There is no company with id {company_id} for user {user_id}"
        )))
    }
}

/// Add the user to the company if it is not already in
pub async fn add_user_to_company(
    user_id: DocumentId,
    company_id: DocumentId,
    role: CompanyRole,
    job_title: String,
) -> Result<(), AppError> {
    let db = &get_database_service().await.db;
    let assignments_collection = db.collection::<db_entities::UserCompanyAssignment>(
        db_entities::UserCompanyAssignment::collection_name(),
    );
    let filter = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = assignments_collection.find_one(filter).await?;
    if query_result.is_none() {
        let new_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id,
            company_id,
            role,
            job_title,
        };
        new_assignment.save().await?;
        Ok(())
    } else {
        Err(AppError::ManagedError(format!("Failed to add user {user_id} to company {company_id} with role {role} because it is already in the Company with role {}", query_result.unwrap().role)))
    }
}

/// Remove the user from the company
pub async fn remove_user_from_company(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<(), AppError> {
    let db = &get_database_service().await.db;
    let assignments_collection = db.collection::<db_entities::UserCompanyAssignment>(
        db_entities::UserCompanyAssignment::collection_name(),
    );
    let filter = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = assignments_collection.find_one(filter).await?;
    if let Some(assignment) = query_result {
        assignment.delete().await
    } else {
        Err(AppError::ManagedError("Failed to remove user {user_id} from company {company_id} because he does not belong to it.".into()))
    }
}

/// Update user in the company by changing role or job title
pub async fn update_user_in_company(
    user_id: &DocumentId,
    company_id: &DocumentId,
    role: Option<CompanyRole>,
    job_title: Option<String>,
) -> Result<(), AppError> {
    let db = &get_database_service().await.db;
    let assignments_collection = db.collection::<db_entities::UserCompanyAssignment>(
        db_entities::UserCompanyAssignment::collection_name(),
    );
    let filter = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = assignments_collection.find_one(filter).await?;
    if let Some(assignment) = query_result {
        let mut update = doc! {};
        if let Some(role_obj) = role {
            update.insert("role", role_obj.to_string());
        }
        if let Some(job_title_obj) = job_title {
            update.insert("job_title", job_title_obj);
        }
        assignment.update(doc! {"$set": update}).await
    } else {
        Err(AppError::ManagedError("Failed to remove user {user_id} from company {company_id} because he does not belong to it.".into()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mongodb::bson::{doc, oid::ObjectId};

    use crate::{
        enums::CompanyRole,
        model::db_entities,
        service::{
            company::{
                add_user_to_company, create_company, get_user_companies, get_user_company,
                remove_user_from_company, update_user_in_company,
            },
            db::{get_database_service, DatabaseDocument},
        },
    };

    #[tokio::test]
    async fn create_company_test() {
        let user = db_entities::User {
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
        let user_id = ObjectId::from_str(&user.save().await.unwrap()).unwrap();

        let job_title = "CEO".to_string();
        let name = "My Company".to_string();
        let result = create_company(&user_id, name, job_title).await;
        assert!(result.is_ok());
        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn get_user_companies_test() {
        let company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save().await.unwrap()).unwrap();
        let first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save().await.unwrap()).unwrap();
        let first_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: first_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        first_assignment.save().await.unwrap();
        let second_user = db_entities::User {
            username: "riverpond".into(),
            password_hash: "fdsg39av2".into(),
            id: None,
            email: "john.smith@mail.com".into(),
            name: "John".into(),
            surname: "Smith".into(),
            api_key: None,
            platform_admin: false,
            active: true,
        };
        let second_user_id = ObjectId::from_str(&second_user.save().await.unwrap()).unwrap();
        let second_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: second_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::User,
            job_title: "Developer".into(),
        };
        second_assignment.save().await.unwrap();

        let result = get_user_companies(&first_user_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().first().unwrap().name, company.name);

        let result = get_user_company(&second_user_id, &company_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, company.name);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn add_user_to_company_test() {
        let company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save().await.unwrap()).unwrap();
        let first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save().await.unwrap()).unwrap();

        let result =
            add_user_to_company(first_user_id, company_id, CompanyRole::User, "CTO".into()).await;
        assert!(result.is_ok());
        let assignment = get_database_service()
            .await
            .db
            .collection::<db_entities::UserCompanyAssignment>(
                db_entities::UserCompanyAssignment::collection_name(),
            )
            .find_one(doc! {})
            .await
            .unwrap()
            .unwrap();
        assert_eq!(assignment.company_id, company_id);
        assert_eq!(assignment.user_id, first_user_id);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn remove_user_from_company_test() {
        let company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save().await.unwrap()).unwrap();
        let first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save().await.unwrap()).unwrap();
        let first_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: first_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        first_assignment.save().await.unwrap();

        let result = remove_user_from_company(&first_user_id, &company_id).await;
        assert!(result.is_ok());

        assert!(get_database_service()
            .await
            .db
            .collection::<db_entities::UserCompanyAssignment>(
                db_entities::UserCompanyAssignment::collection_name(),
            )
            .find_one(doc! {})
            .await
            .unwrap()
            .is_none());

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn update_user_in_company_test() {
        let company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save().await.unwrap()).unwrap();
        let first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save().await.unwrap()).unwrap();
        let first_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: first_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        first_assignment.save().await.unwrap();

        let new_job_title = "CIO".to_string();
        let result = update_user_in_company(
            &first_user_id,
            &company_id,
            None,
            Some(new_job_title.clone()),
        )
        .await;
        assert!(result.is_ok());

        let assignment = get_database_service()
            .await
            .db
            .collection::<db_entities::UserCompanyAssignment>(
                db_entities::UserCompanyAssignment::collection_name(),
            )
            .find_one(doc! {})
            .await
            .unwrap()
            .unwrap();
        assert_eq!(assignment.company_id, company_id);
        assert_eq!(assignment.user_id, first_user_id);
        assert_eq!(assignment.job_title, new_job_title);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
