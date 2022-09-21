use anyhow::Result;

use niko::{api, config, db, log, storage};

#[tokio::main]
async fn main() -> Result<()> {
    config::init_config("niko.toml");
    log::init_log();
    db::init_db().await;
    storage::may_need_scanning().await?;
    tokio::spawn(storage::start_notify());
    api::start_server().await?;
    Ok(())
}
