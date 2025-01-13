use anyhow::anyhow;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

use crate::dtos::AppJson;

/// AppError enumeration of different error typologies that the application
/// can return to clients.
///
/// It implements the trait `IntoResponse` to translate the error into a response
/// composed of status and message.
///
/// Moreover, it implements several `From<T>` trait to automatically translate
/// internal errors to AppError using `?`
#[derive(Debug)]
pub enum AppError {
    /// The request body contained invalid JSON
    JsonRejection(JsonRejection),
    /// Internal error
    InternalServerError(anyhow::Error),
    /// Authorization error
    AuthorizationError(AuthError),
    /// Entity does not exist
    DoesNotExist(anyhow::Error),
    /// The user does not have role to perform the operation
    AccessControlError,
    /// ManagedError that is created when the web app returns an
    /// error to the client that is due to wrong parameters or
    /// something that cannot be done
    ManagedError(String),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        // Define StatusCode and message for every enum variant
        let (status, message) = match self {
            AppError::JsonRejection(rejection) => {
                // This error is caused by bad user input so don't log it
                (rejection.status(), rejection.body_text())
            }
            AppError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong".into(),
            ),
            AppError::AuthorizationError(auth_error) => auth_error.to_status_message(),
            AppError::DoesNotExist(_) => (StatusCode::NOT_FOUND, "Entity not found".into()),
            AppError::AccessControlError => (
                StatusCode::UNAUTHORIZED,
                "Not sufficient permissions".into(),
            ),
            AppError::ManagedError(message) => (StatusCode::BAD_REQUEST, message.clone()),
        };
        (status, AppJson(ErrorResponse { message })).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(value: JsonRejection) -> Self {
        Self::JsonRejection(value)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        Self::InternalServerError(value)
    }
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        Self::AuthorizationError(value)
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(value: mongodb::error::Error) -> Self {
        error!("MongoDB error: {:?}", value);
        Self::InternalServerError(anyhow::Error::new(value))
    }
}

impl From<DatabaseError> for AppError {
    fn from(value: DatabaseError) -> Self {
        error!("Database error: {:?}", value);
        match value {
            DatabaseError::TransactionNotStarted => {
                AppError::InternalServerError(anyhow!("Transaction not started"))
            }
            DatabaseError::TransactionClosed => {
                AppError::InternalServerError(anyhow!("Transaction already committed"))
            }
            DatabaseError::TransactionError => {
                AppError::InternalServerError(anyhow!("Transaction operation encountered an error"))
            }
            DatabaseError::DocumentDoesNotExist => {
                AppError::InternalServerError(anyhow!("Document not found"))
            }
            DatabaseError::DocumentHasAlreadyAnId => AppError::InternalServerError(anyhow!(
                "Document cannot be inserted in database because it has already an id"
            )),
            DatabaseError::InvalidObjectId => {
                AppError::InternalServerError(anyhow!("Invalid object id"))
            }
        }
    }
}

/// AuthError is an internal error used by authentication modules to explain why
/// authentication is failed.
/// They are translated to `AppError` when exposed to the client
#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InvalidApiKey,
}

impl AuthError {
    fn to_status_message(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            AuthError::WrongCredentials => {
                (StatusCode::UNAUTHORIZED, "Wrong credentials".to_string())
            }
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Wrong credentials".to_string()),

            AuthError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials".to_string())
            }
            AuthError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
        };
        (status, message)
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    /// When an operation over a transaction is executed but the transaction is not started yet
    TransactionNotStarted,
    /// When an operation over a transaction is executed but the transaction is already committed
    TransactionClosed,
    /// When an operation over a transaction fails
    TransactionError,
    DocumentDoesNotExist,
    DocumentHasAlreadyAnId,
    InvalidObjectId,
}
