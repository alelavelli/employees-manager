use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    enums::NotificationType,
    error::ServiceAppError,
    model::db_entities,
    service::{company, db::get_database_service},
    DocumentId,
};

use super::db::DatabaseDocument;

pub async fn get_unread_notifications(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::AppNotification>, ServiceAppError> {
    db_entities::AppNotification::find_many(doc! {"user_id": user_id, "read": false}).await
}

pub async fn get_notification(
    notification_id: &DocumentId,
) -> Result<Option<db_entities::AppNotification>, ServiceAppError> {
    db_entities::AppNotification::find_one(doc! {"_id": notification_id}).await
}

pub async fn set_notification_as_read(
    mut notification: db_entities::AppNotification,
) -> Result<(), ServiceAppError> {
    notification.set_read(true);
    notification.save(None).await?;
    Ok(())
}

pub async fn answer_to_invite_add_company(
    mut notification: db_entities::AppNotification,
    answer: bool,
) -> Result<(), ServiceAppError> {
    let db_service = get_database_service().await;
    let mut transaction = db_service.new_transaction().await?;
    transaction.start_transaction().await?;

    notification.set_read(true);
    notification.save(Some(&mut transaction)).await?;

    if let Some(entity_id) = notification.entity_id() {
        db_entities::InviteAddCompany::update_one(
            doc! {"_id": entity_id},
            doc! { "$set": { "answer":  answer}},
            Some(&mut transaction),
        )
        .await?;

        let invite_add_company_doc_result =
            db_entities::InviteAddCompany::find_one(doc! {"_id": notification.entity_id()}).await?;
        if let Some(invite_add_company) = invite_add_company_doc_result {
            if answer {
                company::add_user_to_company(
                    *invite_add_company.invited_user_id(),
                    *invite_add_company.company_id(),
                    *invite_add_company.company_role(),
                    invite_add_company.job_title().clone(),
                    invite_add_company.project_ids().clone(),
                )
                .await?;
            }

            // create new notification for the inviting user with the answer
            #[derive(Serialize, Deserialize, Debug)]
            struct UserQueryResult {
                username: String,
            }
            let invited_username = db_entities::User::find_one_projection::<UserQueryResult>(
                doc! {"_id": invite_add_company.invited_user_id()},
                doc! {"username": 1},
            )
            .await?
            .expect("excepted user in database")
            .username;
            #[derive(Serialize, Deserialize, Debug)]
            struct CompanyQueryResult {
                name: String,
            }
            let company_name = db_entities::Company::find_one_projection::<CompanyQueryResult>(
                doc! {"_id": invite_add_company.company_id()},
                doc! {"name": 1},
            )
            .await?
            .expect("expected company in database")
            .name;
            let message = if answer {
                format!(
                    "User {} has accepted to join in company {}",
                    invited_username, company_name
                )
            } else {
                format!(
                    "User {} has declined to join in company {}",
                    invited_username, company_name
                )
            };
            let mut answer_notification = db_entities::AppNotification::new(
                *invite_add_company.inviting_user_id(),
                NotificationType::InviteAddCompanyAnswer,
                message,
                false,
                invite_add_company.get_id().copied(),
            );
            answer_notification.save(Some(&mut transaction)).await?;
        } else {
            transaction.abort_transaction().await?;
            return Err(ServiceAppError::InternalServerError(format!(
                    "Error in adding user to company for notification with id {:?}, InviteAddCompany document not found",
                    notification.get_id()
                )));
        }
    } else {
        transaction.abort_transaction().await?;
        return Err(ServiceAppError::InternalServerError(format!(
            "Notification with id {:?} does not not contain entity id",
            notification.get_id()
        )));
    }

    transaction.commit_transaction().await?;
    Ok(())
}

pub async fn cancel_invite_user_to_company(
    notification_id: DocumentId,
) -> Result<(), ServiceAppError> {
    if let Some(notification) =
        db_entities::AppNotification::find_one(doc! {"_id": notification_id}).await?
    {
        if let Some(entity_id) = notification.entity_id() {
            if let Some(invitation) = db_entities::InviteAddCompany::find_one(doc! {
                "_id": entity_id
            })
            .await?
            {
                let db_service = get_database_service().await;
                let mut transaction = db_service.new_transaction().await?;
                transaction.start_transaction().await?;

                invitation.delete(Some(&mut transaction)).await?;
                notification.delete(Some(&mut transaction)).await?;

                transaction.commit_transaction().await?;

                Ok(())
            } else {
                Err(ServiceAppError::InternalServerError(format!(
                    "InviteAddCompany with id {entity_id} does not exist"
                )))
            }
        } else {
            Err(ServiceAppError::EntityDoesNotExist(format!(
                "Notification with id {notification_id} does not exist"
            )))
        }
    } else {
        Err(ServiceAppError::EntityDoesNotExist(format!(
            "Notification with id {notification_id} does not exist"
        )))
    }
}
