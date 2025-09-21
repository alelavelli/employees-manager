//! Data Transferable Objects modules contains definition of json compliant object
//! that are transferred between application and client.
//!
//! Information is shared by `requests` and `responses` and they are specific for each
//! route, therefore, we have SDK requests and responses and Web app requests and responses.

use axum::{
    extract::FromRequest,
    http::{HeaderMap, HeaderName, HeaderValue},
    response::{IntoResponse, Response},
};

use crate::error::AppError;

pub mod sdk_request;
pub mod sdk_response;
pub mod web_app_request;
pub mod web_app_response;
// Create our own JSON extractor by wrapping `axum::Json`. This makes it easy to override the
// rejection and provide our own which formats errors to match our application.
//
// `axum::Json` responds with plain text if the input is invalid.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

pub struct ResponseWithHeader<T>
where
    T: IntoResponse,
{
    headers: HeaderMap,
    content: T,
}

impl<T> ResponseWithHeader<T>
where
    T: IntoResponse,
{
    pub fn new(content: T) -> Self {
        Self {
            headers: HeaderMap::new(),
            content,
        }
    }

    pub fn with_header(self, header: HeaderName, value: HeaderValue) -> Self {
        let mut new_headers = self.headers;
        new_headers.insert(header, value);
        Self {
            headers: new_headers,
            content: self.content,
        }
    }
}

impl<T> IntoResponse for ResponseWithHeader<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        (self.headers, self.content).into_response()
    }
}
