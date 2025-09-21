use std::sync::Arc;

use mongodb::{options::ClientOptions, Client, Database};

use crate::{
    error::ServiceAppError,
    service::{db::transaction::DatabaseTransaction, environment::EnvironmentVariables},
    DatabaseServiceTrait, EnvironmentServiceTrait, APP_STATE,
};

use tokio::sync::OnceCell;

pub mod document;
pub mod reference;
pub mod transaction;

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

pub async fn get_database_service(
) -> Result<Arc<dyn DatabaseServiceTrait + 'static>, ServiceAppError> {
    if cfg!(test) {
        DatabaseService::new(None::<&EnvironmentVariables>)
            .await
            .map(|x| Arc::new(x) as Arc<dyn DatabaseServiceTrait + 'static>)
    } else {
        // We use try_with to handle result instead of panicking if the variable has not been set
        APP_STATE
            .try_with(|state| Arc::clone(&state.database_service))
            .map_err(|e| ServiceAppError::DatabaseError(e.to_string()))
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

impl DatabaseServiceTrait for DatabaseService {
    fn get_db(&self) -> &Database {
        &self.db
    }

    fn get_client(&self) -> &Client {
        &self.client
    }
}

impl DatabaseService {
    pub async fn new(
        environment_service: Option<&impl EnvironmentServiceTrait>,
    ) -> Result<DatabaseService, ServiceAppError> {
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
        } else if let Some(environment_service) = environment_service {
            let client_options =
                ClientOptions::parse(environment_service.get_database_connection_string()).await?;
            let client = Client::with_options(client_options)?;

            let db = client.database(environment_service.get_database_db_name());
            Ok(DatabaseService { client, db })
        } else {
            Err(ServiceAppError::DatabaseError(
                "Environment service must be specified during initialization.".into(),
            ))
        }
    }

    /// Create new session from the client to start a new transaction and returns DatabaseTransaction instance
    pub async fn new_transaction(&self) -> Result<DatabaseTransaction, ServiceAppError> {
        Ok(DatabaseTransaction::new(self.client.start_session().await?))
    }
}
