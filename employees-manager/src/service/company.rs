use std::{collections::HashMap, str::FromStr};

use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

use super::db::{document::DatabaseDocument, get_database_service};
use crate::{
    enums::{CompanyRole, NotificationType},
    error::ServiceAppError,
    model::{
        db_entities,
        internal::{AdminPanelOverviewCompanyInfo, InvitedUserInCompanyInfo, UserInCompanyInfo},
    },
    service::{
        self,
        db::{document::SmartDocumentReference, transaction::DatabaseTransaction},
    },
    DocumentId,
};

/// Creates a new company if its name does not already exist and
/// adds it to the corporate group company list
pub async fn create_company(
    corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
    company_name: String,
) -> Result<(), ServiceAppError> {
    let mut corporate_group = corporate_group_id.to_document().await?;
    // check if a company in the corporate group with the same name already exists
    if db_entities::Company::count_documents(doc! {"_id": {"$in": corporate_group.company_ids()}})
        .await?
        > 0
    {
        Err(ServiceAppError::InvalidRequest(format!(
            "A Company with name {company_name} already exists in the Corporate Group {}",
            corporate_group.name()
        )))
    } else {
        // With a transaction we create the company and we add it to the corporate group
        let db_service = get_database_service().await;
        let mut transaction = db_service.new_transaction().await?;
        transaction.start_transaction().await?;

        let mut new_company = db_entities::Company::new(company_name.trim().into(), true);
        let company_id = new_company.save(Some(&mut transaction)).await?;

        if let Ok(company_id_obj) = ObjectId::from_str(&company_id) {
            corporate_group.company_ids_mut().push(company_id_obj);
            corporate_group.save(Some(&mut transaction)).await?;
            transaction.commit_transaction().await?;
            Ok(())
        } else {
            transaction.abort_transaction().await?;
            Err(ServiceAppError::InternalServerError(format!(
                "An error occurred during translation of company_id to ObjectId"
            )))
        }
    }
}

pub async fn delete_company(
    company: SmartDocumentReference<db_entities::Company>,
) -> Result<Option<&mut DatabaseTransaction>, ServiceAppError> {
    // TODO: understand how to create an internal transaction here avoiding compiler problems
    let company_id = company.to_id();

    let mut transaction = transaction;

    // delete user employment contract
    transaction = db_entities::UserEmploymentContract::delete_many(
        doc! {"company_id": company_id},
        transaction,
    )
    .await?;

    // delete user company role
    transaction =
        db_entities::UserCompanyRole::delete_many(doc! {"company_id": company_id}, transaction)
            .await?;

    // delete company employee request
    transaction = db_entities::CompanyEmployeeRequest::delete_many(
        doc! {"company_id": company_id},
        transaction,
    )
    .await?;

    // delete invite add company
    transaction =
        db_entities::InviteAddCompany::delete_many(doc! {"company_id": company_id}, transaction)
            .await?;

    // delete company project
    let projects = db_entities::CompanyProject::find_many(doc! {"company_id": company_id}).await?;
    for project in projects {
        transaction = service::project::delete_project(
            SmartDocumentReference::Id(*project.get_id().unwrap()),
            transaction,
        )
        .await?;
    }

    // delete timesheet activity hours inside timesheet day
    transaction = db_entities::TimesheetDay::delete_many(
        doc! {"activities.company_id": company_id},
        transaction,
    )
    .await?;

    Ok(transaction)
}

/// Returns an hashmap with key the company id and value its name
pub async fn get_company_names(
    company_ids: &Vec<DocumentId>,
) -> Result<HashMap<DocumentId, String>, ServiceAppError> {
    #[derive(Deserialize, Serialize)]
    struct QueryResult {
        _id: DocumentId,
        name: String,
    }

    let query_result = db_entities::Company::find_many_projection::<QueryResult>(
        doc! {"_id": {"$in": company_ids}},
        doc! {"_id": 1, "name": 1},
    )
    .await?;

    Ok(query_result
        .into_iter()
        .map(|e| (e._id, e.name))
        .collect::<HashMap<DocumentId, String>>())
}

