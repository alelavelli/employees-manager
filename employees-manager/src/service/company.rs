use std::str::FromStr;

use anyhow::anyhow;
use futures::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Bson};

use super::db::{get_database_service, DatabaseDocument};
use crate::{
    enums::CompanyRole,
    error::AppError,
    model::{
        db_entities::{self, UserCompanyAssignment},
        internal::AdminPanelOverviewCompanyInfo,
    },
    DocumentId,
};

/// Returns the companies info for the admin panel
pub async fn get_admin_panel_overview_companies_info(
) -> Result<AdminPanelOverviewCompanyInfo, AppError> {
    let result = db_entities::Company::aggregate::<db_entities::Company>(vec![doc! {
        "$group": {
            "_id": null,
            "total_companies": { "$sum": 1 }
        }
    }])
    .await?;

    if let Some(result) = result.first() {
        Ok(AdminPanelOverviewCompanyInfo {
            total_companies: result
                .get("total_companies")
                .expect("total_companies should be present")
                .as_i32()
                .unwrap() as u16,
        })
    } else {
        Ok(AdminPanelOverviewCompanyInfo { total_companies: 0 })
    }
}

/// creates a Company and assigns it to the User creating an entry
/// in UserCompanyAssignment
pub async fn create_company(
    user_id: &DocumentId,
    name: String,
    job_title: String,
) -> Result<String, AppError> {
    let db_service = get_database_service().await;
    let mut transaction = db_service.new_transaction().await?;
    transaction.start_transaction().await?;

    let mut company_model = db_entities::Company {
        id: None,
        name,
        active: true,
    };
    let company_id = company_model.save(Some(&mut transaction)).await?;
    let company_id_object_id = ObjectId::from_str(&company_id);
    if company_id_object_id.is_err() {
        transaction.abort_transaction().await?;
        return Err(AppError::InternalServerError(anyhow!(
            "Unexpected failed conversion of ObjectId"
        )));
    }
    let mut user_company_assignment = db_entities::UserCompanyAssignment {
        id: None,
        user_id: *user_id,
        company_id: company_id_object_id.unwrap(),
        role: CompanyRole::Owner,
        job_title,
    };
    // If for some reasons we fail to dump the assignment we need to rollback
    if user_company_assignment
        .save(Some(&mut transaction))
        .await
        .is_err()
    {
        transaction.abort_transaction().await?;
        Err(AppError::InternalServerError(anyhow!(
            "Unexpected failed conversion of ObjectId"
        )))
    } else {
        transaction.commit_transaction().await?;
        Ok(company_id)
    }
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
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result =
        db_entities::UserCompanyAssignment::find_one::<db_entities::UserCompanyAssignment>(query)
            .await?;

    if query_result.is_some() {
        let query = doc! {"_id": company_id};
        let query_result = db_entities::Company::find_one::<db_entities::Company>(query).await?;
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
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result =
        db_entities::UserCompanyAssignment::find_one::<db_entities::UserCompanyAssignment>(query)
            .await?;
    if query_result.is_none() {
        let mut new_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id,
            company_id,
            role,
            job_title,
        };
        new_assignment.save(None).await?;
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
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result =
        db_entities::UserCompanyAssignment::find_one::<db_entities::UserCompanyAssignment>(query)
            .await?;
    if let Some(assignment) = query_result {
        assignment.delete(None).await
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
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result =
        db_entities::UserCompanyAssignment::find_one::<db_entities::UserCompanyAssignment>(query)
            .await?;
    if let Some(assignment) = query_result {
        let mut update = doc! {};
        if let Some(role_obj) = role {
            update.insert("role", role_obj.to_string());
        }
        if let Some(job_title_obj) = job_title {
            update.insert("job_title", job_title_obj);
        }
        db_entities::UserCompanyAssignment::update_one(
            doc! { "_id": assignment.get_id().unwrap()},
            doc! {"$set": update},
            None,
        )
        .await
    } else {
        Err(AppError::ManagedError("Failed to remove user {user_id} from company {company_id} because he does not belong to it.".into()))
    }
}

/// Returns the user company role assignment
pub async fn get_user_company_role(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<db_entities::UserCompanyAssignment, AppError> {
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result =
        db_entities::UserCompanyAssignment::find_one::<db_entities::UserCompanyAssignment>(query)
            .await?;
    if let Some(assignment) = query_result {
        Ok(assignment)
    } else {
        Err(AppError::DoesNotExist(anyhow!(
            "User with id {user_id} does not have a role in Company with id {company_id}.",
        )))
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

        let job_title = "CEO".to_string();
        let name = "My Company".to_string();
        let result = create_company(&user_id, name.clone(), job_title).await;
        assert!(result.is_ok());

        let assignment = db_entities::UserCompanyAssignment::find_one::<
            db_entities::UserCompanyAssignment,
        >(doc! {"user_id": user_id})
        .await
        .unwrap();
        assert!(assignment.is_some());

        let companies = db_entities::Company::find_many::<db_entities::Company>(doc! {})
            .await
            .unwrap();
        assert!(companies.get(0).unwrap().name == name);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn get_user_companies_test() {
        let mut company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();
        let mut first_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: first_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        first_assignment.save(None).await.unwrap();
        let mut second_user = db_entities::User {
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
        let second_user_id = ObjectId::from_str(&second_user.save(None).await.unwrap()).unwrap();
        let mut second_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: second_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::User,
            job_title: "Developer".into(),
        };
        second_assignment.save(None).await.unwrap();

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
        let mut company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();

        let result =
            add_user_to_company(first_user_id, company_id, CompanyRole::User, "CTO".into()).await;
        assert!(result.is_ok());

        let assignment = db_entities::UserCompanyAssignment::find_one::<
            db_entities::UserCompanyAssignment,
        >(doc! {})
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
        let mut company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();
        let mut first_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: first_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        first_assignment.save(None).await.unwrap();

        let result = remove_user_from_company(&first_user_id, &company_id).await;
        assert!(result.is_ok());

        assert!(
            db_entities::UserCompanyAssignment::find_one::<db_entities::UserCompanyAssignment>(
                doc! {}
            )
            .await
            .unwrap()
            .is_none()
        );

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn update_user_in_company_test() {
        let mut company = db_entities::Company {
            id: None,
            name: "My Company".into(),
            active: true,
        };
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User {
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
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();
        let mut first_assignment = db_entities::UserCompanyAssignment {
            id: None,
            user_id: first_user_id.clone(),
            company_id,
            role: crate::enums::CompanyRole::Owner,
            job_title: "CEO".into(),
        };
        first_assignment.save(None).await.unwrap();

        let new_job_title = "CIO".to_string();
        let result = update_user_in_company(
            &first_user_id,
            &company_id,
            None,
            Some(new_job_title.clone()),
        )
        .await;
        assert!(result.is_ok());

        let assignment = db_entities::UserCompanyAssignment::find_one::<
            db_entities::UserCompanyAssignment,
        >(doc! {})
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
