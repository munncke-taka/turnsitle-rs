use std::net::IpAddr;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::utils::is_safe_path;

pub struct ClientCtx(pub Option<IpAddr>, pub String);

pub enum ClientCtxError {
    MissingIP,
    InvalidIP,
}

impl IntoResponse for ClientCtxError {
    fn into_response(self) -> Response {
        match self {
            ClientCtxError::MissingIP => eprintln!("Client IP header missing"),
            ClientCtxError::InvalidIP => eprintln!("Client IP header invalid"),
        }
        StatusCode::BAD_REQUEST.into_response()
    }
}

impl FromRequestParts<AppState> for ClientCtx {
    type Rejection = ClientCtxError;

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Self, Self::Rejection> {
        let ip = if s.cfg.check_cookie.bind_ip {
            Some(
                parts
                    .headers
                    .get(&s.cfg.ip_header)
                    .ok_or(ClientCtxError::MissingIP)?
                    .to_str()
                    .map_err(|_| ClientCtxError::InvalidIP)?
                    .trim()
                    .parse::<IpAddr>()
                    .map_err(|_| ClientCtxError::InvalidIP)?,
            )
        } else {
            None
        };

        let original_uri = parts
            .headers
            .get(&s.cfg.original_uri_header)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|u| is_safe_path(u))
            .map(str::to_owned)
            .unwrap_or_else(|| "/".to_string());

        Ok(Self(ip, original_uri))
    }
}