/// Verifies that the entry in UserCompanyAssignment exists and then
/// returns the Company
pub async fn get_user_company(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<db_entities::Company, ServiceAppError> {
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = db_entities::UserEmploymentContract::find_one(query).await?;

    if query_result.is_some() {
        let query = doc! {"_id": company_id};
        let query_result = db_entities::Company::find_one(query).await?;
        if let Some(company) = query_result {
            Ok(company)
        } else {
            Err(ServiceAppError::InternalServerError(
                format!("Company with id {company_id} should exist because it is retrieved from UserCompanyAssignment for user {user_id}")
            ))
        }
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "There is no company with id {company_id} for user {user_id}"
        )))
    }
}

/// Add the user to the company if it is not already in
pub async fn create_user_contract_with_company(
    user_id: DocumentId,
    company_id: DocumentId,
    job_title: String,
) -> Result<(), ServiceAppError> {
    if db_entities::UserEmploymentContract::count_documents(
        doc! { "user_id": user_id, "company_id": company_id},
    )
    .await?
        > 0
    {
        Err(ServiceAppError::InvalidRequest(format!("Failed to add user {user_id} to company {company_id} with jot title {job_title} because it is already in the Company ")))
    } else {
        let mut new_doc = db_entities::UserEmploymentContract::new(user_id, company_id, job_title);
        new_doc.save(None).await?;
        Ok(())
    }
}

/// Deletes contract of the user with the company
pub async fn delete_user_contract_with_company(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<(), ServiceAppError> {
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = db_entities::UserEmploymentContract::find_one(query).await?;
    if let Some(assignment) = query_result {
        assignment.delete(None).await?;
        Ok(())
    } else {
        return Err(ServiceAppError::InvalidRequest(format!("Failed to remove user {user_id} from company {company_id} because he does not belong to it.")));
    }
}

/// Update user in the company by changing job title
pub async fn update_user_contract_with_company(
    user_id: &DocumentId,
    company_id: &DocumentId,
    job_title: Option<String>,
) -> Result<(), ServiceAppError> {
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = db_entities::UserEmploymentContract::find_one(query).await?;
    if let Some(assignment) = query_result {
        let mut update = doc! {};
        if let Some(job_title_obj) = job_title {
            update.insert("job_title", job_title_obj);
        }
        db_entities::UserEmploymentContract::update_one(
            doc! { "_id": assignment.get_id().unwrap()},
            doc! {"$set": update},
            None,
        )
        .await
    } else {
        Err(ServiceAppError::InvalidRequest(format!("Failed to remove user {user_id} from company {company_id} because he does not belong to it.")))
    }
}

/// Returns the contract of the user with the company
pub async fn get_user_company_contract(
    user_id: &DocumentId,
    company_id: &DocumentId,
) -> Result<db_entities::UserEmploymentContract, ServiceAppError> {
    let query = doc! { "user_id": user_id, "company_id": company_id};
    let query_result = db_entities::UserEmploymentContract::find_one(query).await?;
    if let Some(assignment) = query_result {
        Ok(assignment)
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "User with id {user_id} does not have a role in Company with id {company_id}.",
        )))
    }
}

pub async fn invite_user(
    inviting_user_id: DocumentId,
    company_id: DocumentId,
    invited_user_id: DocumentId,
    role: CompanyRole,
    job_title: String,
    project_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    // TODO: move this to the corporate group or delete the concept of invitation to corporate group
    /*
    Create InviteAddCompany document and AppNotification document
    */

    // the role can be Admin only if the requesting user is an admin
    if role == CompanyRole::Admin {
        #[derive(Serialize, Deserialize, Debug)]
        struct QueryResult {
            role: CompanyRole,
        }

        let inviting_user_role = db_entities::UserEmploymentContract::find_one_projection::<
            QueryResult,
        >(doc! {"user_id": inviting_user_id}, doc! {"role": 1})
        .await?
        .expect("user must be in company")
        .role;

        if inviting_user_role == CompanyRole::Admin {
            return Err(ServiceAppError::AccessControlError(format!("You don't have Admin role in Company {company_id}, hence, you cannot assign Admin role to other users")));
        }
    }

    let db_service = get_database_service().await;
    let mut transaction = db_service.new_transaction().await?;
    transaction.start_transaction().await?;

    let mut invite = db_entities::InviteAddCompany::new(
        inviting_user_id,
        invited_user_id,
        company_id,
        role,
        job_title,
        project_ids,
        None,
    );
    invite.save(Some(&mut transaction)).await?;

    let query_result = db_entities::Company::find_one(doc! {"_id": company_id}).await;
    if let Ok(Some(company)) = query_result {
        let mut notification = db_entities::AppNotification::new(
            invited_user_id,
            NotificationType::InviteAddCompany,
            format!("You have been invited to Company {}", company.name()),
            false,
            invite.get_id().cloned(),
        );
        notification.save(Some(&mut transaction)).await?;

        transaction.commit_transaction().await?;

        Ok(())
    } else {
        transaction.abort_transaction().await?;
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Company with id {} does not exist",
            company_id
        )))
    }
}

