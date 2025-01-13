use anyhow::anyhow;
use mongodb::bson::doc;

use crate::{
    error::AppError,
    model::db_entities,
    service::{company, db::get_database_service},
    DocumentId,
};

use super::db::DatabaseDocument;

pub async fn get_unread_notifications(
    user_id: &DocumentId,
) -> Result<Vec<db_entities::AppNotification>, AppError> {
    db_entities::AppNotification::find_many(doc! {"user_id": user_id}).await
}

pub async fn get_notification(
    notification_id: &DocumentId,
) -> Result<Option<db_entities::AppNotification>, AppError> {
    db_entities::AppNotification::find_one::<db_entities::AppNotification>(
        doc! {"_id": notification_id},
    )
    .await
}

pub async fn answer_to_invite_add_company(
    mut notification: db_entities::AppNotification,
    answer: bool,
) -> Result<(), AppError> {
    let db_service = get_database_service().await;
    let mut transaction = db_service.new_transaction().await?;
    transaction.start_transaction().await?;

    notification.read = true;
    notification.save(Some(&mut transaction)).await?;

    if let Some(entity_id) = notification.entity_id {
        db_entities::InviteAddCompany::update_one(
            doc! {"_id": entity_id},
            doc! { "$set": { "answer":  answer}},
            Some(&mut transaction),
        )
        .await?;

        if answer {
            let invite_add_company_doc_result = db_entities::InviteAddCompany::find_one::<
                db_entities::InviteAddCompany,
            >(doc! {"_id": notification.entity_id})
            .await?;
            if let Some(invite_add_company) = invite_add_company_doc_result {
                company::add_user_to_company(
                    invite_add_company.invited_user_id,
                    invite_add_company.company_id,
                    invite_add_company.company_role,
                    invite_add_company.job_title,
                )
                .await?;
            } else {
                transaction.abort_transaction().await?;
                return Err(AppError::InternalServerError(anyhow!(format!(
                    "Error in adding user to company for notification with id {:?}, InviteAddCompany document not found",
                    notification.get_id()
                ))));
            }
        }
    } else {
        transaction.abort_transaction().await?;
        return Err(AppError::InternalServerError(anyhow!(format!(
            "Notification with id {:?} does not not contain entity id",
            notification.get_id()
        ))));
    }

    transaction.commit_transaction().await?;
    Ok(())
}
