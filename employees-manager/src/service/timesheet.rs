use std::collections::{hash_map::Entry, HashMap};

use chrono::{DateTime, TimeZone, Utc};
use rust_xlsxwriter::{workbook::Workbook, Format, FormatAlign};

use crate::{
    enums::WorkingDayType,
    error::ServiceAppError,
    model::{db_entities, internal::TimesheetActivityHours},
    DocumentId,
};
use mongodb::bson::doc;

use super::db::document::DatabaseDocument;

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
    user_id: &DocumentId,
    year: &i32,
    month: &u32,
) -> Result<Vec<db_entities::TimesheetDay>, ServiceAppError> {
    let from_date = Utc.with_ymd_and_hms(*year, *month, 1, 0, 0, 0).earliest();
    let to_date = Utc
        .with_ymd_and_hms(*year, *month + 1, 1, 0, 0, 0)
        .earliest()
        .or_else(|| {
            Utc.with_ymd_and_hms(*year + 1, *month, 0, 0, 0, 0)
                .earliest()
        });
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

/// Export the timesheet as excel file
///
/// The file contains the columns:
/// - date
/// - work type
/// - permission hours
/// - company
/// - project
/// - activity
/// - hours
/// - notes
///
pub async fn export_as_excel(
    user_id: &DocumentId,
    year: &i32,
    month: &u32,
) -> Result<Vec<u8>, ServiceAppError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let bold_format = Format::new().set_bold();
    let date_format = Format::new().set_num_format("yyyy-mm-dd");
    let merge_format = Format::new().set_align(FormatAlign::Center);

    // create columns
    let columns = [
        "Date",
        "Work Type",
        "Permission hours",
        "Company",
        "Project",
        "Activity",
        "Hours",
        "Notes",
    ];
    for (column_index, &column) in columns.iter().enumerate() {
        worksheet.write_with_format(0, column_index as u16, column, &bold_format)?;
    }

    // create a caches to store temporary company, project and activity names read from db
    // to avoid useless reads
    let mut company_cache: HashMap<&DocumentId, db_entities::Company> = HashMap::new();
    let mut project_cache: HashMap<&DocumentId, db_entities::CompanyProject> = HashMap::new();
    let mut activity_cache: HashMap<&DocumentId, db_entities::ProjectActivity> = HashMap::new();

    let mut current_row = 1;
    let timesheet_days = get_days(user_id, year, month).await?;
    for timesheet_day in timesheet_days.iter() {
        let initial_activities_row = current_row;
        for activity_doc in timesheet_day.activities() {
            // Here, we cannot use or_insert_with because we cannot use async closure that are considered unsafe
            // therefore, we check if the entry exist and if not we perform the query

            if let Entry::Vacant(entry) = company_cache.entry(activity_doc.company_id()) {
                entry.insert(
                db_entities::Company::find_one(
                    doc! {"_id": activity_doc.company_id()})
                    .await
                    .map_err(|e| ServiceAppError::InternalServerError(format!("An error occurred when retrieving the company document with id {}. Got error {}", activity_doc.company_id(), e)))?
                    .ok_or(ServiceAppError::EntityDoesNotExist(format!("Company with id {} does not exist.", activity_doc.company_id())))?
            );
            }
            let company_name = company_cache
                .get(activity_doc.company_id())
                .ok_or(ServiceAppError::InternalServerError(format!(
                    "Company entry with id {} should exist in the cache",
                    activity_doc.company_id()
                )))?
                .name();

            if let Entry::Vacant(entry) = project_cache.entry(activity_doc.project_id()) {
                entry.insert(
                db_entities::CompanyProject::find_one(
                    doc! {"_id": activity_doc.project_id()})
                    .await
                    .map_err(|e| ServiceAppError::InternalServerError(format!("An error occurred when retrieving the project document with id {}. Got error {}", activity_doc.project_id(), e)))?
                    .ok_or(ServiceAppError::EntityDoesNotExist(format!("Project with id {} does not exist.", activity_doc.project_id())))?
            );
            }
            let project_name = project_cache
                .get(activity_doc.project_id())
                .ok_or(ServiceAppError::InternalServerError(format!(
                    "Project entry with id {} should exist in the cache",
                    activity_doc.project_id()
                )))?
                .name();

            if let Entry::Vacant(entry) = activity_cache.entry(activity_doc.activity_id()) {
                entry.insert(
                    db_entities::ProjectActivity::find_one(
                        doc! {"_id": activity_doc.activity_id()})
                        .await
                        .map_err(|e| ServiceAppError::InternalServerError(format!("An error occurred when retrieving the activity document with id {}. Got error {}", activity_doc.activity_id(), e)))?
                        .ok_or(ServiceAppError::EntityDoesNotExist(format!("Activity with id {} does not exist.", activity_doc.activity_id())))?
                );
            }
            let activity_name = activity_cache
                .get(activity_doc.activity_id())
                .ok_or(ServiceAppError::InternalServerError(format!(
                    "Activity entry with id {} should exist in the cache",
                    activity_doc.activity_id()
                )))?
                .name();

            worksheet.write(current_row, 3, company_name)?;
            worksheet.write(current_row, 4, project_name)?;
            worksheet.write(current_row, 5, activity_name)?;
            worksheet.write(current_row, 6, *activity_doc.hours())?;
            worksheet.write(current_row, 7, activity_doc.notes())?;

            current_row += 1;
        }
        // To write date we need to first merge cells with an empty string and then write with the right format
        if initial_activities_row != (current_row - 1) {
            worksheet.merge_range(
                initial_activities_row,
                0,
                current_row - 1,
                0,
                "",
                &merge_format,
            )?;
        }
        worksheet.write_with_format(
            initial_activities_row,
            0,
            &timesheet_day.date().date_naive(),
            &date_format,
        )?;
        // We can write working type directly as string
        if initial_activities_row != (current_row - 1) {
            worksheet.merge_range(
                initial_activities_row,
                1,
                current_row - 1,
                1,
                &timesheet_day.working_type().to_string(),
                &merge_format,
            )?;
        } else {
            worksheet.write(
                initial_activities_row,
                1,
                timesheet_day.working_type().to_string(),
            )?;
        }
        // To write hours we need to first merge cells with an empty string and then write with the right format

        if initial_activities_row != (current_row - 1) {
            worksheet.merge_range(
                initial_activities_row,
                2,
                current_row - 1,
                2,
                "",
                &merge_format,
            )?;
        }
        worksheet.write(initial_activities_row, 2, *timesheet_day.permit_hours())?;
    }

    Ok(workbook.save_to_buffer()?)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bson::{doc, oid::ObjectId};
    use chrono::{TimeZone, Utc};

    use crate::{
        model::{
            db_entities::{self, Company, CompanyProject, ProjectActivity, WorkPackage},
            internal::TimesheetActivityHours,
        },
        service::{
            db::{document::DatabaseDocument, get_database_service},
            timesheet::{create_day, export_as_excel, get_days},
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
                    work_package_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    work_package_id: DocumentId::new(),
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
                    work_package_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    work_package_id: DocumentId::new(),
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
                    work_package_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: DocumentId::new(),
                    project_id: DocumentId::new(),
                    work_package_id: DocumentId::new(),
                    activity_id: DocumentId::new(),
                    notes: "this is my second description".into(),
                    hours: 4,
                },
            ],
        )
        .await;
        assert!(result.is_ok());

        let docs = get_days(&user_id, &2025, &1).await.unwrap();
        assert_eq!(docs.len(), 2);

        let docs = get_days(&user_id, &2025, &2).await.unwrap();
        assert_eq!(docs.len(), 0);

        let docs = get_days(&user_id, &2025, &3).await.unwrap();
        assert_eq!(docs.len(), 1);

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }

    #[tokio::test]
    async fn excel_export_test() {
        let mut first_company = Company::new("First company".into(), true);
        let first_company_id =
            ObjectId::from_str(&first_company.save(None).await.unwrap()).unwrap();
        let mut second_company = Company::new("Second company".into(), true);
        let second_company_id =
            ObjectId::from_str(&second_company.save(None).await.unwrap()).unwrap();

        let mut first_project = CompanyProject::new(
            "First project".into(),
            "first project code".into(),
            first_company_id.clone(),
            true,
        );
        let first_project_id =
            ObjectId::from_str(&first_project.save(None).await.unwrap()).unwrap();

        let mut second_project = CompanyProject::new(
            "Second project".into(),
            "second project code".into(),
            second_company_id.clone(),
            true,
        );
        let second_project_id =
            ObjectId::from_str(&second_project.save(None).await.unwrap()).unwrap();

        let mut first_work_package = WorkPackage::new(
            first_project_id.clone(),
            "First work package".into(),
            String::new(),
        );
        let first_work_package_id =
            ObjectId::from_str(&first_work_package.save(None).await.unwrap()).unwrap();

        let mut second_work_package = WorkPackage::new(
            second_project_id.clone(),
            "Second work package".into(),
            String::new(),
        );
        let second_work_package_id =
            ObjectId::from_str(&second_work_package.save(None).await.unwrap()).unwrap();

        let mut activity =
            ProjectActivity::new("Activity".into(), "description".into(), first_company_id);
        let activity_id = ObjectId::from_str(&activity.save(None).await.unwrap()).unwrap();

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
                    company_id: first_company_id,
                    project_id: first_project_id,
                    work_package_id: first_work_package_id,
                    activity_id: activity_id,
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: first_company_id,
                    project_id: second_project_id,
                    work_package_id: second_work_package_id,
                    activity_id: activity_id,
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
                    company_id: second_company_id,
                    project_id: second_project_id,
                    work_package_id: first_work_package_id,
                    activity_id: activity_id,
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: first_company_id,
                    project_id: first_project_id,
                    work_package_id: first_work_package_id,
                    activity_id: activity_id,
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
                    company_id: first_company_id,
                    project_id: first_project_id,
                    work_package_id: first_work_package_id,
                    activity_id: activity_id,
                    notes: "this is my description".into(),
                    hours: 2,
                },
                TimesheetActivityHours {
                    company_id: second_company_id,
                    project_id: second_project_id,
                    work_package_id: first_work_package_id,
                    activity_id: activity_id,
                    notes: "this is my second description".into(),
                    hours: 4,
                },
            ],
        )
        .await;
        assert!(result.is_ok());

        let result = export_as_excel(&user_id, &2025, &1).await;
        assert!(result.is_ok());

        let drop_result = get_database_service().await.db.drop().await;
        assert!(drop_result.is_ok());
    }
}
