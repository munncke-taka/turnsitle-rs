mod config;
mod extract;
mod handler;
mod utils;

use reqwest::Client;
use std::sync::Arc;

use crate::config::{Config, FromEnv};

#[derive(Clone)]
pub struct AppState {
    http: Client,
    cfg: Arc<Config>,
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
}
