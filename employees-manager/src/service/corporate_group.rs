use bson::doc;
use serde::{Deserialize, Serialize};

use crate::{enums::CompanyRole, error::ServiceAppError, model::db_entities, DocumentId};

use super::db::DatabaseDocument;

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
    user_id: DocumentId,
    name: String,
    company_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    if company_ids.is_empty() {
        Err(ServiceAppError::InvalidRequest(format!(
            "You cannot create a Corporate Group without companies."
        )))
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
        Err(ServiceAppError::InvalidRequest(format!(
            "Companies cannot belong to more than one Corporate Group"
        )))
    } else if db_entities::UserCompanyAssignment::count_documents(
        doc! { "user_id": user_id, "company_id": {"$in": &company_ids}, "role": {"$in": [CompanyRole::Owner, CompanyRole::Admin]}},
    ).await? != company_ids.len() as u64 {
        Err(ServiceAppError::InvalidRequest(format!("User must have at least admin role to add a company in the corporate group.")))
    } else {
        let mut new_doc = db_entities::CorporateGroup::new(name, company_ids, user_id);
        new_doc.save(None).await?;
        Ok(())
    }
}

/// Returns the corporate groups visible by the user.
/// A user can view a corporate group if it is at least admin of a Company
/// that is in the group.
/// A user can see more than one group because it can belong to more companies that are in different groups
pub async fn get_corporate_groups_for_user(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::CorporateGroup>, ServiceAppError> {
    #[derive(Serialize, Deserialize)]
    struct QueryResult {
        company_id: DocumentId,
    }

    let user_companies = db_entities::UserCompanyAssignment::find_many_projection::<QueryResult>(
        doc! {"user_ids": user_id, "role": {"$in": [CompanyRole::Owner, CompanyRole::Admin]}},
        doc! {"company_id": 1},
    )
    .await?;
    db_entities::CorporateGroup::find_many(
        doc! {"company_ids": user_companies.into_iter().map(|elem| elem.company_id).collect::<Vec<DocumentId>>()},
    )
    .await
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
    user_id: DocumentId,
    group_id: DocumentId,
    name: String,
    company_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    if company_ids.is_empty() {
        Err(ServiceAppError::InvalidRequest(format!(
            "You cannot have a Corporate Group without companies."
        )))
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
        Err(ServiceAppError::InvalidRequest(format!(
            "Companies cannot belong to more than one Corporate Group"
        )))
    } else if db_entities::UserCompanyAssignment::count_documents(
        doc! { "user_id": user_id, "company_id": {"$in": &company_ids}, "role": {"$in": [CompanyRole::Owner, CompanyRole::Admin]}},
    ).await? != company_ids.len() as u64 {
        Err(ServiceAppError::InvalidRequest(format!("User must have at least admin role to add a company in the corporate group.")))
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
            corporate_group::create_corporate_group,
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
        let result = create_corporate_group(user, "New group".into(), companies.clone()).await;

        assert!(
            result.is_err(),
            "expecting an error because there is already a group with companies"
        );

        let result =
            create_corporate_group(user, "New group".into(), companies[3..5].to_vec()).await;
        assert!(
            result.is_err(),
            "expecting an error because the user is not admin of a company"
        );

        let result =
            create_corporate_group(user, "New group".into(), companies[4..5].to_vec()).await;
        assert!(
            result.is_ok(),
            "expecting correct creation of the corporate group"
        );

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
