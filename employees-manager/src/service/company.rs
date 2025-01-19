use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;

use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::db::{get_database_service, DatabaseDocument};
use crate::{
    enums::{CompanyRole, NotificationType},
    error::AppError,
    model::{
        db_entities,
        internal::{AdminPanelOverviewCompanyInfo, InvitedUserInCompanyInfo, UserInCompanyInfo},
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
/// Moreover, it creates the empty document CompanyManagementTeam that will
/// be used to identity manager users for the company
pub async fn create_company(
    user_id: &DocumentId,
    name: String,
    job_title: String,
) -> Result<String, AppError> {
    // check if a company with the same name already exists
    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        name: String,
    }

    let companies =
        db_entities::Company::find_many_projection::<db_entities::Company, QueryResult>(
            doc! {},
            doc! {"name": 1},
        )
        .await?;
    for document in companies {
        if name.to_lowercase().trim() == document.name.to_lowercase().trim() {
            return Err(AppError::ManagedError(
                "A Company with name {name} already exists.".into(),
            ));
        }
    }

    let db_service = get_database_service().await;
    let mut transaction = db_service.new_transaction().await?;
    transaction.start_transaction().await?;

    let mut company_model = db_entities::Company {
        id: None,
        name: name.trim().into(),
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
    let company_id_object_id = company_id_object_id.unwrap();
    let mut user_company_assignment = db_entities::UserCompanyAssignment {
        id: None,
        user_id: *user_id,
        company_id: company_id_object_id,
        role: CompanyRole::Owner,
        job_title,
    };
    // If for some reasons we fail to dump the assignment we need to rollback
    user_company_assignment.save(Some(&mut transaction)).await?;

    let mut company_management_team = db_entities::CompanyManagementTeam {
        id: None,
        company_id: company_id_object_id,
        user_ids: vec![],
    };
    company_management_team.save(Some(&mut transaction)).await?;

    transaction.commit_transaction().await?;
    Ok(company_id)
}

pub async fn get_companies() -> Result<Vec<db_entities::Company>, AppError> {
    db_entities::Company::find_many::<db_entities::Company>(doc! {}).await
}

/// Get all the Companies the User is in by looking at the UserCompanyAssignment
pub async fn get_user_companies(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::Company>, AppError> {
    let query_result = db_entities::UserCompanyAssignment::find_many::<
        db_entities::UserCompanyAssignment,
    >(doc! { "user_id": user_id})
    .await?;

    let mut company_ids = vec![];
    for doc in query_result {
        company_ids.push(Bson::ObjectId(doc.company_id));
    }
    if company_ids.is_empty() {
        return Ok(vec![]);
    }

    let query_result = db_entities::Company::find_many(doc! { "_id": {"$in": company_ids}}).await?;
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

/// Update the management team for the company adding or removing the given user
pub async fn change_user_company_manager(
    user_id: &DocumentId,
    company_id: &DocumentId,
    manager: bool,
) -> Result<(), AppError> {
    let query_result = db_entities::CompanyManagementTeam::find_one::<
        db_entities::CompanyManagementTeam,
    >(doc! { "company_id": company_id})
    .await?;

    if let Some(mut management_team) = query_result {
        let mut user_index = None;
        for (i, i_user_id) in management_team.user_ids.iter().enumerate() {
            if i_user_id == user_id {
                user_index = Some(i);
                break;
            }
        }
        let is_user_a_manager = user_index.is_some();
        if is_user_a_manager & !manager {
            // we remove the user to the management team
            management_team.user_ids.remove(user_index.unwrap());
            management_team.save(None).await?;
        } else if !is_user_a_manager & manager {
            // we add the user to the management team
            management_team.user_ids.push(*user_id);
            management_team.save(None).await?;
        }
        // otherwise the user is either not a manager and we want to remove him
        // or he is a manager and we want to add him
        Ok(())
    } else {
        Err(AppError::InternalServerError(anyhow!(format!(
            "Missing management team for company {}",
            company_id
        ))))
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

/// Returns the users inside a company
pub async fn get_users_in_company(
    company_id: &DocumentId,
) -> Result<Vec<UserInCompanyInfo>, AppError> {
    let assignments: HashMap<DocumentId, db_entities::UserCompanyAssignment> =
        db_entities::UserCompanyAssignment::find_many::<db_entities::UserCompanyAssignment>(
            doc! { "company_id": company_id },
        )
        .await?
        .into_iter()
        .map(|doc| (doc.user_id, doc))
        .collect();

    let management_team = db_entities::CompanyManagementTeam::find_one::<
        db_entities::CompanyManagementTeam,
    >(doc! {"company_id": company_id})
    .await?;

    let user_ids: Vec<Bson> = assignments
        .iter()
        .map(|(&id, _)| Bson::ObjectId(id))
        .collect();
    let users: Vec<db_entities::User> =
        db_entities::User::find_many::<db_entities::User>(doc! {"_id": {"$in": user_ids}}).await?;
    let mut to_return = vec![];
    for user in users {
        let user_id = user
            .get_id()
            .expect("expecting to have id after query on db.");
        if let Some(user_assignment) = assignments.get(user_id) {
            to_return.push(UserInCompanyInfo {
                user_id: *user_id,
                company_id: *company_id,
                role: user_assignment.role,
                username: user.username.clone(),
                surname: user.surname.clone(),
                name: user.name.clone(),
                job_title: user_assignment.job_title.clone(),
                management_team: management_team
                    .as_ref()
                    .is_some_and(|doc| doc.user_ids.contains(user_id)),
            });
        }
    }
    Ok(to_return)
}

pub async fn invite_user(
    inviting_user_id: DocumentId,
    company_id: DocumentId,
    invited_user_id: DocumentId,
    role: CompanyRole,
    job_title: String,
) -> Result<(), AppError> {
    /*
    Create InviteAddCompany document and AppNotification document
    */

    // the role can be Admin only if the requesting user is an admin
    if role == CompanyRole::Admin {
        #[derive(Serialize, Deserialize, Debug)]
        struct QueryResult {
            role: CompanyRole,
        }

        let inviting_user_role = db_entities::UserCompanyAssignment::find_one_projection::<
            db_entities::UserCompanyAssignment,
            QueryResult,
        >(doc! {"user_id": inviting_user_id}, doc! {"role": 1})
        .await?
        .expect("user must be in company")
        .role;

        if inviting_user_role == CompanyRole::Admin {
            return Err(AppError::ManagedError("You don't have Admin role in Company {company_id}, hence, you cannot assign Admin role to other users".into()));
        }
    }

    let db_service = get_database_service().await;
    let mut transaction = db_service.new_transaction().await?;
    transaction.start_transaction().await?;

    let mut invite = db_entities::InviteAddCompany {
        id: None,
        inviting_user_id,
        invited_user_id,
        company_id,
        company_role: role,
        job_title,
        answer: None,
    };
    invite.save(Some(&mut transaction)).await?;

    let query_result =
        db_entities::Company::find_one::<db_entities::Company>(doc! {"_id": company_id}).await;
    if let Ok(Some(company)) = query_result {
        let mut notification = db_entities::AppNotification {
            id: None,
            user_id: invited_user_id,
            notification_type: NotificationType::InviteAddCompany,
            message: format!("You has been invited to Company {}", company.name),
            read: false,
            entity_id: invite.get_id().cloned(),
        };
        notification.save(Some(&mut transaction)).await?;

        transaction.commit_transaction().await?;

        Ok(())
    } else {
        transaction.abort_transaction().await?;
        Err(AppError::ManagedError(format!(
            "Company with id {} does not exist",
            company_id
        )))
    }
}

pub async fn get_pending_invited_users(
    company_id: &DocumentId,
) -> Result<Vec<InvitedUserInCompanyInfo>, AppError> {
    let pending_invitations = db_entities::InviteAddCompany::find_many::<
        db_entities::InviteAddCompany,
    >(doc! {"company_id": company_id, "answer": null})
    .await?;

    #[derive(Serialize, Deserialize, Debug)]
    struct NotificationQueryResult {
        _id: DocumentId,
        entity_id: DocumentId,
    }

    let notifications_map = db_entities::AppNotification::find_many_projection::<db_entities::AppNotification, NotificationQueryResult>(
        doc! { "entity_id": {"$in": pending_invitations.iter().map(|doc| doc.get_id().expect("expecting object id after database read")).collect::<Vec<&DocumentId>>()} }, 
        doc! {
            "_id": 1,
            "entity_id": 1
        }
    ).await?.iter().map(|doc| (doc.entity_id, doc._id)).collect::<HashMap<DocumentId, DocumentId>>();

    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        _id: DocumentId,
        username: String,
    }
    let usernames = db_entities::User::find_many_projection::<db_entities::User, QueryResult>(
        doc! {"_id": {"$in": pending_invitations.iter().map(|doc| &doc.invited_user_id).collect::<Vec<&DocumentId>>()}},
        doc! {
            "username": 1,
            "_id": 1
        },
    )
    .await?.iter().map(|doc| (doc._id, doc.username.clone())).collect::<HashMap<DocumentId, String>>();

    let mut to_return = vec![];

    for invitation in pending_invitations {
        if let Some(username) = usernames.get(&invitation.invited_user_id) {
            let notification_id = *notifications_map
                .get(
                    invitation
                        .get_id()
                        .expect("id should exist from document retrieved from db"),
                )
                .expect("Expecting object id since it is read above");
            to_return.push(InvitedUserInCompanyInfo {
                notification_id: notification_id.to_hex(),
                user_id: invitation.invited_user_id.to_hex(),
                username: username.clone(),
                role: invitation.company_role,
                job_title: invitation.job_title,
                company_id: invitation.company_id.to_hex(),
            });
        } else {
            return Err(AppError::InternalServerError(anyhow!(format!(
                "User {} should exist",
                invitation.invited_user_id.to_hex()
            ))));
        }
    }

    Ok(to_return)
}

pub async fn get_users_to_invite_in_company(
    company_id: DocumentId,
) -> Result<Vec<(DocumentId, String)>, AppError> {
    // Users can be invited to a company if they are not already in it and if there is no pending invitation

    #[derive(Serialize, Deserialize, Debug)]
    struct InvitedUsersQueryResult {
        invited_user_id: DocumentId,
    }
    let mut users_to_exclude: Vec<DocumentId> =
        db_entities::InviteAddCompany::find_many_projection::<
            db_entities::InviteAddCompany,
            InvitedUsersQueryResult,
        >(
            doc! {"company_id": company_id, "answer": null},
            doc! {"invited_user_id": 1},
        )
        .await?
        .iter()
        .map(|doc| doc.invited_user_id)
        .collect();

    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        user_id: DocumentId,
    }
    let mut users_in_company: Vec<DocumentId> =
        db_entities::UserCompanyAssignment::find_many_projection::<
            db_entities::UserCompanyAssignment,
            QueryResult,
        >(
            doc! {"company_id": company_id},
            doc! {
                "user_id": 1
            },
        )
        .await?
        .iter()
        .map(|doc| doc.user_id)
        .collect();

    #[derive(Serialize, Deserialize, Debug)]
    struct UserQueryResult {
        _id: DocumentId,
        username: String,
    }

    users_to_exclude.append(&mut users_in_company);

    let to_return: Vec<(DocumentId, String)> =
        db_entities::User::find_many_projection::<db_entities::User, UserQueryResult>(
            doc! {"_id": {"$not": {"$in": users_to_exclude}}},
            doc! {
                "_id": 1,
                "username": 1,
            },
        )
        .await?
        .iter()
        .map(|user| (user._id, user.username.clone()))
        .collect();

    Ok(to_return)
}

pub async fn get_company_projects(
    company_id: DocumentId,
) -> Result<Vec<db_entities::CompanyProject>, AppError> {
    db_entities::CompanyProject::find_many::<db_entities::CompanyProject>(
        doc! {"company_id": company_id},
    )
    .await
}

pub async fn create_project(
    company_id: DocumentId,
    name: String,
    code: String,
) -> Result<String, AppError> {
    let company_projects = db_entities::CompanyProject::find_many::<db_entities::CompanyProject>(
        doc! {"company_id": company_id},
    )
    .await?;

    // project name and code must be unique
    for project in company_projects {
        if project.name == name || project.code == code {
            return Err(AppError::ManagedError(format!(
                "Project name and code must be unique got name: {} and code: {}",
                name, code
            )));
        }
    }

    let mut new_project = db_entities::CompanyProject {
        id: None,
        company_id,
        name,
        code,
    };

    new_project.save(None).await
}

pub async fn edit_project(
    company_id: DocumentId,
    project_id: DocumentId,
    name: String,
    code: String,
) -> Result<String, AppError> {
    let company_project_query =
        db_entities::CompanyProject::find_one::<db_entities::CompanyProject>(
            doc! {"_id": project_id, "company_id": company_id},
        )
        .await?;

    if let Some(mut company_project) = company_project_query {
        debug!("Name is {name}, Code is {code}");
        company_project.name = name;
        company_project.code = code;
        company_project.save(None).await
    } else {
        Err(AppError::ManagedError(format!(
            "Project with id {} does not exist",
            project_id
        )))
    }
}

pub async fn delete_project(
    company_id: DocumentId,
    project_id: DocumentId,
) -> Result<(), AppError> {
    let company_project_query =
        db_entities::CompanyProject::find_one::<db_entities::CompanyProject>(
            doc! {"_id": project_id, "company_id": company_id},
        )
        .await?;

    if let Some(company_project) = company_project_query {
        company_project.delete(None).await
    } else {
        Err(AppError::ManagedError(format!(
            "Project with id {} does not exist",
            project_id
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
