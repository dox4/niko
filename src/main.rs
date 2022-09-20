use std::error::Error;

use niko::{api, config::config, db, log, storage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    log::init_log(config().log());
    db::init_db(config().db()).await;
    storage::may_need_scanning(config().dir().clone()).await?;
    let dir = config().dir().clone();
    tokio::spawn(storage::start_notify(dir));
    axum::Server::bind(&config().server().sock())
        .serve(api::app().into_make_service())
        .await
        .unwrap();
    Ok(())
}
