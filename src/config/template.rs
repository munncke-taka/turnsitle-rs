use std::fs;

use crate::config::env_opt;

use super::{FromEnv, env_req};

const BASE_TEMPLATE: &str = include_str!("../public/challenge.html");

struct Template {
    custom_path: Option<String>,
    title: String,
    url: String,
    version: String,
}

impl FromEnv for Template {
    fn from_env() -> Self {
        Template {
            custom_path: env_opt("TEMPLATE__CUSTOM_PATH"),
            title: env_req("TEMPLATE__TITLE"),
            url: env_req("TEMPLATE__URL"),
            version: env_req("TEMPLATE__VERSION"),
        }
    }
}

pub fn build_template() -> &'static str {
    let vars = Template::from_env();

    let html = match &vars.custom_path {
        Some(path) => fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read template at {path}: {e}")),
        None => BASE_TEMPLATE.to_string(),
    };

    html.replace("{{TITLE}}", &vars.title)
        .replace("{{URL}}", &vars.url)
        .replace("{{VERSION}}", &vars.version)
        .leak()
}
