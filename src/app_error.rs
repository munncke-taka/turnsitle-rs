use axum::response::{IntoResponse, Response};

use crate::{config::cookie::HmacCookieError, extract::ClientCtxError};

pub enum AppError {
    HmacCookieError(HmacCookieError),
    ClientCtxError(ClientCtxError),
}

impl From<HmacCookieError> for AppError {
    fn from(e: HmacCookieError) -> Self {
        AppError::HmacCookieError(e)
    }
}

impl From<ClientCtxError> for AppError {
    fn from(e: ClientCtxError) -> Self {
        AppError::ClientCtxError(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::HmacCookieError(e) => e.into_response(),
            AppError::ClientCtxError(e) => e.into_response(),
        }
    }
}
