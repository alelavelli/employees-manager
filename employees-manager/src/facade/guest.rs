use jsonwebtoken::Header;

use crate::{
    auth::JWTAuthClaim,
    dtos::web_app_response,
    error::{AppError, AuthError},
    service::{db::document::DatabaseDocument, user::UserService},
    APP_STATE,
};

/// Facade with operations for unauthenticated requests
pub struct GuestFacade {}

impl GuestFacade {
    pub async fn authenticate_user(
        username: &str,
        password: &str,
    ) -> Result<web_app_response::JWTAuthResponse, AppError> {
        let user_model = UserService::login(username, password).await?;
        let exp = APP_STATE
            .try_with(|state| state.clone())
            .map_err(|e| AuthError::InternalServerError(e.to_string()))?
            .environment_service
            .get_authentication_jwt_expiration();
        let now = chrono::offset::Local::now().timestamp();
        let claims = JWTAuthClaim {
            exp: now as u32 + exp as u32,
            user_id: *user_model.get_id().expect("User id must be not missing"),
            username: user_model.username().clone(),
        };
        let token = claims.build_token(&Header::default())?;

        Ok(web_app_response::JWTAuthResponse {
            token,
            token_type: "Bearer".into(),
        })
    }
}
