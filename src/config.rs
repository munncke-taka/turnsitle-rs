use std::env;

// #[derive(Clone, Copy, Debug)]
// pub struct Cidr {
//     net: IpAddr,
//     len: u8,
// }

pub trait FromEnv {
    fn from_env() -> Self;
}

pub struct Template {
    title: String,
    url: String,
    version: String,
}

pub struct Turnstile {
    secret: String,
    sitekey: String,
}

pub struct Cookie {
    hmac_key: Vec<u8>,
    name: String,
    domain: Option<String>,
    ttl: u64,
    bind_ip: bool, //If the client ip should be used to generate the HMAC
}

pub struct Config {
    pub turnstile: Turnstile,
    pub cookie: Cookie,
    pub custom_template_path: Option<String>,
    pub template: Template,
    pub ip_header: String,
}

fn env_req(k: &str) -> String {
    env::var(k).unwrap_or_else(|_| panic!("ERROR: Missing required env var {k}"))
}

fn env_or(k: &str, d: &str) -> String {
    env::var(k).unwrap_or_else(|_| d.to_string())
}

fn env_opt(k: &str) -> Option<String> {
    env::var(k).ok()
}

// impl Cidr {
//     fn parse(s: &str) -> Option<Cidr> {
//         let (addr, len) = match s.split_once('/') {
//             Some((a, l)) => (a, Some(l)),
//             None => (s, None),
//         };
//         let net: IpAddr = addr.trim().parse().ok()?;
//         let max = if net.is_ipv4() { 32 } else { 128 };
//         let len = match len {
//             Some(l) => l.trim().parse::<u8>().ok()?,
//             None => max,
//         };
//         (len <= max).then_some(Cidr { net, len })
//     }
//     fn contains(&self, ip: IpAddr) -> bool {
//         // to_canonical() turns IPv4-mapped IPv6 (::ffff:1.2.3.4) into plain IPv4
//         match (self.net, ip.to_canonical()) {
//             (IpAddr::V4(n), IpAddr::V4(i)) => {
//                 let m = u32::MAX.checked_shl(32 - self.len as u32).unwrap_or(0);
//                 u32::from(n) & m == u32::from(i) & m
//             }
//             (IpAddr::V6(n), IpAddr::V6(i)) => {
//                 let m = u128::MAX.checked_shl(128 - self.len as u32).unwrap_or(0);
//                 u128::from(n) & m == u128::from(i) & m
//             }
//             _ => false,
//         }
//     }
//     /// ALLOW_CIDRS="10.0.0.0/8, 192.168.1.0/24 2001:db8::/32" (comma and/or space separated).
//     /// Invalid entries abort startup so a typo never silently disables the allowlist.
//     fn parse_allowlist(raw: &str) -> Vec<Cidr> {
//         raw.split(|c: char| c == ',' || c.is_whitespace())
//             .filter(|s| !s.is_empty())
//             .map(|s| Cidr::parse(s).unwrap_or_else(|| panic!("invalid CIDR in ALLOW_CIDRS: {s:?}")))
//             .collect()
//     }
//     fn from_env() -> Option<Vec<Cidr>> {
//         if let Some(raw) = env_opt("IP_WHITELIST") {
//             raw.split(|c: char| c == ',' || c.is_whitespace());
//             return None;
//         }
//         None
//     }
// }

impl FromEnv for Template {
    fn from_env() -> Self {
        Template {
            title: env_req("TEMPLATE__TITLE"),
            url: env_req("TEMPLATE__URL"),
            version: env_req("TEMPLATE__VERSION"),
        }
    }
}

impl FromEnv for Turnstile {
    fn from_env() -> Self {
        Turnstile {
            secret: env_req("TURNSTILE__SECRET"),
            sitekey: env_req("TURNSTILE__SITEKEY"),
        }
    }
}

impl FromEnv for Cookie {
    fn from_env() -> Self {
        let hmac_key = env_req("COOKIE__HMAC_KEY");
        let bytes = (hmac_key.len() > 43).then(|| hmac_key.into_bytes()).expect(
            "COOKIE__HMAC_KEY smaller than 44 chars; generate with `openssl rand -base64 32`",
        );

        Cookie {
            hmac_key: bytes,
            name: env_or("COOKIE__NAME", "TS_STATUS"),
            domain: env_opt("COOKIE__DOMAIN"),
            ttl: env_or("COOKIE__TTL", "3600")
                .parse()
                .expect("COOKIE__TTL must be a number"),
            bind_ip: env_or("COOKIE__BIND_IP", "1") != "0",
        }
    }
}

impl FromEnv for Config {
    fn from_env() -> Self {
        Config {
            turnstile: Turnstile::from_env(),
            cookie: Cookie::from_env(),
            custom_template_path: env_opt("CUSTOM_TEMPLATE_PATH"),
            template: Template::from_env(),
            ip_header: env_or("IP_HEADER", "X-Forwarded-From"),
        }
    }
}
