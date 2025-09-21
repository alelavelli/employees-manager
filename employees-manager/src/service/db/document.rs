use std::{str::FromStr, sync::Arc};

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneOptions, FindOptions},
    IndexModel,
};

use crate::{
    error::{DatabaseError, ServiceAppError},
    service::db::get_database_service,
    DocumentId, TRANSACTION,
};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;

#[derive(Clone)]
pub enum SmartDocumentReference<T: DatabaseDocument> {
    Id(DocumentId),
    Document(T),
}

impl<T: DatabaseDocument> SmartDocumentReference<T> {
    /// Returns a reference of document id without consuming the object
    pub fn as_ref_id(&self) -> &DocumentId {
        match self {
            SmartDocumentReference::Id(doc_id) => doc_id,
            SmartDocumentReference::Document(document) => {
                document.get_id().expect("ObjectId must always be set.")
            }
        }
    }

    /// Transform the enum SmartDocumentReference into the actual Document
    ///
    /// If the variant is Id(doc_id) then a query is done over the database to read the document
    pub async fn to_document(self) -> Result<T, ServiceAppError> {
        match self {
            SmartDocumentReference::Id(doc_id) => {
                T::find_one(doc! { "_id": doc_id }).await.and_then(|opt| {
                    opt.ok_or(ServiceAppError::EntityDoesNotExist(format!(
                        "Entity with id {doc_id} does not exist for collection"
                    )))
                })
            }
            SmartDocumentReference::Document(document) => Ok(document),
        }
    }

    /// Transform the enum SmartDocumentReference into the id of the Document
    ///
    /// If the variant is Document(doc) then the get_id() method is used with expect
    /// because the assumption is that it is a valid document
    pub fn to_id(self) -> DocumentId {
        match self {
            SmartDocumentReference::Id(doc_id) => doc_id,
            SmartDocumentReference::Document(document) => {
                *document.get_id().expect("ObjectId must always be set.")
            }
        }
    }
}

impl<T: DatabaseDocument> Display for SmartDocumentReference<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SmartDocumentReference::Id(id) => id.to_string(),
                SmartDocumentReference::Document(doc) => doc
                    .get_id()
                    .expect("ObjectId must always be set")
                    .to_string(),
            }
        )
    }
}

impl<T: DatabaseDocument> From<DocumentId> for SmartDocumentReference<T> {
    fn from(value: DocumentId) -> Self {
        Self::Id(value)
    }
}

impl<T: DatabaseDocument> From<&DocumentId> for SmartDocumentReference<T> {
    fn from(value: &DocumentId) -> Self {
        Self::Id(*value)
    }
}

impl<T: DatabaseDocument> From<T> for SmartDocumentReference<T> {
    fn from(value: T) -> Self {
        Self::Document(value)
    }
}

/// Trait that defines the behavior for each collection in database.
///
/// Operations divide in methods and functions.
/// Methods are save and delete and refers only to the current instance.
///
/// Functions are general and operate outside the instance
pub trait DatabaseDocument: Sized + Send + Sync + Serialize + DeserializeOwned {
    fn get_id(&self) -> Option<&DocumentId>;
    fn set_id(&mut self, document_id: &str) -> Result<(), DatabaseError>;
    fn collection_name() -> &'static str;
    //fn get_reference_fields(&self) -> Vec<Box<dyn ReferenceFieldTrait>>;

