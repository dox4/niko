use axum::{routing::get, Router};

use crate::config::NiKoServerConfig;

async fn greetings() -> &'static str {
    "hello world."
}

pub fn app() -> Router {
    Router::new()
}
