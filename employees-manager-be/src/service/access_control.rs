use crate::{auth::AuthInfo, error::AppError, service::user::get_user};

/// Access control struct that validate and verify the
/// role of the user
pub struct AccessControl<T: AuthInfo> {
    auth_info: T,
}

impl<T: AuthInfo> AccessControl<T> {
    pub fn new(auth_info: T) -> AccessControl<T> {
        AccessControl { auth_info }
    }
    /// Verify that the user has ADMIN role, otherwise it
    /// returns AccessControlError
    pub async fn is_platform_admin(self) -> Result<Self, AppError> {
        let user = get_user(self.auth_info.user_id()).await?;
        if user.platform_admin {
            Ok(self)
        } else {
            Err(AppError::AccessControlError)
        }
    }
}
