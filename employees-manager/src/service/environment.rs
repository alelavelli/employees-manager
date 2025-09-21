//! Environment service use to build and store all the application environment variables.
//!
//! This struct loads the application variables from the environment or other secret manager endpoints
//! providing them to other services.
//! It represents the true and unique source of application variables

use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;

use crate::{
    enums::{FrontendMode, ObjectSourceType},
    EnvironmentServiceTrait,
};

/// ENVIRONMENT struct containing application variables
pub static ENVIRONMENT: Lazy<EnvironmentVariables> = Lazy::new(EnvironmentVariables::new);

/// Struct containing application environment variables that is initialized from
/// environment or accessing external services
///
/// Required environment variables:
///
/// - LOCAL: if the application is running in local debug running mode. For instance, from vscode debugger
/// - DEPLOY_ENVIRONMENT: in which context the application is running, it is used to specify some resources according to it
///
/// Required variables when LOCAL is False:
/// - JWT_SECRET: string used as secret to sign jwt
/// - MONGODB_CONNECTION_STRING: authenticated connection string to the mongodb cluster
/// - MONGODB_DB_NAME: name of mongodb database to use as prefix by adding deploy environment
/// - OBJECT_STORAGE_BACKEND: which type of backend to use as object storage
/// - OBJECT_STORAGE_PREFIX_PATH: prefix path to store objects. In case of remote object storage it contains also the bucket name
#[derive(Clone)]
pub struct EnvironmentVariables {
    logging: LoggingVariables,
    authentication: AuthenticationVariables,
    database: DatabaseVariables,
    storage: ObjectStorageVariables,
    frontend: FrontendVariables,
}

impl EnvironmentServiceTrait for EnvironmentVariables {
    fn get_database_connection_string(&self) -> &str {
        &self.database.connection_string
    }

    fn get_database_db_name(&self) -> &str {
        &self.database.db_name
    }

    fn get_authentication_jwt_expiration(&self) -> usize {
        self.authentication.jwt_expiration
    }

    fn get_authentication_jwt_encoding(&self) -> &EncodingKey {
        &self.authentication.jwt_encoding
    }

    fn get_authentication_jwt_decoding(&self) -> &DecodingKey {
        &self.authentication.jwt_decoding
    }

    fn get_logging_include_headers(&self) -> bool {
        self.logging.include_headers
    }

    fn get_logging_level(&self) -> tracing::Level {
        self.logging.level
    }

    fn get_object_storage_source_type(&self) -> &ObjectSourceType {
        &self.storage.storage_backend
    }

    fn get_object_storage_prefix_path(&self) -> &str {
        &self.storage.prefix_path
    }

    fn get_frontend_mode(&self) -> &FrontendMode {
        &self.frontend.frontend_mode
    }
}

impl EnvironmentVariables {
    /// Create new instance of this struct by invoking the different builds functions
    pub fn new() -> Self {
        // during testing use hardcoded custom env variables

        let local = if cfg!(test) {
            true
        } else {
            std::env::var("LOCAL")
                .map(|value| value.to_lowercase().cmp(&"true".to_string()).is_eq())
                .unwrap_or(false)
        };
        let deploy_environment = if cfg!(test) {
            "test".to_string()
        } else {
            std::env::var("DEPLOY_ENVIRONMENT").expect("DEPLOY_ENVIRONMENT must be set")
        };
        EnvironmentVariables {
            logging: Self::build_logging(&local, &deploy_environment),
            authentication: Self::build_authentication(&local, &deploy_environment),
            database: Self::build_database(&local, &deploy_environment),
            storage: Self::build_storage(&local, &deploy_environment),
            frontend: Self::build_frontend(&local, &deploy_environment),
        }
    }

    /// Build logging variables
    ///
    /// they are used by tracing to define correct logging properties
    fn build_logging(local: &bool, _deploy_environment: &str) -> LoggingVariables {
        let (level, include_headers) = if *local {
            (tracing::Level::TRACE, false)
        } else {
            (tracing::Level::INFO, false)
        };
        LoggingVariables {
            level,
            include_headers,
        }
    }

