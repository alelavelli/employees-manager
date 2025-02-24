use chrono::{DateTime, TimeZone, Utc};

use crate::{
    enums::WorkingDayType,
    error::ServiceAppError,
    model::{db_entities, internal::TimesheetActivityHours},
    DocumentId,
};
use mongodb::bson::doc;

use super::db::DatabaseDocument;

/// Create or update a timesheet day.
///
/// If an entry in the database exists for the tuple (user_id, date) then it is entirely
/// updated with the given parameters.
/// Otherwise, it is created as a new document.
pub async fn create_day(
    user_id: DocumentId,
    date: DateTime<Utc>,
    permit_hours: u32,
    working_type: WorkingDayType,
    activities: Vec<TimesheetActivityHours>,
) -> Result<(), ServiceAppError> {
    let count =
        db_entities::TimesheetDay::count_documents(doc! {"user_id": user_id, "date": date}).await?;
    match count {
        0 => {
            let mut new_document = db_entities::TimesheetDay::new(
                user_id,
                date,
                permit_hours,
                working_type,
                activities
                    .into_iter()
                    .map(|e| e.into())
                    .collect::<Vec<db_entities::TimesheetActivityHours>>(),
            );
            new_document.save(None).await?;
            Ok(())
        }
        1 => {
            db_entities::TimesheetDay::update_one(
                doc! {"user_id": user_id, "date": date},
                doc! {
                    "$set": {
                        "permit_hours": permit_hours,
                        "working_type": working_type,
                        "activities": activities.into_iter()
                        .map(|e| e.into())
                        .collect::<Vec<db_entities::TimesheetActivityHours>>()
                    }
                },
                None,
            )
            .await?;
            Ok(())
        }
        _ => Err(ServiceAppError::InternalServerError(format!(
            "There are more than one database documents for user_id {user_id} and date {:?}",
            date.to_string()
        ))),
    }
}

/// Returns the timesheet days for the user and the month passed as parameters
pub async fn get_days(
    user_id: DocumentId,
    year: i32,
    month: u32,
) -> Result<Vec<db_entities::TimesheetDay>, ServiceAppError> {
    let from_date = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).earliest();
    let to_date = Utc
        .with_ymd_and_hms(year, month + 1, 1, 0, 0, 0)
        .earliest()
        .or_else(|| Utc.with_ymd_and_hms(year + 1, month, 0, 0, 0, 0).earliest());
    if from_date.is_none() || to_date.is_none() {
        Err(ServiceAppError::InvalidRequest(format!(
            "Invalid year and month. Got year: {year} and month {month}"
        )))
    } else {
        let from_date = from_date.unwrap();
        let to_date = to_date.unwrap();
        db_entities::TimesheetDay::find_many(doc! {
            "user_id": user_id,
            "date": {"$lt": to_date, "$gte": from_date},
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use bson::doc;
    use chrono::{TimeZone, Utc};

    use crate::{
        model::{db_entities, internal::TimesheetActivityHours},
        service::{
            db::{get_database_service, DatabaseDocument},
            timesheet::{create_day, get_days},
        },
        DocumentId,
    };

    #[tokio::test]
    async fn create_and_get_day_test() {
        let user_id = DocumentId::new();
        let first_day = Utc
            .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
            .earliest()
            .unwrap();
        let second_day = Utc
            .with_ymd_and_hms(2025, 1, 2, 0, 0, 0)
            .earliest()
            .unwrap();
        let third_day = Utc
            .with_ymd_and_hms(2025, 3, 2, 0, 0, 0)
            .earliest()
            .unwrap();
        let result = create_day(
            user_id,
            first_day,
            6,
            crate::enums::WorkingDayType::Office,
            vec![
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my second description".into(),
                    hours: 4,
                },
            ],
        )
        .await;
        assert!(result.is_ok());

        let doc = db_entities::TimesheetDay::find_one(doc! {})
            .await
            .unwrap()
            .unwrap();
        assert_eq!(doc.working_type(), &crate::enums::WorkingDayType::Office);

        let result = create_day(
            user_id,
            second_day,
            6,
            crate::enums::WorkingDayType::Office,
            vec![
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my second description".into(),
                    hours: 4,
                },
            ],
        )
        .await;
        assert!(result.is_ok());

        let result = create_day(
            user_id,
            third_day,
            6,
            crate::enums::WorkingDayType::Office,
            vec![
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my second description".into(),
                    hours: 4,
                },
            ],
        )
        .await;
        assert!(result.is_ok());

        let docs = get_days(user_id, 2025, 1).await.unwrap();
        assert_eq!(docs.len(), 2);

        let docs = get_days(user_id, 2025, 2).await.unwrap();
        assert_eq!(docs.len(), 0);

        let docs = get_days(user_id, 2025, 3).await.unwrap();
        assert_eq!(docs.len(), 1);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
