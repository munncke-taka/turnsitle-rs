use std::net::IpAddr;

use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Form, Router};
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::app_error::AppError;
use crate::extract::ClientCtx;
use crate::utils::{is_safe_path, now};

const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

#[derive(Debug, Deserialize)]
struct SiteverifyResponse {
    success: bool,
    #[serde(rename = "error-codes", default)]
    error_codes: Vec<String>,
    challenge_ts: Option<String>,
    hostname: Option<String>,
    action: Option<String>,
    cdata: Option<String>,
}

#[derive(Debug, Serialize)]
struct SiteverifyRequest<'a> {
    secret: &'a str,
    response: &'a str,
    idempotency_key: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    remoteip: Option<IpAddr>,
}

#[derive(Debug, Deserialize)]
struct ChallengeForm {
    #[serde(rename = "cf-turnstile-response")]
    token: String,
    next: Option<String>,
}

impl SiteverifyResponse {
    fn internal_error() -> Self {
        Self {
            success: false,
            error_codes: vec!["internal-error".into()],
            challenge_ts: None,
            hostname: None,
            action: None,
            cdata: None,
        }
    }
}

async fn get_challenge() -> Response {
    StatusCode::NOT_IMPLEMENTED.into_response()
}

async fn post_challenge(
    State(s): State<AppState>,
    ClientCtx(ip, _): ClientCtx,
    jar: CookieJar,
    Form(f): Form<ChallengeForm>,
) -> Result<Response, AppError> {
    let idempotency_key = Uuid::new_v4();
    let request = SiteverifyRequest {
        secret: &s.cfg.turnstile.secret,
        response: &f.token,
        idempotency_key,
        remoteip: ip,
    };

    let result = validate_with_retry(&s.http, request, s.cfg.turnstile.max_retries).await;

    if result.success {
        let next = f
            .next
            .as_deref()
            .map(str::trim)
            .filter(|u| is_safe_path(u))
            .unwrap_or("/");

        let value = s
            .cfg
            .check_cookie
            .sign(now() + s.cfg.check_cookie.ttl, ip)?;
        let cookie = Cookie::build((s.cfg.check_cookie.name, value))
            .path("/")
            .http_only(true)
            .secure(s.cfg.check_cookie.secure)
            .same_site(SameSite::Lax)
            .max_age(s.cfg.check_cookie.ttl);

        Ok((jar.add(cookie), Redirect::to(next)).into_response())
    } else {
        todo!()
    }
}

async fn get_verify(
    State(s): State<AppState>,
    ClientCtx(ip, _): ClientCtx,
    jar: CookieJar,
) -> Response {
    match jar.get(&s.cfg.check_cookie.name) {
        Some(c) => match s.cfg.check_cookie.verify_hmac(c.value(), ip) {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(_) => StatusCode::UNAUTHORIZED.into_response(),
        },
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

pub fn new(state: AppState) -> Router {
    Router::new()
        .route("/_ts/challenge", get(get_challenge).post(post_challenge))
        .route("/_ts/verify", get(get_verify))
        .with_state(state)
}

async fn validate_with_retry(
    client: &Client,
    form: SiteverifyRequest<'_>,
    max_retries: u32,
) -> SiteverifyResponse {
    for attempt in 0..=max_retries {
        let result = async {
            let resp = client.post(SITEVERIFY_URL).form(&form).send().await?;
            let ok = resp.status().is_success();
            let body: SiteverifyResponse = resp.json().await?;
            Ok::<_, reqwest::Error>((ok, body))
        }
        .await;

        match result {
            // Success, or the last attempt: return whatever Cloudflare said
            Ok((true, body)) => return body,
            Ok((false, body)) if attempt == max_retries => return body,
            // Network/parse failure on the last attempt
            Err(e) if attempt == max_retries => {
                eprintln!("Turnstile siteverify failed: {e}");
                return SiteverifyResponse::internal_error();
            }
            Ok((false, _)) => {}
            Err(e) => eprintln!("Turnstile attempt {attempt} failed: {e}"),
        }

        // Exponential backoff: 2s, 4s, 8s…
        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
    }
    SiteverifyResponse::internal_error()
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
