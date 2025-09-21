use std::borrow::Borrow;

use mongodb::{bson::Document, ClientSession};

use crate::{
    error::{DatabaseError, ServiceAppError},
    service::db::{document::DatabaseDocument, get_database_service},
};

use serde::Serialize;

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
    pub fn new(session: ClientSession) -> DatabaseTransaction {
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            let query_result = collection
                .insert_one(document)
                .session(&mut self.session)
                .await;
            if let Ok(outcome) = query_result {
                let id = outcome.inserted_id.as_object_id().unwrap().to_hex();
                Ok(id)
            } else {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(query_result.err().unwrap().to_string()).into())
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            if let Err(err) = collection
                .insert_many(documents)
                .session(&mut self.session)
                .await
            {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(err.to_string()).into())
            } else {
                Ok(())
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            if let Err(err) = collection
                .update_one(query, update)
                .session(&mut self.session)
                .await
            {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(err.to_string()).into())
            } else {
                Ok(())
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            if let Err(err) = collection
                .update_many(query, update)
                .session(&mut self.session)
                .await
            {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(err.to_string()).into())
            } else {
                Ok(())
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            if let Err(err) = collection
                .replace_one(query, replacement)
                .session(&mut self.session)
                .await
            {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(err.to_string()).into())
            } else {
                Ok(())
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            if let Err(err) = collection
                .delete_one(filter)
                .session(&mut self.session)
                .await
            {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(err.to_string()).into())
            } else {
                Ok(())
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
                .database(get_database_service().await?.get_db().name());
            let collection = db.collection::<T>(T::collection_name());
            if let Err(err) = collection
                .delete_many(filter)
                .session(&mut self.session)
                .await
            {
                self.abort_transaction().await?;
                Err(DatabaseError::TransactionError(err.to_string()).into())
            } else {
                Ok(())
            }
        } else {
            Err(DatabaseError::TransactionNotStarted.into())
        }
    }
}