pub async fn get_pending_invited_users(
    company_id: &DocumentId,
) -> Result<Vec<InvitedUserInCompanyInfo>, ServiceAppError> {
    let pending_invitations =
        db_entities::InviteAddCompany::find_many(doc! {"company_id": company_id, "answer": null})
            .await?;

    #[derive(Serialize, Deserialize, Debug)]
    struct NotificationQueryResult {
        _id: DocumentId,
        entity_id: DocumentId,
    }

    let notifications_map = db_entities::AppNotification::find_many_projection::<NotificationQueryResult>(
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
    let usernames = db_entities::User::find_many_projection::<QueryResult>(
        doc! {"_id": {"$in": pending_invitations.iter().map(|doc| doc.invited_user_id()).collect::<Vec<&DocumentId>>()}},
        doc! {
            "username": 1,
            "_id": 1
        },
    )
    .await?.iter().map(|doc| (doc._id, doc.username.clone())).collect::<HashMap<DocumentId, String>>();

    let mut to_return = vec![];

    for invitation in pending_invitations {
        if let Some(username) = usernames.get(invitation.invited_user_id()) {
            let notification_id = *notifications_map
                .get(
                    invitation
                        .get_id()
                        .expect("id should exist from document retrieved from db"),
                )
                .expect("Expecting object id since it is read above");
            to_return.push(InvitedUserInCompanyInfo {
                notification_id: notification_id.to_hex(),
                user_id: invitation.invited_user_id().to_hex(),
                username: username.clone(),
                role: *invitation.company_role(),
                job_title: invitation.job_title().clone(),
                company_id: invitation.company_id().to_hex(),
            });
        } else {
            return Err(ServiceAppError::InternalServerError(format!(
                "User {} should exist",
                invitation.invited_user_id().to_hex()
            )));
        }
    }

    Ok(to_return)
}

pub async fn get_users_to_invite_in_company(
    company_id: DocumentId,
) -> Result<Vec<(DocumentId, String)>, ServiceAppError> {
    // Users can be invited to a company if they are not already in it and if there is no pending invitation

    #[derive(Serialize, Deserialize, Debug)]
    struct InvitedUsersQueryResult {
        invited_user_id: DocumentId,
    }
    let mut users_to_exclude: Vec<DocumentId> =
        db_entities::InviteAddCompany::find_many_projection::<InvitedUsersQueryResult>(
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
        db_entities::UserEmploymentContract::find_many_projection::<QueryResult>(
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
        db_entities::User::find_many_projection::<UserQueryResult>(
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
    company_id: &DocumentId,
) -> Result<Vec<db_entities::CompanyProject>, ServiceAppError> {
    db_entities::CompanyProject::find_many(doc! {"company_id": company_id}).await
}

pub async fn get_company_project_allocations(
    company_id: DocumentId,
) -> Result<HashMap<DocumentId, Vec<DocumentId>>, ServiceAppError> {
    #[derive(Serialize, Deserialize)]
    struct QueryResult {
        user_id: DocumentId,
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        project_ids: Vec<DocumentId>,
    }

    let assignments = db_entities::UserEmploymentContract::find_many_projection::<QueryResult>(
        doc! {"company_id": company_id},
        doc! {"user_id": 1, "project_ids": 1},
    )
    .await?;

    let mut to_return = HashMap::new();
    for assignment in assignments {
        for project_id in assignment.project_ids {
            to_return
                .entry(project_id)
                .or_insert_with(Vec::new)
                .push(assignment.user_id);
        }
    }

    Ok(to_return)
}

pub async fn create_project(
    company_id: DocumentId,
    name: String,
    code: String,
) -> Result<String, ServiceAppError> {
    let company_projects =
        db_entities::CompanyProject::find_many(doc! {"company_id": company_id}).await?;

    // project name and code must be unique
    for project in company_projects {
        if *project.name() == name || *project.code() == code {
            return Err(ServiceAppError::InvalidRequest(format!(
                "Project name and code must be unique got name: {} and code: {}",
                name, code
            )));
        }
    }

    let mut new_project = db_entities::CompanyProject::new(name, code, company_id, true);

    new_project.save(None).await
}

pub async fn edit_project(
    company_id: DocumentId,
    project_id: DocumentId,
    name: String,
    code: String,
    active: bool,
) -> Result<String, ServiceAppError> {
    // TODO: instead of loading all the projects documents, load a project with id and then use update_one()

    let company_project_query =
        db_entities::CompanyProject::find_one(doc! {"_id": project_id, "company_id": company_id})
            .await?;

    if let Some(mut company_project) = company_project_query {
        let company_projects = db_entities::CompanyProject::find_many(
            doc! {"company_id": company_id, "_id": {"$ne": project_id}},
        )
        .await?;

        // project name and code must be unique
        for project in company_projects {
            if *project.name() == name || *project.code() == code {
                return Err(ServiceAppError::InvalidRequest(format!(
                    "Project name and code must be unique got name: {} and code: {}",
                    name, code
                )));
            }
        }

        company_project.set_name(name);
        company_project.set_code(code);
        company_project.set_active(active);
        company_project.save(None).await
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Project with id {} does not exist",
            project_id
        )))
    }
}

pub async fn delete_project(
    company_id: DocumentId,
    project_id: DocumentId,
) -> Result<(), ServiceAppError> {
    let company_project_query =
        db_entities::CompanyProject::find_one(doc! {"_id": project_id, "company_id": company_id})
            .await?;

    if let Some(company_project) = company_project_query {
        // a project can be deleted only if it has no users
        let n_allocations = db_entities::UserEmploymentContract::count_documents(doc! {
            "company_id": company_id,
            "project_ids": project_id
        })
        .await?;

        if n_allocations == 0 {
            company_project.delete(None).await
        } else {
            Err(ServiceAppError::InvalidRequest(format!(
                "Project with id {} is used in your company and cannot  be deleted",
                project_id
            )))
        }
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Project with id {} does not exist",
            project_id
        )))
    }
}

