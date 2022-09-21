use anyhow::Result;
use axum::{routing::get, Router};

use crate::config::global_config;

async fn greetings() -> &'static str {
    "hello world."
}

pub fn app() -> Router {
    Router::new()
}

pub async fn start_server() -> Result<()> {
    axum::Server::bind(&global_config().server().sock())
        .serve(app().into_make_service())
        .await?;
    Ok(())
}