    /// Build authentication variables
    ///
    /// Environment variable `JWT_SECRET` is used to create JWT encoding and decoding keys
    /// therefore, it is mandatory.
    fn build_authentication(local: &bool, _deploy_environment: &str) -> AuthenticationVariables {
        let secret = if *local {
            "secret".to_string()
        } else {
            std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
        };
        AuthenticationVariables {
            jwt_expiration: 60 * 60 * 24,
            jwt_encoding: EncodingKey::from_secret(secret.as_bytes()),
            jwt_decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Build database variables
    fn build_database(local: &bool, deploy_environment: &str) -> DatabaseVariables {
        let (connection_string, db_name) = if *local {
            let db_name = format!("application-database-{}", deploy_environment);
            (
                format!(
                    "mongodb://localhost:27117/{}?replicaSet=rs0&directConnection=true",
                    db_name
                ),
                db_name,
            )
        } else {
            (
                std::env::var("MONGODB_CONNECTION_STRING")
                    .expect("MONGODB_CONNECTION_STRING must be set"),
                std::env::var("MONGODB_DB_NAME").expect("MONGODB_DB_NAME must be set"),
            )
        };

        DatabaseVariables {
            connection_string,
            db_name,
        }
    }

    /// Storage variables determine where data objects are stored. According to an
    /// environment variable the final location can be different. In local testing
    /// environment it is the local file system
    fn build_storage(local: &bool, _deploy_environment: &str) -> ObjectStorageVariables {
        if *local {
            ObjectStorageVariables {
                storage_backend: ObjectSourceType::LocalFileSystem,
                prefix_path: "../app-objects".into(),
            }
        } else {
            let storage_backend_var = std::env::var("OBJECT_STORAGE_BACKEND")
                .expect("OBJECT_STORAGE_BACKEND must be set.");
            ObjectStorageVariables {
                storage_backend: ObjectSourceType::try_from(storage_backend_var.as_str())
                    .unwrap_or_else(|_| {
                        panic!(
                            "Invalid OBJECT_STORAGE_BACKEND value. Got {}",
                            storage_backend_var
                        )
                    }),
                prefix_path: std::env::var("OBJECT_STORAGE_PREFIX_PATH")
                    .expect("OBJECT_STORAGE_PREFIX_PATH must be set"),
            }
        }
    }

    fn build_frontend(local: &bool, _deploy_environment: &str) -> FrontendVariables {
        if *local {
            FrontendVariables {
                frontend_mode: FrontendMode::External,
            }
        } else {
            let frontend_mode = std::env::var("FRONTEND_MODE").expect("FRONTEND_MODE must be set.");
            FrontendVariables {
                frontend_mode: FrontendMode::try_from(frontend_mode.as_str()).unwrap_or_else(
                    |_| panic!("Invalid FRONTEND_MODE value. Got `{frontend_mode}`"),
                ),
            }
        }
    }
}

/// Struct containing logging variables like logging level
#[derive(Debug, Clone)]
struct LoggingVariables {
    /// application logging level
    level: tracing::Level,
    /// if true, we include headers in every log coming from a http request
    include_headers: bool,
}

/// Struct containing variables for authentication
///
/// It contains two keys used to encode and decode jwt tokens for web application
#[derive(Clone)]
struct AuthenticationVariables {
    jwt_expiration: usize,
    jwt_encoding: EncodingKey,
    jwt_decoding: DecodingKey,
}

/// Struct containing variables for data base like connection string
#[derive(Debug, Clone)]
struct DatabaseVariables {
    connection_string: String,
    db_name: String,
}

/// Variables with information to store objects of the application
#[derive(Debug, Clone)]
struct ObjectStorageVariables {
    storage_backend: ObjectSourceType,
    prefix_path: String,
}

/// Frontend configuration
///
/// Frontend can be served as static content from the web server or
/// as an external entity. These variables defines which type of
/// configuration is used.
///
/// FrontendMode::Internal contains a string variable that represents
/// the path of the root folder containing static files
#[derive(Debug, Clone)]
struct FrontendVariables {
    frontend_mode: FrontendMode,
}