pub async fn edit_company_project_allocations(
    company_id: DocumentId,
    project_id: DocumentId,
    user_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    // TODO: change this with Project's Work Packages
    let project = db_entities::CompanyProject::find_one(doc! {
        "_id": project_id,
        "company_id": company_id,
    })
    .await?;

    if project.is_some() {
        // for each assignment that contains the project_id but the user is not in user_ids
        // we remove the project id from the project_ids list of the assignment

        let mut assignments = db_entities::UserEmploymentContract::find_many(
            doc! { "company_id": company_id, "project_ids": project_id},
        )
        .await?;

        let db_service = get_database_service().await;
        let mut transaction = db_service.new_transaction().await?;
        transaction.start_transaction().await?;

        let mut handled_users = vec![];

        for assignment in assignments.iter_mut() {
            if !user_ids.contains(assignment.user_id()) {
                //assignment.project_ids_mut().retain(|id| id != &project_id);
                assignment.save(Some(&mut transaction)).await?;
            } else {
                // we store the users in the list that are already in the project
                // to ignore them in the next step in which we add the project id
                // to the user assignments
                handled_users.push(assignment.user_id());
            }
        }

        // For each user id in user_ids that is not in handled_users we retrieve the assignment and
        // add the project id to the project_ids list
        let remaining_users: Vec<ObjectId> = user_ids
            .into_iter()
            .filter(|user| !handled_users.contains(&user))
            .collect();
        let mut new_assignments = db_entities::UserEmploymentContract::find_many(doc! {
            "company_id": company_id,
            "user_id": {"$in": remaining_users.into_iter().map(Bson::ObjectId).collect::<Vec<Bson>>()} 
        })
        .await?;

        for assignment in new_assignments.iter_mut() {
            //assignment.project_ids_mut().push(project_id);
            assignment.save(Some(&mut transaction)).await?;
        }

        transaction.commit_transaction().await?;

        Ok(())
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Project with id {} does not exist",
            project_id
        )))
    }
}

pub async fn edit_company_project_allocations_for_user(
    company_id: DocumentId,
    user_id: DocumentId,
    project_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    // TODO: change this using Project's Work Packages
    let assignment = db_entities::UserEmploymentContract::find_one(
        doc! { "company_id": company_id, "user_id": user_id},
    )
    .await?;
    if let Some(mut assignment) = assignment {
        //assignment.set_project_ids(project_ids);
        assignment.save(None).await?;
        Ok(())
    } else {
        Err(ServiceAppError::InvalidRequest(format!(
            "User with id {user_id} is not in the company with id {company_id}"
        )))
    }
}

