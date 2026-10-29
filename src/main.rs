use crate::config::{Config, FromEnv};

mod config;
mod network;
mod server;
mod state;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
}
