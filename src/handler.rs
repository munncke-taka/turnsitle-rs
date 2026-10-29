use std::env;
use std::sync::LazyLock;

use axum::Router;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use reqwest::StatusCode;

use crate::AppState;

const TEMPLATE: &str = include_str!("./public/challenge.html");
static INDEX: LazyLock<String> = LazyLock::new(|| {
    let var = |k: &str| env::var(k).unwrap_or_else(|_| panic!("{k} not set"));
    TEMPLATE
        .replace("{{CHALLENGE__TITLE}}", &var("APP_TITLE"))
        .replace("{{CHALLENGE__API_URL}}", &var("API_URL"))
        .replace("{{CHALLENGE__VERSION}}", &var("APP_VERSION"))
});

async fn get_challenge() -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}

async fn post_challenge() -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}

async fn get_verify() -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
    // let ip = client_ip(&h);
    // match get_cookie(&h, &s.cfg.cookie_name) {
    //     Some(v) if verify_cookie(&s.cfg, v, &ip) => StatusCode::NO_CONTENT,
    //     _ => StatusCode::UNAUTHORIZED,
    // }
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
fn read_cookie<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|pair| {
            let (k, v) = pair.trim().split_once('=')?;
            (k == name).then_some(v)
        })
}
