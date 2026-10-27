use crate::config::Config;

use reqwest::Client;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    http: Client,
    cfg: Arc<Config>,
}