pub async fn get_company_project_activities(
    company_id: DocumentId,
) -> Result<Vec<db_entities::ProjectActivity>, ServiceAppError> {
    db_entities::ProjectActivity::find_many(doc! {"company_id": company_id}).await
}

pub async fn get_projects_with_activity(
    activity_id: DocumentId,
) -> Result<Vec<String>, ServiceAppError> {
    #[derive(Serialize, Deserialize)]
    struct QueryResult {
        project_id: DocumentId,
    }

    Ok(
        db_entities::WPActivityAssignment::find_many_projection::<QueryResult>(
            doc! {"activity_ids": activity_id},
            doc! {"project_id": 1},
        )
        .await?
        .into_iter()
        .map(|elem| elem.project_id.to_hex())
        .collect::<Vec<String>>(),
    )
}

pub async fn get_activities_by_id(
    activity_ids: &Vec<DocumentId>,
) -> Result<Vec<db_entities::ProjectActivity>, ServiceAppError> {
    db_entities::ProjectActivity::find_many(doc! {"_id": {"$in": activity_ids}}).await
}

pub async fn get_projects_activity_assignment(
    project_id: &DocumentId,
) -> Result<Vec<DocumentId>, ServiceAppError> {
    if let Some(assignment) =
        db_entities::WPActivityAssignment::find_one(doc! {"project_id": project_id}).await?
    {
        Ok(assignment.activity_ids().clone())
    } else {
        // If there is not assignment then we return an empty list
        Ok(vec![])
    }
}

pub async fn create_company_project_activity(
    company_id: DocumentId,
    name: String,
    description: String,
) -> Result<(), ServiceAppError> {
    // First check if the name does not exist yet for this company
    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        name: String,
    }
    if db_entities::ProjectActivity::find_one_projection::<QueryResult>(
        doc! {
            "name": name.clone(),
            "company_id": company_id
        },
        doc! {"name": 1},
    )
    .await?
    .is_some()
    {
        Err(ServiceAppError::InvalidRequest(format!(
            "Activity with name {name} already exist for company with id {company_id}"
        )))
    } else {
        db_entities::ProjectActivity::new(name, description, company_id)
            .save(None)
            .await?;
        Ok(())
    }
}

pub async fn edit_company_project_activity(
    company_id: DocumentId,
    activity_id: DocumentId,
    name: String,
    description: String,
) -> Result<(), ServiceAppError> {
    // First check if the name does not already exist for this company
    if db_entities::ProjectActivity::find_one(doc! {"name": &name, "_id": {"$ne": activity_id}})
        .await?
        .is_some()
    {
        return Err(ServiceAppError::InvalidRequest(format!(
            "Activity with name {name} already exist for company with id {company_id}"
        )));
    }

    if let Some(mut activity) = db_entities::ProjectActivity::find_one(doc! {
        "_id": activity_id, "company_id": company_id
    })
    .await?
    {
        activity.set_name(name);
        activity.set_description(description);
        activity.save(None).await?;
        Ok(())
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Activity with id {activity_id} does not exist for company with id {company_id}"
        )))
    }
}

/// Deletes a project activity from the company, it can be deleted
/// only if it is not used in any timesheet
pub async fn delete_company_project_activity(
    company_id: DocumentId,
    activity_id: DocumentId,
) -> Result<(), ServiceAppError> {
    if let Some(activity) = db_entities::ProjectActivity::find_one(doc! {
        "_id": activity_id, "company_id": company_id
    })
    .await?
    {
        // To safely delete the activity we need to check if it is used by in some timesheet
        #[derive(Serialize, Deserialize, Debug)]
        struct QueryResult {
            _id: DocumentId,
        }
        let query_result = db_entities::TimesheetDay::find_many_projection::<QueryResult>(
            doc! { "activities.activity_id": activity_id },
            doc! { "_id": 1},
        )
        .await?;

        if query_result.is_empty() {
            // we can safely delete it
            activity.delete(None).await?;
            Ok(())
        } else {
            Err(ServiceAppError::InvalidRequest(format!("Cannot delete the activity with id {activity_id} because it is used in a timesheet. Please just remove it from your Projects.")))
        }
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Activity with id {activity_id} does not exist for company with id {company_id}"
        )))
    }
}

