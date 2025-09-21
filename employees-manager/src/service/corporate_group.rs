use bson::doc;

use crate::{
    enums::{CompanyRole, CorporateGroupRole},
    error::ServiceAppError,
    model::db_entities,
    service::db::document::SmartDocumentReference,
    DocumentId,
};

use super::db::document::DatabaseDocument;

pub struct CorporateGroupService {
    corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
}

impl CorporateGroupService {
    pub fn new(
        corporate_group_id: SmartDocumentReference<db_entities::CorporateGroup>,
    ) -> CorporateGroupService {
        CorporateGroupService { corporate_group_id }
    }

    /// Creates a new corporate group verifying that its name is unique
    ///
    /// This function can be invoked either by a user from the web app or by
    /// a platform admin from the admin panel.
    ///
    /// For the first case, user_id is Some and a role is created for the user
    /// so that he can operate on the corporate group.
    ///
    /// For the second case, the admin will assign the admin role for the created
    /// corporate group to a user
    pub async fn create_corporate_group(
        user_id: Option<SmartDocumentReference<db_entities::User>>,
        name: String,
    ) -> Result<(), ServiceAppError> {
        // check if a corporate group with the same name already exists
        let corporate_groups =
            db_entities::CorporateGroup::count_documents(doc! {"name": &name}).await?;
        if corporate_groups != 0 {
            Err(ServiceAppError::InvalidRequest(format!(
                "Corporate Group with name {name} already exists."
            )))
        } else {
            let mut new_doc = db_entities::CorporateGroup::new(name, true, Vec::new());
            new_doc.save().await?;
            if let Some(user_id) = user_id {
                let mut doc = db_entities::UserCorporateGroupRole::new(
                    user_id.to_id(),
                    *new_doc
                        .get_id()
                        .expect("Doc Id must be present after creating the document"),
                    CorporateGroupRole::Owner,
                );
                doc.save().await?;
            }
            Ok(())
        }
    }

    /// Deletes corporate group and all its related content
    ///
    /// This is a destructive operation that remove from database any entity
    /// which is related to the corporate group
    pub async fn delete(&self) -> Result<(), ServiceAppError> {
        let corporate_group = self.corporate_group_id.clone().to_document().await?;
        let corporate_group_id = self.corporate_group_id.as_ref_id();

        // We delete any user assignment to the corporate group
        db_entities::UserCorporateGroupRole::delete_many(
            doc! {"corporate_group_id": corporate_group_id},
        )
        .await?;

        // We delete any company in the corporate group
        for company_id in corporate_group.company_ids() {
            //company::delete_company(SmartDocumentReference::Id(*company_id)).await?;
            todo!()
        }

        // We delete any project activity in the corporate group
        db_entities::ProjectActivity::delete_many(doc! {"corporate_group_id": corporate_group_id})
            .await?;

        Ok(())
    }

    /// Edit corporate group by changing the name or the company list
    ///
    /// It returns ServiceAppError::InvalidRequest:
    ///     - if a corporate group with the same name already exists
    ///     - if a Company already belongs to another group
    ///     - if company vector is empty
    pub async fn update(
        &self,
        name: Option<String>,
        company_ids: Option<Vec<SmartDocumentReference<db_entities::Company>>>,
    ) -> Result<(), ServiceAppError> {
        let group_id = self.corporate_group_id.as_ref_id();
        // Update name
        let mut update_query = doc! {};
        if let Some(name) = name {
            if db_entities::CorporateGroup::count_documents(
                doc! { "name": &name, "_id": {"$ne": group_id}},
            )
            .await?
                > 0
            {
                return Err(ServiceAppError::InvalidRequest(format!(
                    "Corporate Group with name {name} already exist."
                )));
            } else {
                update_query.insert("name", name);
            }
        }

        if let Some(company_ids) = company_ids {
            if company_ids.len() == 0 {
                return Err(ServiceAppError::InvalidRequest(
                    "Company ids vector must contain values.".into(),
                ));
            } else {
                let company_ids: Vec<DocumentId> =
                    company_ids.into_iter().map(|elem| elem.to_id()).collect();
                update_query.insert("company_ids", company_ids);
            }
        }

        if update_query.is_empty() {
            Err(ServiceAppError::InvalidRequest(
                "At least corporate group name or company list must be specified".into(),
            ))
        } else {
            db_entities::CorporateGroup::update_one(
                doc! {"_id": group_id},
                doc! {"$set": update_query},
            )
            .await
        }
    }

