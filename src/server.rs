use std::env;
use std::sync::LazyLock;

use axum::response::Html;

const TEMPLATE: &str = include_str!("./public/challenge.html");
static INDEX: LazyLock<String> = LazyLock::new(|| {
    let var = |k: &str| env::var(k).unwrap_or_else(|_| panic!("{k} not set"));
    TEMPLATE
        .replace("{{CHALLENGE__TITLE}}", &var("APP_TITLE"))
        .replace("{{CHALLENGE__API_URL}}", &var("API_URL"))
        .replace("{{CHALLENGE__VERSION}}", &var("APP_VERSION"))
});
