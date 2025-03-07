use bson::doc;
use serde::{Deserialize, Serialize};

use crate::{enums::CompanyRole, error::ServiceAppError, model::db_entities, DocumentId};

use super::db::DatabaseDocument;

/// Returns the list of companies the user can use to create a new Corporate Group
///
/// Companies are eligible if they do not belong to any other corporate group and if
/// the user has Admin or Owner role for the Company.
pub async fn get_eligible_companies_for_corporate_group(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::Company>, ServiceAppError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct CompaniesQueryResult {
        company_id: DocumentId,
    }
    // First retrieve the companies for which the user is Admin or Owner
    // because they are ones that he can add in the group
    let user_companies =
        db_entities::UserCompanyAssignment::find_many_projection::<CompaniesQueryResult>(
            doc! {
                "user_id": user_id,
                "role": {"$in": [CompanyRole::Admin, CompanyRole::Owner]},
            },
            doc! {"company_id": 1},
        )
        .await?;

    // Then retrieve the companies that are already in a Corporate Group
    // because they need to be discarded
    let pipeline = vec![
        doc! {
            "$unwind": "$company_ids"
        },
        doc! {
            "$group": {
                "_id": null,
                "all_company_ids": { "$addToSet": "$company_ids" }
            }
        },
        doc! {
            "$project": {
                "_id": 0,
                "all_company_ids": 1
            }
        },
        doc! {"$unwind": "$all_company_ids"},
    ];
    // TODO: flat map can lead to silent failure if the extraction to object id returns Err.
    // However, since the object id is retrieved from the database, the extraction should always go well
    let company_ids_in_group: Vec<DocumentId> = db_entities::CorporateGroup::aggregate(pipeline)
        .await?
        .into_iter()
        .flat_map(|elem| elem.get_object_id("all_company_ids").clone())
        .collect();

    let eligible_company_ids: Vec<DocumentId> = user_companies
        .into_iter()
        .map(|elem| elem.company_id)
        .filter(|elem| !company_ids_in_group.contains(elem))
        .collect();

    let eligible_companies = db_entities::Company::find_many(
        doc! {"_id": {"$in": eligible_company_ids}, "active": true},
    )
    .await?;
    Ok(eligible_companies)
}

/// Creates a new corporate group
///
/// It returns ServiceAppError::InvalidRequest:
///     - if a corporate group with the same name already exists
///     - if a Company already belongs to another group
///     - if company vector is empty
///     - if a company in the list has not this user as owner or admin
///
/// The user that creates the corporate group becomes the owner
pub async fn create_corporate_group(
    user_id: &DocumentId,
    name: String,
    company_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    if company_ids.is_empty() {
        Err(ServiceAppError::InvalidRequest(
            "You cannot create a Corporate Group without companies.".to_string()
        ))
    } else if db_entities::CorporateGroup::count_documents(doc! { "name": &name }).await? > 0 {
        Err(ServiceAppError::InvalidRequest(format!(
            "Corporate Group with name {name} already exist."
        )))
    } else if db_entities::CorporateGroup::count_documents(
        doc! { "company_ids": {"$in": &company_ids} },
    )
    .await?
        > 0
    {
        Err(ServiceAppError::InvalidRequest(
            "Companies cannot belong to more than one Corporate Group".to_string()
        ))
    } else if db_entities::UserCompanyAssignment::count_documents(
        doc! { "user_id": user_id, "company_id": {"$in": &company_ids}, "role": {"$in": [CompanyRole::Owner, CompanyRole::Admin]}},
    ).await? != company_ids.len() as u64 {
        Err(ServiceAppError::InvalidRequest("User must have at least admin role to add a company in the corporate group.".to_string()))
    } else {
        let mut new_doc = db_entities::CorporateGroup::new(name, company_ids, *user_id);
        new_doc.save(None).await?;
        Ok(())
    }
}

/// Deletes corporate group if the user has permissions to do it
pub async fn delete_corporate_group(
    user_id: &DocumentId,
    corporate_group_id: &DocumentId,
) -> Result<(), ServiceAppError> {
    let corporate_group_list = get_corporate_groups_for_user(user_id)
        .await?
        .into_iter()
        .filter(|doc| {
            doc.get_id()
                .is_some_and(|doc_id| doc_id == corporate_group_id)
        })
        .collect::<Vec<db_entities::CorporateGroup>>();

    if let Some(corporate_group) = corporate_group_list.first() {
        corporate_group.delete(None).await
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Corporate group with id {corporate_group_id} does not exist."
        )))
    }
}

