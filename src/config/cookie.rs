use std::net::IpAddr;

use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::StatusCode;
use sha2::Sha256;
use time::Duration;

use crate::config::{FromEnv, env_opt, env_or, env_req};
use crate::utils::now;

pub(crate) enum HmacCookieError {
    Malformed,
    Expired,
    MissingIp,
    InvalidCookie,
    SignError,
}

pub(crate) struct CheckCookie {
    hmac_key: Vec<u8>,
    domain: Option<String>,
    pub name: String,
    pub secure: bool,
    pub ttl: Duration,
    pub bind_ip: bool, //If the client ip should be used to generate the HMAC and be sent to Cloudflare
}

impl IntoResponse for HmacCookieError {
    fn into_response(self) -> Response {
        match self {
            HmacCookieError::Malformed => eprintln!("Malformed Cookie for verification"),
            HmacCookieError::Expired => eprintln!("Expired cookie"),
            HmacCookieError::MissingIp => eprintln!("Missing required IP for cookie"),
            HmacCookieError::InvalidCookie => eprintln!("Invalid Cookie"),
            HmacCookieError::SignError => {
                eprintln!("Error during signing");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
        StatusCode::UNAUTHORIZED.into_response()
    }
}

impl FromEnv for CheckCookie {
    fn from_env() -> Self {
        let hmac_key = env_req("CHECK_COOKIE__HMAC_KEY");
        let bytes = (hmac_key.len() > 43).then(|| hmac_key.into_bytes()).expect(
            "CHECK_COOKIE__HMAC_KEY smaller than 44 chars; generate with `openssl rand -base64 32`",
        );
        let ttl = Duration::seconds(
            env_or("CHECK_COOKIE__TTL", "3600")
                .parse()
                .expect("CHECK_COOKIE__TTL must be a i64 number for seconds"),
        );

        CheckCookie {
            hmac_key: bytes,
            name: super::env_or("CHECK_COOKIE__NAME", "TS_STATUS"),
            domain: env_opt("CHECK_COOKIE__DOMAIN"),
            ttl: ttl,
            bind_ip: env_or("CHECK_COOKIE__BIND_IP", "true")
                .parse()
                .expect("CHECK_COOKIE__BIND_IP must be a valid bool"),
            secure: env_or("CHECK_COOKIE__SECURE", "true")
                .parse()
                .expect("CHECK_COOKIE__SECURE must be a valid bool"),
        }
    }
}

type HmacSha256 = Hmac<Sha256>;

impl CheckCookie {
    fn generate_hmac(&self, exp: i64, ip: Option<IpAddr>) -> Result<HmacSha256, HmacCookieError> {
        let mut hmac =
            HmacSha256::new_from_slice(&self.hmac_key).map_err(|_| HmacCookieError::SignError)?;
        hmac.update(&exp.to_ne_bytes());
        hmac.update(b"|");
        match (self.bind_ip, ip) {
            (true, Some(ip)) => hmac.update(ip.to_string().as_bytes()),
            (true, None) => return Err(HmacCookieError::MissingIp),
            (false, _) => {}
        }
        Ok(hmac)
    }

    pub fn sign(&self, exp: i64, ip: Option<IpAddr>) -> Result<String, HmacCookieError> {
        let hmac = self.generate_hmac(exp, ip)?.finalize().into_bytes();
        Ok(format!("{exp}.{}", URL_SAFE_NO_PAD.encode(hmac)))
    }

    pub fn verify_hmac(&self, value: &str, ip: Option<IpAddr>) -> Result<(), HmacCookieError> {
        let (c_exp, c_hmac) = value.split_once('.').ok_or(HmacCookieError::Malformed)?;
        let exp = c_exp
            .parse::<i64>()
            .map_err(|_| HmacCookieError::Malformed)?;

        if exp < now() {
            return Err(HmacCookieError::Expired);
        }

        let hmac = URL_SAFE_NO_PAD
            .decode(c_hmac)
            .map_err(|_| HmacCookieError::Malformed)?;

        self.generate_hmac(exp, ip)
            .map_err(|_| HmacCookieError::MissingIp)?
            .verify_slice(&hmac)
            .map_err(|_| HmacCookieError::InvalidCookie)
    }

    // pub fn verify_hmac(&self, value: &str, ip: Option<IpAddr>) -> Result<(), VerifyError> {
    //     let Some((c_exp, c_hmac)) = value.split_once('.') else {
    //         return Err(VerifyError::BadCookie);
    //     };
    //     let Ok(exp) = c_exp.parse::<u64>() else {
    //         return Err(VerifyError::BadCookie);
    //     };
    //     if exp < now() {
    //         return Err(VerifyError::ExpiredCookie);
    //     }
    //     let Ok(hmac) = URL_SAFE_NO_PAD.decode(c_hmac) else {
    //         return Err(VerifyError::BadCookie);
    //     };
    //     let mac = self.generate_hmac(exp, ip)?;

    //     match mac.verify_slice(&hmac) {
    //         Ok(()) => Ok(()),
    //         Err(_) => Err(VerifyError::InvalidCookie),
    //     }
    // }
}