    /// If the user is not already in the corporate group it
    /// creates a new entry for the User Corporate Group role
    /// Then, for each company inside the corporate group it
    /// creates a User Company Role.
    ///
    /// The role cannot be owner but only Admin or User.
    ///
    /// Finally, it creates a new contract for the employee with one
    /// of the companies that belong to the corporate group
    pub async fn add_user(
        &self,
        user_id: SmartDocumentReference<db_entities::User>,
        role: CorporateGroupRole,
        company_id: Option<SmartDocumentReference<db_entities::Company>>,
        job_title: Option<String>,
    ) -> Result<(), ServiceAppError> {
        // If the role is Owner then we return directly Err
        if role == CorporateGroupRole::Owner {
            return Err(ServiceAppError::InvalidRequest(
                "Cannot assign role Owner to the user in the corporate group".into(),
            ));
        }

        // First we check if the user is already in the corporate group, if not we add it
        // We add a user to the corporate group via adding a role
        if db_entities::UserCorporateGroupRole::count_documents(
        doc! {"user_id": user_id.as_ref_id(), "corporate_group_id": self.corporate_group_id.as_ref_id()},
        )
        .await?
            > 0
        {
            Err(ServiceAppError::InvalidRequest(format!(
                "User with id {user_id} is already inside the corporate group with id {}",
                self.corporate_group_id.as_ref_id()
            )))
        } else {
            // now we check if the company belongs to the corporate group,
            // if not we return Err
            if company_id.is_some() && job_title.is_some() {
                let company_id = company_id.unwrap();
                let job_title = job_title.unwrap();

                let corporate_group_doc = self.corporate_group_id.clone().to_document().await?;
                if !corporate_group_doc
                    .company_ids()
                    .contains(company_id.as_ref_id())
                {
                    return Err(ServiceAppError::InvalidRequest(format!(
                        "Company with id {} is not in the corporate group with id {}",
                        company_id.as_ref_id(),
                        self.corporate_group_id.as_ref_id()
                    )));
                }

                let mut contract = db_entities::UserEmploymentContract::new(
                    *user_id.as_ref_id(),
                    *company_id.as_ref_id(),
                    job_title,
                );
                contract.save().await?;
            }

            let mut doc = db_entities::UserCorporateGroupRole::new(
                user_id.as_ref_id().clone(),
                self.corporate_group_id.as_ref_id().clone(),
                role,
            );
            doc.save().await?;

            for company_id in self.corporate_group_id.clone()
                .to_document()
                .await
                .expect("Expecting corporate group after access control step.")
                .company_ids()
            {
                // TODO: define the default role for users in the companies. It can be Viewer
                let mut doc = db_entities::UserCompanyRole::new(
                    user_id.as_ref_id().clone(),
                    company_id.clone(),
                    CompanyRole::User,
                );
                doc.save().await?;
            }

            Ok(())
        }
    }