pub async fn edit_project_activity_assignment(
    company_id: DocumentId,
    project_id: DocumentId,
    activity_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    let project = db_entities::CompanyProject::find_one(doc! {
        "_id": project_id,
        "company_id": company_id,
    })
    .await?;

    if project.is_some() {
        // if the project assignment document does not exist we create it, otherwise we update it
        let mut assignments_doc = if let Some(mut doc) =
            db_entities::WPActivityAssignment::find_one(doc! { "project_id": project_id}).await?
        {
            doc.set_activity_ids(activity_ids);
            doc
        } else {
            db_entities::WPActivityAssignment::new(project_id, activity_ids)
        };
        assignments_doc.save(None).await?;
        Ok(())
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Project with id {project_id} does not exist for company {company_id}"
        )))
    }
}

pub async fn edit_project_activity_assignment_by_activity(
    activity_id: DocumentId,
    project_ids: Vec<DocumentId>,
) -> Result<(), ServiceAppError> {
    let activity = db_entities::ProjectActivity::find_one(doc! {"_id": activity_id}).await?;
    if activity.is_some() {
        /* let mut assignments =
            db_entities::WPActivityAssignment::find_many(doc! {"activity_ids": activity_id})
                .await?;
        let db_service = get_database_service().await;
        let mut transaction = db_service.new_transaction().await?;
        transaction.start_transaction().await?;

        let mut handled_projects = vec![];

        for assignment in assignments.iter_mut() {
            if !project_ids.contains(assignment.project_id()) {
                // we remove the activity from the list because it is not present anymore
                assignment
                    .activity_ids_mut()
                    .retain(|id| id != &activity_id);
                assignment.save(Some(&mut transaction)).await?;
            } else {
                // the activity is still in the list so we do nothing
                handled_projects.push(assignment.project_id())
            }
        }

        // for each project id in project_ids that is not in handled_projects we try to retrieve the assignment
        // if it is present we add activity_id to the list otherwise we create a new document
        let remaining_projects: Vec<ObjectId> = project_ids
            .into_iter()
            .filter(|project| !handled_projects.contains(&project))
            .collect();

        for project in remaining_projects {
            let mut assignments_doc = if let Some(mut doc) =
                db_entities::WPActivityAssignment::find_one(doc! { "project_id": project}).await?
            {
                doc.activity_ids_mut().push(activity_id);
                doc
            } else {
                db_entities::WPActivityAssignment::new(project, vec![activity_id])
            };
            assignments_doc.save(Some(&mut transaction)).await?;
        }

        transaction.commit_transaction().await?; */
        Ok(())
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Activity with id {activity_id} does not exist",
        )))
    }
}

