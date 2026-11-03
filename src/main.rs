mod app_error;
mod config;
mod extract;
mod handler;
mod utils;

use dotenvy::dotenv;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

use crate::config::template;
use crate::config::{Config, FromEnv};

#[derive(Clone)]
pub struct AppState {
    http: Client,
    cfg: Arc<Config>,
    html: &'static str,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let config = Config::from_env();
    let html = template::build_template();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Error on creating HTTP Client");

    let state = AppState {
        http,
        html,
        cfg: Arc::new(config),
    };

    let app = handler::new(state);
    let listen = config::listen_address();

    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .expect("Failed to initialize TCP Listener bind");

    println!("Listenting on {listen}");

    axum::serve(listener, app)
        .await
        .expect("Error on initializing Axum");
}