    /// If the user is inside the corporate group, it removes him from it by
    /// deleting his associated the corporate group and company roles
    pub async fn remove_user(
        &self,
        user_id: SmartDocumentReference<db_entities::User>,
    ) -> Result<(), ServiceAppError> {
        if let Some(cg_role_doc) = db_entities::UserCorporateGroupRole::find_one(
        doc! {"user_id": user_id.as_ref_id(), "corporate_group_id": self.corporate_group_id.as_ref_id()},
        )
        .await?
        {
            cg_role_doc.delete().await?;

            let corporate_group = self.corporate_group_id.clone()
                .to_document()
                .await
                .expect("Expecting corporate group after access control step.");

            for company_id in corporate_group.company_ids() {
                if let Some(company_role_doc) = db_entities::UserCompanyRole::find_one(
                    doc! { "user_id": user_id.as_ref_id(), "company_id": company_id },
                )
                .await?
                {
                    company_role_doc.delete().await?;
                }
            }

            // if exists delete the employment contract
            if let Some(contract_doc) = db_entities::UserEmploymentContract::find_one(
                doc! {"user_id": user_id.as_ref_id(), "company_id": {"$in": corporate_group.company_ids()}},
            )
            .await?
            {
                contract_doc.delete().await?;
            }


            Ok(())
        } else {
            Err(ServiceAppError::InvalidRequest(format!(
                "User with id {user_id} is already inside the corporate group with id {}",
                self.corporate_group_id.as_ref_id()
            )))
        }
    }

    /// We update only fields that are Some
    ///
    /// If role is some than we update UserCorporateGroupRole document
    /// If company_id and job_title are Some then we update the contract
    pub async fn update_user(
        &self,
        user_id: SmartDocumentReference<db_entities::User>,
        role: Option<CorporateGroupRole>,
        company_id: Option<SmartDocumentReference<db_entities::Company>>,
        job_title: Option<String>,
    ) -> Result<(), ServiceAppError> {
        if db_entities::UserCorporateGroupRole::count_documents(
        doc! {"user_id": user_id.as_ref_id(), "corporate_group_id": self.corporate_group_id.as_ref_id()},
        )
        .await?
            > 0
        {
            Err(ServiceAppError::InvalidRequest(format!(
                "User with id {user_id} is already inside the corporate group with id {}",
                self.corporate_group_id.as_ref_id()
            )))
        } else {
            // Update the corporate group role if present
            if let Some(role) = role {
                db_entities::UserCorporateGroupRole::update_one(
                    doc! { "user_id": user_id.as_ref_id(), "corporate_group_id": self.corporate_group_id.as_ref_id() },
                    doc! { "role": role },
                ).await?;
            }

            // if company id and job title are present, update company contract
            if company_id.is_some() && job_title.is_some() {
                let company_id = company_id.unwrap();
                let job_title = job_title.unwrap();

                // now we check if the company belongs to the corporate group,
                // if not we return Err
                let corporate_group_doc = self.corporate_group_id.clone().to_document().await?;
                if !corporate_group_doc
                    .company_ids()
                    .contains(company_id.as_ref_id())
                {
                    return Err(ServiceAppError::InvalidRequest(format!(
                        "Company with id {} is not in the corporate group with id {}",
                        company_id.as_ref_id(),
                        self.corporate_group_id.as_ref_id()
                    )));
                }

                db_entities::UserEmploymentContract::update_one(
                    doc! { "user_id": user_id.as_ref_id(), "company_id": company_id.as_ref_id() },
                    doc! { "job_title": job_title, "company_id": company_id.to_id() },
                )
                .await?;
            }

            Ok(())
        }
    }

    // Activate the corporate group and all the companies in it
    pub async fn activate(&self) -> Result<(), ServiceAppError> {
        db_entities::CorporateGroup::update_one(
            doc! {"_id": self.corporate_group_id.as_ref_id()},
            doc! {"$set": {"active": true}},
        )
        .await?;

        let corporate_group = self.corporate_group_id.clone().to_document().await?;

        db_entities::Company::update_many(
            doc! {"_id": corporate_group.company_ids()},
            doc! { "$set": {"active": true}},
        )
        .await?;

        Ok(())
    }

    // Deactivate the corporate group and all the companies in it
    pub async fn deactivate(&self) -> Result<(), ServiceAppError> {
        db_entities::CorporateGroup::update_one(
            doc! {"_id": self.corporate_group_id.as_ref_id()},
            doc! {"$set": {"active": false}},
        )
        .await?;

        let corporate_group = self.corporate_group_id.clone().to_document().await?;

        db_entities::Company::update_many(
            doc! {"_id": corporate_group.company_ids()},
            doc! { "$set": {"active": false}},
        )
        .await?;

        Ok(())
    }
}
