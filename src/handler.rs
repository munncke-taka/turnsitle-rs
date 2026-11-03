use std::env;
use std::sync::LazyLock;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Form, Router};
use axum_extra::extract::CookieJar;
use reqwest::StatusCode;

use crate::AppState;
use crate::extract::ClientIp;

async fn get_challenge() -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}

async fn post_challenge(State(s): State<AppState>, ClientIp(ip): ClientIp) -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}

async fn get_verify(State(s): State<AppState>, ClientIp(ip): ClientIp, jar: CookieJar) -> Response {
    match jar.get(&s.cfg.cookie.name) {
        Some(c) => {
            if s.cfg.cookie.verify_hmac(c.value(), &ip.to_string()) {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::UNAUTHORIZED.into_response()
            }
        }
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

pub fn new(state: AppState) -> Router {
    Router::new()
        .route("/_ts/challenge", get(get_challenge).post(post_challenge))
        .route("/_ts/verify", get(get_verify))
        .with_state(state)
}

// use axum::{
//     Json, Router,
//     extract::State,
//     http::{HeaderMap, StatusCode, header},
//     response::{Html, IntoResponse, Response},
//     routing::{get, post},
// };
// use serde::Deserialize;

// use crate::{AppState, extract::ClientIp};

// pub fn router(state: AppState) -> Router {
//     Router::new()
//         .route("/", get(index))
//         .route("/verify", post(verify))
//         .route("/check", get(check))
//         .with_state(state)
// }

// /// Serves the pre-rendered challenge page.
// async fn index(State(s): State<AppState>) -> Html<String> {
//     Html(s.config.index_html.clone())
// }

// #[derive(Deserialize)]
// struct VerifyBody {
//     token: String,
// }

// /// Validates the Turnstile token and, on success, issues the signed cookie.
// async fn verify(
//     State(s): State<AppState>,
//     ClientIp(ip): ClientIp,
//     Json(body): Json<VerifyBody>, // body extractor must stay last
// ) -> Response {
//     if !s.config.turnstile.verify(&s.http, &body.token, ip).await {
//         return StatusCode::FORBIDDEN.into_response();
//     }

//     let set_cookie = s.config.cookie.issue(ip);
//     (StatusCode::NO_CONTENT, [(header::SET_COOKIE, set_cookie)]).into_response()
// }

// /// 204 if the request carries a valid cookie for this IP, 401 otherwise.
// /// Suitable as an nginx `auth_request` target.
// async fn check(
//     State(s): State<AppState>,
//     ClientIp(ip): ClientIp,
//     headers: HeaderMap,
// ) -> StatusCode {
//     let cookie = &s.config.cookie;
//     match read_cookie(&headers, cookie.name()) {
//         Some(value) if cookie.verify(value, ip) => StatusCode::NO_CONTENT,
//         _ => StatusCode::UNAUTHORIZED,
//     }
// }

// /// Finds `name` in the request's Cookie header(s).
