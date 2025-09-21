use std::{borrow::Borrow, str::FromStr, sync::Arc};

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOneOptions, FindOptions},
    Client, ClientSession, Database, IndexModel,
};

use tracing::debug;

use crate::{
    error::{DatabaseError, ServiceAppError},
    service::environment::ENVIRONMENT,
    DocumentId,
};

use mongodb::bson::oid::ObjectId;
use mongodb::bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
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
/* thread_local! {
    static THREAD_LOCAL_DB: OnceCell<Arc<DatabaseService>> = OnceCell::const_new();
} */

pub async fn get_database_service() -> Arc<DatabaseService> {
    if cfg!(test) {
        // Form some reason thead_local stop working due to `there is no reactor running, must be called from the context of Tokio runtime`
        // therefore, the database service always returns a connection using the current thread id.
        // The drawback is the creation of multiple connections
        let db_service = DatabaseService::new().await.unwrap();
        Arc::new(db_service)
        /* THREAD_LOCAL_DB
        .with(|f| {
            if !f.initialized() {
                f.set(Arc::new(db_service)).unwrap();
            }
            f.clone()
        })
        .get()
        .unwrap()
        .clone() */
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

/// Database service struct
///
/// It connects to the database and creates session objects to perform transactions
#[derive(Debug)]
pub struct DatabaseService {
    client: Client,
    pub db: Database,
}

impl DatabaseService {
    async fn new() -> Result<DatabaseService, ServiceAppError> {
        if cfg!(test) {
            let id = format!("{:?}", std::thread::current().id());
            let mut db_name = String::from("app-test-db-");
            db_name.push_str(&id);
            let connection_string = format!(
                "mongodb://localhost:27117/{}?replicaSet=rs0&directConnection=true",
                db_name
            );
            let client_options = ClientOptions::parse(connection_string).await?;
            let client = Client::with_options(client_options)?;

            let db = client.database(&db_name);
            Ok(DatabaseService { client, db })
        } else {
            debug!(
                "Connecting to database with connection string: {}",
                ENVIRONMENT.get_database_connection_string()
            );
            let client_options =
                ClientOptions::parse(ENVIRONMENT.get_database_connection_string()).await?;
            let client = Client::with_options(client_options)?;

            let db = client.database(ENVIRONMENT.get_database_db_name());
            Ok(DatabaseService { client, db })
        }
    }

    /// Create new session from the client to start a new transaction and returns DatabaseTransaction instance
    pub async fn new_transaction(&self) -> Result<DatabaseTransaction, ServiceAppError> {
        Ok(DatabaseTransaction::new(self.client.start_session().await?))
    }
}

/// Wraps database operations inside the transaction allowing to commit or abort everything.
///
/// When the object is created the transaction is not started yet and any operation will fail if
/// the user service does not start the transaction
pub struct DatabaseTransaction {
    session: ClientSession,
    transaction_started: bool,
    transaction_closed: bool,
}

impl DatabaseTransaction {
    fn new(session: ClientSession) -> DatabaseTransaction {
        DatabaseTransaction {
            session,
            transaction_started: false,
            transaction_closed: false,
        }
    }

    pub async fn start_transaction(&mut self) -> Result<(), ServiceAppError> {
        self.session.start_transaction().await?;
        self.transaction_started = true;
        Ok(())
    }

    pub async fn abort_transaction(&mut self) -> Result<(), ServiceAppError> {
        if self.transaction_started {
            self.session.abort_transaction().await?;
            self.transaction_closed = true;
        }
        Ok(())
    }

    pub async fn commit_transaction(&mut self) -> Result<(), ServiceAppError> {
        if self.transaction_started {
            self.session.commit_transaction().await?;
            self.transaction_closed = true;
        }
        Ok(())
    }

    pub async fn insert_one<T>(&mut self, document: &T) -> Result<String, ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if let Ok(outcome) = collection
                .insert_one(document)
                .session(&mut self.session)
                .await
            {
                let id = outcome.inserted_id.as_object_id().unwrap().to_hex();
                Ok(id)
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }

    pub async fn insert_many<T>(&mut self, documents: Vec<&T>) -> Result<(), ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if collection
                .insert_many(documents)
                .session(&mut self.session)
                .await
                .is_ok()
            {
                Ok(())
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }

    pub async fn update_one<T>(
        &mut self,
        query: Document,
        update: Document,
    ) -> Result<(), ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if collection
                .update_one(query, update)
                .session(&mut self.session)
                .await
                .is_ok()
            {
                Ok(())
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }

    pub async fn update_many<T>(
        &mut self,
        query: Document,
        update: Document,
    ) -> Result<(), ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if collection
                .update_many(query, update)
                .session(&mut self.session)
                .await
                .is_ok()
            {
                Ok(())
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }

    pub async fn replace_one<T>(
        &mut self,
        query: Document,
        replacement: &T,
    ) -> Result<(), ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize + Borrow<T>,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if collection
                .replace_one(query, replacement)
                .session(&mut self.session)
                .await
                .is_ok()
            {
                Ok(())
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }

    pub async fn delete_one<T>(&mut self, filter: Document) -> Result<(), ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if collection
                .delete_one(filter)
                .session(&mut self.session)
                .await
                .is_ok()
            {
                Ok(())
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }

    pub async fn delete_many<T>(&mut self, filter: Document) -> Result<(), ServiceAppError>
    where
        T: DatabaseDocument + Send + Sync + Serialize,
    {
        if self.transaction_closed {
            Err(DatabaseError::TransactionClosed.into())
        } else if self.transaction_started {
            let db = self
                .session
                .client()
                .database(get_database_service().await.db.name());
            let collection = db.collection::<T>(T::collection_name());
            if collection
                .delete_many(filter)
                .session(&mut self.session)
                .await
                .is_ok()
            {
                Ok(())
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError.into())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
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

    fn set_indexes(
        keys: Document,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> {
        async {
            let index = IndexModel::builder().keys(keys).build();

            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
            collection.create_index(index).await?;
            Ok(())
        }
    }

    /// Reload the document from the database
    fn reload(&mut self) -> impl std::future::Future<Output = Result<(), ServiceAppError>> {
        async {
            if let Some(document_id) = self.get_id() {
                let query = doc! {"_id": document_id};
                let db_service = get_database_service().await;
                let collection = db_service.db.collection::<Self>(Self::collection_name());
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
    fn save(
        &mut self,
        transaction: Option<&mut DatabaseTransaction>,
    ) -> impl std::future::Future<Output = Result<String, ServiceAppError>> + Send
    where
        Self: Sized + Serialize + Send + Clone,
    {
        async {
            // TODO: improve the save operation by computing the diff with the document in the database and call the update method
            let document_id = if let Some(document_id) = self.get_id() {
                let query = doc! {"_id": document_id};
                // the document already exists, hence we call replace_one method
                if let Some(transaction) = transaction {
                    transaction.replace_one(query, self).await?;
                    document_id.to_hex()
                } else {
                    let db_service = get_database_service().await;
                    let collection = db_service.db.collection::<Self>(Self::collection_name());
                    collection.replace_one(query, self.clone()).await?;
                    document_id.to_hex()
                }
            } else {
                // the document does not exist in the database, hence we call insert_one method
                if let Some(transaction) = transaction {
                    transaction.insert_one(self).await?
                } else {
                    let db_service = get_database_service().await;
                    let collection = db_service.db.collection::<Self>(Self::collection_name());
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
    fn delete(
        &self,
        transaction: Option<&mut DatabaseTransaction>,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send
    where
        Self: Sized + Serialize + Send,
    {
        async {
            if let Some(document_id) = self.get_id() {
                let query = doc! {"_id": document_id};
                if let Some(transaction) = transaction {
                    transaction.delete_one::<Self>(query).await
                } else {
                    let db_service = get_database_service().await;
                    let collection = db_service.db.collection::<Self>(Self::collection_name());
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
            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
            let result = collection.find_one(query).await?;
            Ok(result)
        }
    }

    fn find_many(
        query: Document,
    ) -> impl std::future::Future<Output = Result<Vec<Self>, ServiceAppError>> + Send {
        async {
            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
            let result: Vec<Self> = collection.find(query).await?.try_collect().await?;
            Ok(result)
        }
    }

    fn count_documents(
        query: Document,
    ) -> impl std::future::Future<Output = Result<u64, ServiceAppError>> + Send {
        async {
            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
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
            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
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
            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
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
        transaction: Option<&mut DatabaseTransaction>,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send {
        async {
            if let Some(transaction) = transaction {
                transaction.update_one::<Self>(query, update).await
            } else {
                let db_service = get_database_service().await;
                let collection = db_service.db.collection::<Self>(Self::collection_name());
                collection.update_one(query.clone(), update).await?;
                Ok(())
            }
        }
    }

    fn update_many(
        query: Document,
        update: Document,
        transaction: Option<&mut DatabaseTransaction>,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send {
        async {
            if let Some(transaction) = transaction {
                transaction.update_many::<Self>(query, update).await
            } else {
                let db_service = get_database_service().await;
                let collection = db_service.db.collection::<Self>(Self::collection_name());
                collection.update_many(query.clone(), update).await?;
                Ok(())
            }
        }
    }

    fn delete_many(
        query: Document,
        transaction: Option<&mut DatabaseTransaction>,
    ) -> impl std::future::Future<Output = Result<(), ServiceAppError>> + Send {
        async {
            if let Some(transaction) = transaction {
                transaction.delete_many::<Self>(query).await
            } else {
                let db_service = get_database_service().await;
                let collection = db_service.db.collection::<Self>(Self::collection_name());
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
            let db_service = get_database_service().await;
            let collection = db_service.db.collection::<Self>(Self::collection_name());
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
