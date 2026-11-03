use time::OffsetDateTime;

pub fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

pub fn is_safe_path(uri: &str) -> bool {
    uri.starts_with('/') && !uri.starts_with("//") && !uri.starts_with("/\\")
}
