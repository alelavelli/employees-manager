use std::{str::FromStr, sync::Arc};

use axum::async_trait;
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Database,
};
use uuid::Uuid;

use crate::{error::AppError, service::environment::ENVIRONMENT, DocumentId};

use anyhow::anyhow;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use tokio::sync::OnceCell;

/*
differently from other global variables, database initialization requires async futures
therefore, we use tokio OnceCell and an async coroutine to initialize or get it lazily and
only once.

Database service is a single struct instance shared among all threads ensuring single
connection to the database.
However, tests needs temporary databases that are filled
with partial information. Since tests need to be independent we create a different
database for each test. In order to bypass the single instance we use thread_local
macro that allows us to create a different instance for each thread.
The method get_database_service will select the right database according to
the thread creating a new database and database connection.
 */
static DATABASE: OnceCell<Arc<DatabaseService>> = OnceCell::const_new();
thread_local! {
    static THREAD_LOCAL_DB: OnceCell<Arc<DatabaseService>> = const {OnceCell::const_new()};
}

pub async fn get_database_service() -> Arc<DatabaseService> {
    if cfg!(test) {
        let db_service = DatabaseService::new().await.unwrap();
        THREAD_LOCAL_DB
            .with(|f| {
                if !f.initialized() {
                    f.set(Arc::new(db_service)).unwrap();
                }
                f.clone()
            })
            .get()
            .unwrap()
            .clone()
    } else {
        DATABASE
            .get_or_init(|| async {
                Arc::new(
                    DatabaseService::new()
                        .await
                        .expect("Error in database initialization"),
                )
            })
            .await
            .clone()
    }
}

/// Database service struct that contain access to the database
#[derive(Debug)]
pub struct DatabaseService {
    pub db: Database,
}

impl DatabaseService {
    async fn new() -> Result<DatabaseService, AppError> {
        if cfg!(test) {
            let id = Uuid::new_v4().to_string();
            let mut db_name = String::from("app-test-db-");
            db_name.push_str(&id);
            let connection_string = format!("mongodb://localhost:27017/{}", db_name);
            let client_options = ClientOptions::parse(connection_string).await?;
            let client = Client::with_options(client_options)?;

            let db = client.database(&db_name);
            Ok(DatabaseService { db })
        } else {
            let client_options =
                ClientOptions::parse(&ENVIRONMENT.database.connection_string).await?;
            let client = Client::with_options(client_options)?;

            let db = client.database(&ENVIRONMENT.database.db_name);
            Ok(DatabaseService { db })
        }
    }

    /// Delete the document from the database
    pub async fn delete_document<T: DatabaseDocument + Send + Sync>(
        &self,
        document_id: &DocumentId,
    ) -> Result<(), AppError> {
        let collection = self.db.collection::<T>(T::collection_name());
        let filter = doc! { "_id": document_id};
        let result = collection.delete_one(filter).await?;
        if result.deleted_count >= 1 {
            Ok(())
        } else {
            Err(AppError::InternalServerError(anyhow!(format!(
                "Something went wrong when deleting document with id {} for collection {}",
                document_id,
                T::collection_name()
            ))))
        }
    }

    /// Update the document from the database
    pub async fn update_document<T: DatabaseDocument + Send + Sync>(
        &self,
        document_id: &DocumentId,
        update: Document,
    ) -> Result<(), AppError> {
        let collection = self.db.collection::<T>(T::collection_name());
        let query = doc! { "_id": document_id};
        let result = collection.update_one(query, update).await?;
        if result.matched_count == result.modified_count {
            Ok(())
        } else {
            Err(AppError::InternalServerError(anyhow!(format!(
                "Something went wrong when updating document with id {} for collection {}",
                document_id,
                T::collection_name()
            ))))
        }
    }
}

#[async_trait]
pub trait DatabaseDocument {
    fn get_id(&self) -> Result<&DocumentId, AppError>;
    fn collection_name() -> &'static str;

    async fn save(&self) -> Result<String, AppError>
    where
        Self: Sized + Serialize + Send,
    {
        let db = &get_database_service().await.db;
        let collection = db.collection::<Self>(Self::collection_name());
        let outcome = collection.insert_one(self).await?;
        let id = outcome.inserted_id.as_object_id().unwrap().to_hex();
        Ok(id)
    }

    async fn delete(&self) -> Result<(), AppError>
    where
        Self: Sized + Serialize + Send,
    {
        let db_service = get_database_service().await;
        if let Ok(document_id) = self.get_id() {
            db_service.delete_document::<Self>(document_id).await
        } else {
            Err(AppError::InternalServerError(anyhow!(format!(
                "Something went wrong when deleting collection {} because there is not ObjectId",
                Self::collection_name()
            ))))
        }
    }

    async fn update(&self, update: Document) -> Result<(), AppError>
    where
        Self: Sized + Serialize + Send,
    {
        let db_service = get_database_service().await;
        if let Ok(document_id) = self.get_id() {
            db_service
                .update_document::<Self>(document_id, update)
                .await
        } else {
            Err(AppError::InternalServerError(anyhow!(format!(
                "Something went wrong when deleting collection {} because there is not ObjectId",
                Self::collection_name()
            ))))
        }
    }
}

pub fn serialize_opt_object_id<S>(
    object_id: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match object_id {
        Some(ref object_id) => serialize_object_id_as_hex_string(object_id, serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_object_id_from_string<'de, D>(deserializer: D) -> Result<ObjectId, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    ObjectId::from_str(&buf).map_err(serde::de::Error::custom)
}