/// Returns the corporate groups visible by the user.
/// A user can view a corporate group if it is at least admin of a Company
/// that is in the group.
/// A user can see more than one group because it can belong to more companies that are in different groups
pub async fn get_corporate_groups_for_user(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::CorporateGroup>, ServiceAppError> {
    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct QueryResult {
        company_id: DocumentId,
    }

    let user_companies = db_entities::UserCompanyAssignment::find_many_projection::<QueryResult>(
        doc! {"user_id": user_id, "role": {"$in": [CompanyRole::Owner, CompanyRole::Admin]}},
        doc! {"company_id": 1},
    )
    .await?
    .into_iter()
    .map(|elem| elem.company_id)
    .collect::<Vec<DocumentId>>();

    db_entities::CorporateGroup::find_many(doc! {"company_ids": {"$in": user_companies}}).await
}

/// Returns the corporate group that contains the company.
/// It is None when the Company does not belong to any group.
pub async fn get_corporate_group_for_company(
    company_id: &DocumentId,
) -> Result<Option<db_entities::CorporateGroup>, ServiceAppError> {
    db_entities::CorporateGroup::find_one(doc! {"company_ids": company_id}).await
}

/// Edit corporate group by changing the name or the company list
///
/// It returns ServiceAppError::InvalidRequest:
///     - if a corporate group with the same name already exists
///     - if a Company already belongs to another group
///     - if company vector is empty
pub async fn edit_corporate_group(
    user_id: &DocumentId,
    group_id: &DocumentId,
    name: String,
    company_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    if company_ids.is_empty() {
        Err(ServiceAppError::InvalidRequest(
            "You cannot have a Corporate Group without companies.".to_string()
        ))
    } else if db_entities::CorporateGroup::count_documents(
        doc! { "name": &name, "_id": {"$ne": group_id}},
    )
    .await?
        > 0
    {
        Err(ServiceAppError::InvalidRequest(format!(
            "Corporate Group with name {name} already exist."
        )))
    } else if db_entities::CorporateGroup::count_documents(
        doc! { "company_ids": {"$in": &company_ids}, "_id": {"$ne": group_id}},
    )
    .await?
        > 0
    {
        Err(ServiceAppError::InvalidRequest(
            "Companies cannot belong to more than one Corporate Group".to_string()))
    } else if db_entities::UserCompanyAssignment::count_documents(
        doc! { "user_id": user_id, "company_id": {"$in": &company_ids}, "role": {"$in": [CompanyRole::Owner, CompanyRole::Admin]}},
    ).await? != company_ids.len() as u64 {
        Err(ServiceAppError::InvalidRequest("User must have at least admin role to add a company in the corporate group.".to_string()))
    } else {
        db_entities::CorporateGroup::update_one(
            doc! {"_id": group_id},
            doc! {"$set": {"name": name, "company_ids": company_ids}},
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use bson::oid::ObjectId;

    use crate::{
        enums::CompanyRole,
        model::db_entities,
        service::{
            corporate_group::{create_corporate_group, get_eligible_companies_for_corporate_group},
            db::{get_database_service, DatabaseDocument},
        },
    };

    #[tokio::test]
    async fn test_create_corporate_group() {
        let companies: Vec<ObjectId> = (0..5).map(|_| ObjectId::new()).collect();
        let user = ObjectId::new();
        let mut first_group =
            db_entities::CorporateGroup::new("First group".into(), companies[0..3].to_vec(), user);

        for (index, company_id) in companies.iter().enumerate() {
            let mut assignment = db_entities::UserCompanyAssignment::new(
                user,
                company_id.clone(),
                if index == 3 {
                    CompanyRole::User
                } else {
                    CompanyRole::Admin
                },
                "job_title".into(),
                vec![],
            );
            assignment.save(None).await.unwrap();
        }

        first_group.save(None).await.unwrap();
        let result = create_corporate_group(&user, "New group".into(), companies.clone()).await;

        assert!(
            result.is_err(),
            "expecting an error because there is already a group with companies"
        );

        let result =
            create_corporate_group(&user, "New group".into(), companies[3..5].to_vec()).await;
        assert!(
            result.is_err(),
            "expecting an error because the user is not admin of a company"
        );

        let result =
            create_corporate_group(&user, "New group".into(), companies[4..5].to_vec()).await;
        assert!(
            result.is_ok(),
            "expecting correct creation of the corporate group"
        );

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn test_get_eligible_companies_for_corporate_group() {
        let mut companies: Vec<ObjectId> = vec![];
        for i in 0..5 {
            let mut company = db_entities::Company::new(format!("company {i}"), true);
            company.save(None).await.unwrap();
            company.reload().await.unwrap();
            companies.push(company.get_id().unwrap().clone());
        }

        let user = ObjectId::new();
        let mut first_group =
            db_entities::CorporateGroup::new("First group".into(), companies[0..3].to_vec(), user);
        first_group.save(None).await.unwrap();

        for (index, company_id) in companies.iter().enumerate() {
            let mut assignment = db_entities::UserCompanyAssignment::new(
                user,
                company_id.clone(),
                if index == 3 {
                    CompanyRole::User
                } else {
                    CompanyRole::Admin
                },
                "job_title".into(),
                vec![],
            );
            assignment.save(None).await.unwrap();
        }

        let eligible_companies = get_eligible_companies_for_corporate_group(&user)
            .await
            .unwrap();

        assert_eq!(eligible_companies.len(), 1);
        assert_eq!(
            eligible_companies[0].get_id().unwrap(),
            companies.last().unwrap()
        );

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
