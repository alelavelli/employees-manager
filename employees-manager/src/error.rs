use std::fmt::Display;

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rust_xlsxwriter::XlsxError;
use serde::Serialize;
use tracing::error;

use crate::dtos::{web_app_response, AppJson};

/// AppError enumeration of different error typologies that the application
/// can return to clients.
/// This enumerator differs from the `ServiceAppError` because any error is
/// returned to the client, while `ServiceAppError` is translated to AppError
/// according to the context.
///
/// It implements the trait `IntoResponse` to translate the error into a response
/// composed of status and message.
///
#[derive(Debug)]
pub enum AppError {
    /// The request body contained invalid JSON aka 422
    JsonRejection(JsonRejection),
    /// Internal error aka 500
    InternalServerError(String),
    /// Authorization error aka 401
    AuthorizationError(AuthError),
    /// Entity does not exist aka 404
    DoesNotExist(String),
    /// The user does not have role to perform the operation aka 403
    AccessControlError(String),
    /// When the request is not valid due to one of its parameters aka 400
    InvalidRequest(String),
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
            AppError::InternalServerError(error_message) => {
                error!(error_message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".into(),
                )
            }
            AppError::AuthorizationError(auth_error) => auth_error.to_status_message(),
            AppError::DoesNotExist(message) => (StatusCode::NOT_FOUND, message),
            AppError::AccessControlError(message) => (StatusCode::FORBIDDEN, message),
            AppError::InvalidRequest(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, AppJson(ErrorResponse { message })).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(value: JsonRejection) -> Self {
        Self::JsonRejection(value)
    }
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        Self::AuthorizationError(value)
    }
}

/// Error enumeration used by services that specifies all the different
/// error kinds that the application backend can encounter.
///
/// It is different from the AppError because it must be translated into it.
/// The translation depends on the context of the trace, for instance, a DoesNotExist
/// database error can be translated into 404 if the client is requesting a non existing entity
/// or into 500 if another service requires an entity and it should be present in the database.
///
/// Moreover, it implements several `From<T>` trait to automatically translate
/// internal errors to AppError using `?`
#[derive(Debug)]
pub enum ServiceAppError {
    /// Authorization error aka 401
    AuthorizationError(AuthError),
    /// AccessControl error aka 403
    AccessControlError(String),
    /// Error that can occur in the `mongodb` crate
    MongoDBBaseAPIError(String),
    /// Error that can occur in the ´database´ service
    DatabaseError(String),
    /// Error that can occur during the build of a response
    ResponseBuildError(String),
    /// Error that can occur when an entity is requested but it does not exist in database
    EntityDoesNotExist(String),
    /// Error that can occur when a request contains parameters that are not valid or the
    /// request cannot be done
    InvalidRequest(String),
    /// InternalServerError
    InternalServerError(String),
}

impl Display for ServiceAppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AuthorizationError(auth_error) => format!("AuthorizationError: {auth_error}"),
                Self::DatabaseError(message) => format!("DatabaseError: {message}"),
                Self::EntityDoesNotExist(message) => format!("EntityDoesNotExist: {message}"),
                Self::InvalidRequest(message) => format!("InvalidRequest: {message}"),
                Self::MongoDBBaseAPIError(message) => format!("MongoDBBaseAPIError: {message}"),
                Self::ResponseBuildError(message) => format!("ResponseBuildError: {message}"),
                Self::InternalServerError(message) => format!("InternalServerError: {message}"),
                Self::AccessControlError(message) => format!("AccessControlError: {message}"),
            }
        )
    }
}

impl From<mongodb::error::Error> for ServiceAppError {
    fn from(value: mongodb::error::Error) -> Self {
        // It is translated into a InternalServerError
        error!("MongoDB error: {:?}", value);
        Self::MongoDBBaseAPIError(value.kind.to_string())
    }
}

impl From<DatabaseError> for ServiceAppError {
    fn from(value: DatabaseError) -> Self {
        error!("Database error: {:?}", value);
        match value {
            DatabaseError::TransactionNotStarted => {
                ServiceAppError::DatabaseError("Transaction not started".into())
            }
            DatabaseError::TransactionClosed => {
                ServiceAppError::DatabaseError("Transaction already committed".into())
            }
            DatabaseError::TransactionError => {
                ServiceAppError::DatabaseError("Transaction operation encountered an error".into())
            }
            DatabaseError::DocumentHasAlreadyAnId => ServiceAppError::DatabaseError(
                "Document cannot be inserted in database because it has already an id".into(),
            ),
            DatabaseError::InvalidObjectId => {
                ServiceAppError::DatabaseError("Invalid object id".into())
            }
        }
    }
}

impl From<web_app_response::CompanyInfoBuilderError> for ServiceAppError {
    fn from(value: web_app_response::CompanyInfoBuilderError) -> Self {
        match value {
            web_app_response::CompanyInfoBuilderError::UninitializedField(message) => {
                ServiceAppError::ResponseBuildError(message.into())
            }
            web_app_response::CompanyInfoBuilderError::ValidationError(message) => {
                ServiceAppError::ResponseBuildError(message)
            }
        }
    }
}

impl From<XlsxError> for ServiceAppError {
    fn from(value: XlsxError) -> Self {
        ServiceAppError::InternalServerError(value.to_string())
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
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials".into()),
            AuthError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "Wrong credentials".into()),

            AuthError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials".into())
            }
            AuthError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".into(),
            ),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".into()),
        };
        (status, message)
    }
}

impl Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AuthError::WrongCredentials => "WrongCredentials",
                AuthError::MissingCredentials => "MissingCredentials",
                AuthError::TokenCreation => "TokenCreation",
                AuthError::InvalidToken => "InvalidToken",
                AuthError::InvalidApiKey => "InvalidApiKey",
            }
        )
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
    DocumentHasAlreadyAnId,
    InvalidObjectId,
}
