use crate::utils::now;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use super::{FromEnv, env_opt, env_or, env_req};

pub(crate) struct Cookie {
    hmac_key: Vec<u8>,
    pub name: String,
    domain: Option<String>,
    ttl: u64,
    bind_ip: bool, //If the client ip should be used to generate the HMAC
}

impl FromEnv for Cookie {
    fn from_env() -> Self {
        let hmac_key = env_req("COOKIE__HMAC_KEY");
        let bytes = (hmac_key.len() > 43).then(|| hmac_key.into_bytes()).expect(
            "COOKIE__HMAC_KEY smaller than 44 chars; generate with `openssl rand -base64 32`",
        );

        Cookie {
            hmac_key: bytes,
            name: super::env_or("COOKIE__NAME", "TS_STATUS"),
            domain: env_opt("COOKIE__DOMAIN"),
            ttl: env_or("COOKIE__TTL", "3600")
                .parse()
                .expect("COOKIE__TTL must be a number"),
            bind_ip: env_or("COOKIE__BIND_IP", "1") != "0",
        }
    }
}

type HmacSha256 = Hmac<Sha256>;

impl Cookie {
    fn generate_hmac(&self, exp: u64, ip: &str) -> HmacSha256 {
        let mut hmac =
            HmacSha256::new_from_slice(&self.hmac_key).expect("HMAC accepts variable key size");
        hmac.update(&exp.to_ne_bytes());
        hmac.update(b"|");
        if self.bind_ip {
            hmac.update(ip.as_bytes());
        }
        hmac
    }

    pub fn sign(&self, exp: u64, ip: &str) -> String {
        let hmac = self.generate_hmac(exp, ip).finalize().into_bytes();
        format!("{exp}.{}", URL_SAFE_NO_PAD.encode(hmac))
    }

    pub fn verify_hmac(&self, value: &str, ip: &str) -> bool {
        let Some((c_exp, c_hmac)) = value.split_once('.') else {
            return false;
        };
        let Ok(exp) = c_exp.parse::<u64>() else {
            return false;
        };

        if exp < now() {
            return false;
        }
        let Ok(hmac) = URL_SAFE_NO_PAD.decode(c_hmac) else {
            return false;
        };
        self.generate_hmac(exp, ip).verify_slice(&hmac).is_ok()
    }
}
