use bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    enums::CorporateGroupRole,
    error::ServiceAppError,
    model::{
        db_entities,
        internal::{AdminPanelOverviewCompanyInfo, AdminPanelOverviewUserInfo, AdminPanelUserInfo},
    },
    service::db::document::{DatabaseDocument, SmartDocumentReference},
    DocumentId,
};

pub async fn get_admin_panel_overview_users_info(
) -> Result<AdminPanelOverviewUserInfo, ServiceAppError> {
    let result = db_entities::User::aggregate(vec![doc! {"$group": {
        "_id": null,
        "total_users": {"$sum": 1},
        "total_admins": {"$sum": {"$cond": [{"$eq": ["$platform_admin", true]}, 1, 0]}},
        "total_inactive_users": {"$sum": {"$cond": [{"$eq": ["$active", false]}, 1, 0]}},
        "total_active_users": {"$sum": {"$cond": [{"$eq": ["$active", true]}, 1, 0]}}
    }
    }])
    .await?;

    if let Some(result) = result.first() {
        Ok(AdminPanelOverviewUserInfo {
            total_users: result
                .get("total_users")
                .expect("total_users should be present")
                .as_i32()
                .unwrap() as u16,
            total_admins: result
                .get("total_admins")
                .expect("total_admins should be present")
                .as_i32()
                .unwrap() as u16,
            total_active_users: result
                .get("total_active_users")
                .expect("total_active_users should be present")
                .as_i32()
                .unwrap() as u16,
            total_inactive_users: result
                .get("total_inactive_users")
                .expect("total_inactive_users should be present")
                .as_i32()
                .unwrap() as u16,
        })
    } else {
        Ok(AdminPanelOverviewUserInfo::default())
    }
}

/// Returns the companies info for the admin panel
pub async fn get_admin_panel_overview_companies_info(
) -> Result<AdminPanelOverviewCompanyInfo, ServiceAppError> {
    let result = db_entities::Company::aggregate(vec![doc! {
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

pub async fn get_admin_panel_users_info() -> Result<Vec<AdminPanelUserInfo>, ServiceAppError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct QueryResult {
        _id: DocumentId,
        username: String,
        email: String,
        name: String,
        surname: String,
        platform_admin: bool,
        active: bool,
    }

    let users = db_entities::User::find_many_projection::<QueryResult>(
        doc! {},
        doc! {
            "_id": 1,
            "username": 1,
            "email": 1,
            "name": 1,
            "surname": 1,
            "platform_admin": 1,
            "active": 1
        },
    )
    .await?;

    let users = users
        .iter()
        .map(|user| AdminPanelUserInfo {
            id: user._id,
            username: user.username.clone(),
            email: user.email.clone(),
            name: user.name.clone(),
            surname: user.surname.clone(),
            platform_admin: user.platform_admin,
            active: user.active,
            total_companies: 0,
        })
        .collect::<Vec<AdminPanelUserInfo>>();

    Ok(users)
}

/// Returns the list of corporate groups in the application
pub async fn get_admin_corporate_groups_info(
) -> Result<Vec<db_entities::CorporateGroup>, ServiceAppError> {
    db_entities::CorporateGroup::find_many(doc! {}).await
}

/// Set the user as corporate group owner
///
/// Since this operation can happen at the creation of
/// the corporate group, is it possible that the user is
/// not inside the corporate group.
///
/// If not, then he is added and the role created.
/// If an owner is already present then he is changed to Admin
pub async fn set_corporate_group_owner(
    corporate_group_id: &SmartDocumentReference<db_entities::CorporateGroup>,
    user_id: &SmartDocumentReference<db_entities::User>,
) -> Result<(), ServiceAppError> {
    if db_entities::CorporateGroup::count_documents(doc! {"_id": corporate_group_id.as_ref_id()})
        .await?
        == 0
    {
        return Err(ServiceAppError::EntityDoesNotExist(format!(
            "Corporate group with id {} does not exist.",
            corporate_group_id.as_ref_id()
        )));
    }

    if db_entities::User::count_documents(doc! {"_id": user_id.as_ref_id()}).await? == 0 {
        return Err(ServiceAppError::EntityDoesNotExist(format!(
            "User with id {} does not exist.",
            user_id.as_ref_id()
        )));
    }

    // If the corporate group already has an owner then we change his role
    let previous_owner = db_entities::UserCorporateGroupRole::find_one(
        doc! {"corporate_group": corporate_group_id.as_ref_id(), "role": CorporateGroupRole::Owner},
    )
    .await?;

    if let Some(previous_owner) = previous_owner {
        db_entities::UserCorporateGroupRole::update_one(
            doc! {"corporate_group": corporate_group_id.as_ref_id(), "user_id": previous_owner.user_id()}, 
            doc! {"$set": {"role": CorporateGroupRole::Admin}}).await?;
    }

    // if the user is not present in the corporate group we add it
    if db_entities::UserCorporateGroupRole::find_one(
        doc! {"corporate_group": corporate_group_id.as_ref_id(), "user_id": user_id.as_ref_id()},
    )
    .await?
    .is_some()
    {
        db_entities::UserCorporateGroupRole::update_one(
            doc! {"corporate_group": corporate_group_id.as_ref_id(), "user_id": user_id.as_ref_id()}, 
            doc! {"$set": {"role": CorporateGroupRole::Owner}}).await?;
    } else {
        let mut new_role = db_entities::UserCorporateGroupRole::new(
            *user_id.as_ref_id(),
            *corporate_group_id.as_ref_id(),
            CorporateGroupRole::Owner,
        );
        new_role.save().await?;
    }

    Ok(())
}
