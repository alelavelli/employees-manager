//! Environment service use to build and store all the application environment variables.
//!
//! This struct loads the application variables from the environment or other secret manager endpoints
//! providing them to other services.
//! It represents the true and unique source of application variables

use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;

/// ENVIRONMENT struct containing application variables
pub static ENVIRONMENT: Lazy<EnvironmentVariables> = Lazy::new(EnvironmentVariables::new);

/// Struct containing application environment variables that is initialized from
/// environment or accessing external services
pub struct EnvironmentVariables {
    pub logging: LoggingVariables,
    pub authentication: AuthenticationVariables,
    pub database: DatabaseVariables,
}

impl EnvironmentVariables {
    /// Create new instance of this struct by invoking the different builds functions
    fn new() -> Self {
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
}

/// Struct containing logging variables like logging level
pub struct LoggingVariables {
    /// application logging level
    pub level: tracing::Level,
    /// if true, we include headers in every log coming from a http request
    pub include_headers: bool,
}

/// Struct containing variables for authentication
///
/// It contains two keys used to encode and decode jwt tokens for web application
pub struct AuthenticationVariables {
    pub jwt_encoding: EncodingKey,
    pub jwt_decoding: DecodingKey,
}

/// Struct containing variables for data base like connection string
pub struct DatabaseVariables {
    pub connection_string: String,
    pub db_name: String,
}
