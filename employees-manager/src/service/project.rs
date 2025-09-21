use bson::doc;

use crate::{
    error::ServiceAppError,
    model::db_entities,
    service::db::{
        document::{DatabaseDocument, SmartDocumentReference},
        transaction::DatabaseTransaction,
    },
};

pub async fn delete_project(
    project_id: SmartDocumentReference<db_entities::CompanyProject>,
    transaction: Option<&mut DatabaseTransaction>,
) -> Result<Option<&mut DatabaseTransaction>, ServiceAppError> {
    // Understand how to handle the creation of a transaction here
    let mut transaction = transaction;

    let project_id = project_id.to_id();

    // Delete associations of users with the project
    transaction =
        db_entities::UserProjects::delete_many(doc! {"project_id": project_id}, transaction)
            .await?;

    // Delete invite add company
    transaction =
        db_entities::InviteAddCompany::delete_many(doc! {"project_ids": project_id}, transaction)
            .await?;

    // Delete work package
    transaction =
        db_entities::WorkPackage::delete_many(doc! {"project_id": project_id}, transaction).await?;

    // Delete timesheet activity hours
    transaction = db_entities::TimesheetDay::delete_many(
        doc! {"activities.project_id": project_id},
        transaction,
    )
    .await?;

    Ok(transaction)
}
