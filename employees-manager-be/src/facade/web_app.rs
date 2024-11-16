use jsonwebtoken::Header;

use crate::{
    auth::{AuthInfo, JWTAuthClaim},
    dtos::{web_app_request, web_app_response},
    error::AppError,
    service::{company, user},
    DocumentId,
};

pub async fn authenticate_user(
    username: &str,
    password: &str,
) -> Result<web_app_response::JWTAuthResponse, AppError> {
    let user_model = user::login(username, password).await?;

    let claims = JWTAuthClaim {
        exp: 2000000000,
        user_id: user_model.id.expect("User id must be not missing"),
        username: user_model.username,
    };
    let token = claims.build_token(&Header::default())?;

    Ok(web_app_response::JWTAuthResponse {
        token,
        token_type: "Bearer".into(),
    })
}

pub async fn create_company(
    auth_info: impl AuthInfo,
    payload: web_app_request::CreateCompany,
) -> Result<String, AppError> {
    // any user can create a new company hence, we don't have access control
    // however, we need to verify that the User does not have already a Company
    // with the same name
    let user_companies = company::get_user_companies(auth_info.user_id()).await?;
    for company in user_companies {
        if payload.name == company.name {
            return Err(AppError::ManagedError(format!(
                "Failed to create Company: Company with name {} already exists.",
                payload.name
            )))?;
        }
    }
    company::create_company(auth_info.user_id(), payload.name, payload.job_title).await
}

pub async fn get_company(
    auth_info: impl AuthInfo,
    company_id: DocumentId,
) -> Result<web_app_response::Company, AppError> {
    // any User can read his companies hence, we don't have access control
    let company_model = company::get_user_company(auth_info.user_id(), &company_id).await?;
    Ok(web_app_response::Company {
        id: company_model
            .id
            .expect("field company_id should exist since the model comes from a db query"),
        name: company_model.name,
    })
}

pub async fn add_company_user(
    auth_info: impl AuthInfo,
    payload: web_app_request::AddCompanyUser,
) -> Result<(), AppError> {
    // only company owner and admin can add users to the company
    todo!()
}

pub async fn remove_company_user(
    auth_info: impl AuthInfo,
    username: String,
) -> Result<(), AppError> {
    todo!()
}