    fn set_indexes(
        keys: Document,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> {
        async {
            let index = IndexModel::builder().keys(keys).build();

            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            collection.create_index(index).await?;
            Ok(())
        }
    }

    /// Reload the document from the database
    fn reload(&mut self) -> impl std::future::Future<Output = Result<(), ServiceAppError>> {
        async {
            if let Some(document_id) = self.get_id() {
                let query = doc! {"_id": document_id};
                let db_service = get_database_service().await?;
                let collection = db_service
                    .get_db()
                    .collection::<Self>(Self::collection_name());
                let result = collection.find_one(query).await?;
                match result {
                    Some(document) => {
                        *self = document;
                        Ok(())
                    }
                    None => Err(ServiceAppError::EntityDoesNotExist(format!(
                        "Document with id {} not found in collection {}",
                        document_id,
                        Self::collection_name()
                    ))),
                }
            } else {
                Err(ServiceAppError::EntityDoesNotExist(format!(
                "Something went wrong when reloading collection {} because there is not ObjectId",
                Self::collection_name()
            )))
            }
        }
    }

    /// Save the document on the database
    ///
    /// It insert the new document or update it if it already exists.
    /// If the transaction parameter is present then the operation is added to the transaction
    fn save(&mut self) -> impl std::future::Future<Output = Result<String, ServiceAppError>> + Send
    where
        Self: Sized + Serialize + Send + Clone,
    {
        async {
            // TODO: improve the save operation by computing the diff with the document in the database and call the update method
            let document_id = if let Some(document_id) = self.get_id() {
                let query = doc! {"_id": document_id};
                // the document already exists, hence we call replace_one method
                if let Ok(transaction_lock) =
                    TRANSACTION.try_with(|transaction| Arc::clone(transaction))
                {
                    let mut transaction = transaction_lock.try_write().unwrap();
                    transaction.replace_one(query, self).await?;
                    document_id.to_hex()
                } else {
                    let db_service = get_database_service().await?;
                    let collection = db_service
                        .get_db()
                        .collection::<Self>(Self::collection_name());
                    collection.replace_one(query, self.clone()).await?;
                    document_id.to_hex()
                }
            } else {
                // the document does not exist in the database, hence we call insert_one method
                if let Ok(transaction_lock) =
                    TRANSACTION.try_with(|transaction| Arc::clone(transaction))
                {
                    let mut transaction = transaction_lock.try_write().unwrap();
                    transaction.insert_one(self).await?
                } else {
                    let db_service = get_database_service().await?;
                    let collection = db_service
                        .get_db()
                        .collection::<Self>(Self::collection_name());
                    let outcome = collection.insert_one(&mut *self).await?;
                    outcome.inserted_id.as_object_id().unwrap().to_hex()
                }
            };
            if self.get_id().is_none() {
                self.set_id(&document_id)?;
            }
            Ok(document_id)
        }
    }

    /// Delete the current document from the database
    ///
    /// Returns Err if the document does not exist
    fn delete(&self) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send
    where
        Self: Sized + Serialize + Send,
    {
        async {
            if let Some(document_id) = self.get_id() {
                let query = doc! {"_id": document_id};
                if let Ok(transaction_lock) =
                    TRANSACTION.try_with(|transaction| Arc::clone(transaction))
                {
                    let mut transaction = transaction_lock.try_write().unwrap();
                    transaction.delete_one::<Self>(query).await
                } else {
                    let db_service = get_database_service().await?;
                    let collection = db_service
                        .get_db()
                        .collection::<Self>(Self::collection_name());
                    let result = collection.delete_one(query).await?;
                    if result.deleted_count >= 1 {
                        Ok(())
                    } else {
                        Err(ServiceAppError::DatabaseError(format!(
                        "Something went wrong when deleting document with id {} for collection {}",
                        document_id,
                        Self::collection_name()
                    )))
                    }
                }
            } else {
                Err(ServiceAppError::DatabaseError(format!(
                "Something went wrong when deleting collection {} because there is not ObjectId",
                Self::collection_name()
            )))
            }
        }
    }

    fn find_one(
        query: Document,
    ) -> impl std::future::Future<Output = Result<Option<Self>, ServiceAppError>> + Send {
        async {
            //let db_service = get_database_service().await?;
            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            let result = collection.find_one(query).await?;
            Ok(result)
        }
    }

    fn find_many(
        query: Document,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, ServiceAppError>> + Send {
        async {
            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            let result: Vec<Self> = collection.find(query).await?.try_collect().await?;
            Ok(result)
        }
    }

    fn count_documents(
        query: Document,
    ) -> impl std::future::Future<Output = Result<u64, ServiceAppError>> + Send {
        async {
            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            let result: u64 = collection.count_documents(query).await?;
            Ok(result)
        }
    }

    fn find_one_projection<P>(
        query: Document,
        projection: Document,
    ) -> impl std::future::Future<Output = Result<Option<P>, ServiceAppError>> + Send
    where
        P: Send + Sync + Serialize + DeserializeOwned,
    {
        async {
            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            let query_options = FindOneOptions::builder().projection(projection).build();
            let result: Option<P> = collection
                .clone_with_type::<P>()
                .find_one(query)
                .with_options(query_options)
                .await?;
            Ok(result)
        }
    }

    fn find_many_projection<P>(
        query: Document,
        projection: Document,
    ) -> impl std::future::Future<Output = Result<Vec<P>, ServiceAppError>> + Send
    where
        P: Send + Sync + Serialize + DeserializeOwned,
    {
        async {
            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            let query_options = FindOptions::builder().projection(projection).build();
            let result: Vec<P> = collection
                .clone_with_type::<P>()
                .find(query)
                .with_options(query_options)
                .await?
                .try_collect()
                .await?;
            Ok(result)
        }
    }

    fn update_one(
        query: Document,
        update: Document,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send {
        async {
            if let Ok(transaction_lock) =
                TRANSACTION.try_with(|transaction| Arc::clone(transaction))
            {
                let mut transaction = transaction_lock.try_write().unwrap();
                transaction.update_one::<Self>(query, update).await
            } else {
                let db_service = get_database_service().await?;
                let collection = db_service
                    .get_db()
                    .collection::<Self>(Self::collection_name());
                collection.update_one(query.clone(), update).await?;
                Ok(())
            }
        }
    }

    fn update_many(
        query: Document,
        update: Document,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send {
        async {
            if let Ok(transaction_lock) =
                TRANSACTION.try_with(|transaction| Arc::clone(transaction))
            {
                let mut transaction = transaction_lock.try_write().unwrap();
                transaction.update_many::<Self>(query, update).await
            } else {
                let db_service = get_database_service().await?;
                let collection = db_service
                    .get_db()
                    .collection::<Self>(Self::collection_name());
                collection.update_many(query.clone(), update).await?;
                Ok(())
            }
        }
    }

    fn delete_many(
        query: Document,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send {
        async {
            if let Ok(transaction_lock) =
                TRANSACTION.try_with(|transaction| Arc::clone(transaction))
            {
                let mut transaction = transaction_lock.try_write().unwrap();
                transaction.delete_many::<Self>(query).await?;
                Ok(())
            } else {
                let db_service = get_database_service().await?;
                let collection = db_service
                    .get_db()
                    .collection::<Self>(Self::collection_name());
                let result = collection.delete_many(query.clone()).await?;
                if result.deleted_count >= 1 {
                    Ok(())
                } else {
                    Err(ServiceAppError::DatabaseError(format!(
                    "Something went wrong when deleting documents with query {} for collection {}",
                    query,
                    Self::collection_name()
                )))
                }
            }
        }
    }

    fn aggregate(
        pipeline: Vec<Document>,
    ) -> impl std::future::Future<Output = Result<Vec<Document>, ServiceAppError>> + Send
where {
        async {
            let db_service = get_database_service().await?;
            let collection = db_service
                .get_db()
                .collection::<Self>(Self::collection_name());
            let result = collection.aggregate(pipeline).await?.try_collect().await?;
            Ok(result)
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
