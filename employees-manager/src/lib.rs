use std::sync::Arc;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use mongodb::{bson::oid::ObjectId, Client};
use tokio::{sync::RwLock, task_local};

use crate::{
    enums::{FrontendMode, ObjectSourceType},
    error::AppError,
    service::db::transaction::DatabaseTransaction,
};
use mongodb::Database;

mod auth;
mod dtos;
pub mod enums;
mod error;
mod facade;
pub mod middleware;
pub mod model;
pub mod router;
pub mod service;

type DocumentId = ObjectId;

/// Trait to define the environment service behavior
pub trait EnvironmentServiceTrait: Send + Sync {
    fn get_database_connection_string(&self) -> &str;

    fn get_database_db_name(&self) -> &str;

    fn get_authentication_jwt_expiration(&self) -> usize;

    fn get_authentication_jwt_encoding(&self) -> &EncodingKey;

    fn get_authentication_jwt_decoding(&self) -> &DecodingKey;

    fn get_logging_include_headers(&self) -> bool;

    fn get_logging_level(&self) -> tracing::Level;

    fn get_object_storage_source_type(&self) -> &ObjectSourceType;

    fn get_object_storage_prefix_path(&self) -> &str;

    fn get_frontend_mode(&self) -> &FrontendMode;
}

/// Trait to define the database service behavior
pub trait DatabaseServiceTrait: Send + Sync {
    fn get_db(&self) -> &Database;

    fn get_client(&self) -> &Client;
}

/// Struct containing application global variables used
/// as state by any request in the application.
///
/// According to documentation https://github.com/tokio-rs/axum/blob/main/examples/dependency-injection/src/main.rs
/// it is recommended to use dyn trait to leverage on dependency injection
#[derive(Clone)]
pub struct AppState {
    pub environment_service: Arc<dyn EnvironmentServiceTrait>,
    pub database_service: Arc<dyn DatabaseServiceTrait>,
}

impl AppState {
    pub fn new(
        environment_service: Arc<dyn EnvironmentServiceTrait>,
        database_service: Arc<dyn DatabaseServiceTrait>,
    ) -> AppState {
        AppState {
            environment_service,
            database_service,
        }
    }
}

impl<S> FromRequestParts<S> for AppState
where
    Self: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

task_local! {
    static APP_STATE: AppState;
    static TRANSACTION: Arc<RwLock<DatabaseTransaction>>;
}