/*
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::{DateTime, Utc};
    use mongodb::bson::{doc, oid::ObjectId};

    use crate::{
        enums::CompanyRole,
        model::db_entities,
        service::{
            company::{
                create_company, create_user_contract_with_company,
                delete_user_contract_with_company, get_user_companies, get_user_company,
                update_user_contract_with_company,
            },
            db::{document::DatabaseDocument, get_database_service},
        },
    };

    use super::delete_company_project_activity;

    #[tokio::test]
    async fn create_company_test() {
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

        let job_title = "CEO".to_string();
        let name = "My Company".to_string();
        let result = create_company(&user_id, name.clone(), job_title).await;
        assert!(result.is_ok());

        let assignment = db_entities::UserEmploymentContract::find_one(doc! {"user_id": user_id})
            .await
            .unwrap();
        assert!(assignment.is_some());

        let companies = db_entities::Company::find_many(doc! {}).await.unwrap();
        assert!(*companies.get(0).unwrap().name() == name);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn get_user_companies_test() {
        let mut company = db_entities::Company::new("My Company".into(), true);
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            true,
        );
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();
        let mut first_assignment = db_entities::UserEmploymentContract::new(
            first_user_id.clone(),
            company_id,
            crate::enums::CompanyRole::Owner,
            "CEO".into(),
            vec![],
        );
        first_assignment.save(None).await.unwrap();
        let mut second_user = db_entities::User::new(
            "river.pond@mail.com".into(),
            "riverpond".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            true,
        );
        let second_user_id = ObjectId::from_str(&second_user.save(None).await.unwrap()).unwrap();
        let mut second_assignment = db_entities::UserEmploymentContract::new(
            second_user_id.clone(),
            company_id,
            crate::enums::CompanyRole::User,
            "Developer".into(),
            vec![],
        );
        second_assignment.save(None).await.unwrap();

        let result = get_user_companies(&first_user_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().first().unwrap().name(), company.name());

        let result = get_user_company(&second_user_id, &company_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name(), company.name());

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn add_user_to_company_test() {
        let mut company = db_entities::Company::new("My Company".into(), true);
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            true,
        );
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();

        let result = create_user_contract_with_company(
            first_user_id,
            company_id,
            CompanyRole::User,
            "CTO".into(),
            vec![],
        )
        .await;
        assert!(result.is_ok());

        let assignment = db_entities::UserEmploymentContract::find_one(doc! {})
            .await
            .unwrap()
            .unwrap();

        assert_eq!(*assignment.company_id(), company_id);
        assert_eq!(*assignment.user_id(), first_user_id);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn remove_user_from_company_test() {
        let mut company = db_entities::Company::new("My Company".into(), true);
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            true,
        );
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();
        let mut first_assignment = db_entities::UserEmploymentContract::new(
            first_user_id.clone(),
            company_id,
            crate::enums::CompanyRole::Owner,
            "CEO".into(),
            vec![],
        );
        first_assignment.save(None).await.unwrap();

        let result = delete_user_contract_with_company(&first_user_id, &company_id).await;
        assert!(result.is_ok());

        assert!(db_entities::UserEmploymentContract::find_one(doc! {})
            .await
            .unwrap()
            .is_none());

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn update_user_in_company_test() {
        let mut company = db_entities::Company::new("My Company".into(), true);
        let company_id = ObjectId::from_str(&company.save(None).await.unwrap()).unwrap();
        let mut first_user = db_entities::User::new(
            "john.smith@mail.com".into(),
            "johnsmith".into(),
            "fdsg39av2".into(),
            "John".into(),
            "Smith".into(),
            Some("api_key".into()),
            false,
            true,
        );
        let first_user_id = ObjectId::from_str(&first_user.save(None).await.unwrap()).unwrap();
        let mut first_assignment = db_entities::UserEmploymentContract::new(
            first_user_id.clone(),
            company_id,
            crate::enums::CompanyRole::User,
            "CEO".into(),
            vec![],
        );
        first_assignment.save(None).await.unwrap();

        let new_job_title = "CIO".to_string();
        let result = update_user_contract_with_company(
            &first_user_id,
            &company_id,
            None,
            Some(new_job_title.clone()),
        )
        .await;
        assert!(result.is_ok());

        let assignment = db_entities::UserEmploymentContract::find_one(doc! {})
            .await
            .unwrap()
            .unwrap();

        assert_eq!(*assignment.company_id(), company_id);
        assert_eq!(*assignment.user_id(), first_user_id);
        assert_eq!(*assignment.job_title(), new_job_title);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn delete_company_project_activity_test() {
        let company_id = ObjectId::new();
        let mut activity = db_entities::ProjectActivity::new(
            "my_activity".into(),
            "description".into(),
            company_id.clone(),
        );
        activity.save(None).await.unwrap();
        let mut second_activity = db_entities::ProjectActivity::new(
            "my_activity_2".into(),
            "description".into(),
            company_id.clone(),
        );
        second_activity.save(None).await.unwrap();

        let mut timesheet_day = db_entities::TimesheetDay::new(
            ObjectId::new(),
            DateTime::<Utc>::default(),
            0,
            crate::enums::WorkingDayType::Office,
            vec![db_entities::TimesheetActivityHours::new(
                company_id.clone(),
                ObjectId::new(),
                *activity.get_id().unwrap(),
                "description".into(),
                1,
            )],
        );
        timesheet_day.save(None).await.unwrap();

        let result = delete_company_project_activity(company_id, *activity.get_id().unwrap()).await;
        assert!(result.is_err());
        assert_eq!(
            db_entities::ProjectActivity::find_many(doc! {})
                .await
                .unwrap()
                .len(),
            2
        );
        let result =
            delete_company_project_activity(company_id, *second_activity.get_id().unwrap()).await;
        assert!(result.is_ok());
        assert_eq!(
            db_entities::ProjectActivity::find_many(doc! {})
                .await
                .unwrap()
                .len(),
            1
        );
        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
*/
