use std::net::IpAddr;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::AppState;

pub struct ClientIp(pub IpAddr);

pub enum ClientIpError {
    Missing,
    Invalid,
}

impl IntoResponse for ClientIpError {
    fn into_response(self) -> Response {
        match self {
            ClientIpError::Missing => eprintln!("Client IP header missing"),
            ClientIpError::Invalid => eprintln!("Client IP header invalid"),
        }
        StatusCode::BAD_REQUEST.into_response()
    }
}

impl FromRequestParts<AppState> for ClientIp {
    type Rejection = ClientIpError;

    async fn from_request_parts(parts: &mut Parts, s: &AppState) -> Result<Self, Self::Rejection> {
        let raw = parts
            .headers
            .get(&s.cfg.ip_header)
            .ok_or(ClientIpError::Missing)?
            .to_str()
            .map_err(|_| ClientIpError::Invalid)?;

        raw.trim()
            .parse::<IpAddr>()
            .map(ClientIp)
            .map_err(|_| ClientIpError::Invalid)
    }
}
