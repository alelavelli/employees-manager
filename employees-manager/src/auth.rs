use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderValue},
    RequestPartsExt,
};
use axum_extra::{
    headers::{
        authorization::{Bearer, Credentials},
        Authorization,
    },
    TypedHeader,
};
use jsonwebtoken::{decode, encode, Header, Validation};

use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

use crate::{
    error::{AppError, AuthError},
    model::db_entities::User,
    service::{db::DatabaseDocument, environment::ENVIRONMENT},
    DocumentId,
};

/// Trait for auth info objects that need to return specific information
pub trait AuthInfo: Clone {
    fn user_id(&self) -> &DocumentId;
}

/// Struct containing information that will be encoded inside the jwt token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthClaim {
    pub exp: usize,
    pub user_id: DocumentId,
    pub username: String,
}

impl JWTAuthClaim {
    pub fn build_token(&self, header: &Header) -> Result<String, AuthError> {
        let token = encode(header, &self, &ENVIRONMENT.authentication.jwt_encoding)
            .map_err(|_| AuthError::TokenCreation)?;
        Ok(token)
    }
}

//#[async_trait]
impl<S> FromRequestParts<S> for JWTAuthClaim
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        tracing::debug!("Got bearer token {}", bearer.token());
        // Decode the user data
        let token_data = decode::<JWTAuthClaim>(
            bearer.token(),
            &ENVIRONMENT.authentication.jwt_decoding,
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::error!("Got error {}", e);
            AuthError::InvalidToken
        })?;

        Ok(token_data.claims)
    }
}

//#[async_trait]
impl AuthInfo for JWTAuthClaim {
    fn user_id(&self) -> &DocumentId {
        &self.user_id
    }
}

/// Struct containing api key authentication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct APIKeyAuthClaim {
    pub key: String,
    pub user_id: DocumentId,
}

//#[async_trait]
impl<S> FromRequestParts<S> for APIKeyAuthClaim
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(api_key)) = parts
            .extract::<TypedHeader<Authorization<ApiKey>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let query_result = User::find_one(doc! { "api_key": api_key.key() })
            .await
            .map_err(|_| AppError::AuthorizationError(AuthError::InvalidApiKey))?;
        if let Some(user_document) = query_result {
            let auth_data = APIKeyAuthClaim {
                user_id: *user_document
                    .get_id()
                    .expect("User id must be not missing since we have an api key"),
                key: api_key.key().into(),
            };

            Ok(auth_data)
        } else {
            Err(AppError::AuthorizationError(AuthError::InvalidApiKey))
        }
    }
}

#[async_trait]
impl AuthInfo for APIKeyAuthClaim {
    fn user_id(&self) -> &DocumentId {
        &self.user_id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApiKey(String);

impl ApiKey {
    /// View the token part as a `&str`.
    pub fn key(&self) -> &str {
        self.0.as_str()["x-api-key ".len()..].trim_start()
    }
}

impl Credentials for ApiKey {
    const SCHEME: &'static str = "x-api-key";

    fn decode(value: &HeaderValue) -> Option<Self> {
        debug_assert!(
            value.as_bytes()[..Self::SCHEME.len()].eq_ignore_ascii_case(Self::SCHEME.as_bytes()),
            "HeaderValue to decode should start with \"x-api-key ..\", received = {:?}",
            value,
        );

        value.to_str().ok().map(|s| ApiKey(s.to_string()))
    }

    fn encode(&self) -> HeaderValue {
        HeaderValue::from_str(&self.0).unwrap()
    }
}
