use std::error::Error;
use std::fs;
use tracing::*;

use niko::{api, config::NiKoConfig, db, fs as nikofs, log, model};

async fn initialize_before_serving(nkconf: &NiKoConfig<'_>) -> Result<(), sqlx::Error> {
    let result =
        model::metadata::find_key_updated_at(model::metadata::KEY_WALKING_DIR.into()).await;
    match result {
        Ok(time) => {
            debug!("last updated at {:?}", time);
        }
        Err(err) => {
            warn!("error occured while query the updating time, {:?}", err);
            info!("we will scanning the target dir before serving.");
            nikofs::may_need_scanning(nkconf.dir()).await?;
            model::metadata::create_key(
                model::metadata::KEY_WALKING_DIR.to_string(),
                "done".into(),
            )
            .await?;
        }
    };
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf_data = fs::read("niko.toml").unwrap();
    let nk = niko::config::from_slice(&conf_data[..]);
    log::init_log(nk.log());
    db::init_db(nk.db()).await;
    initialize_before_serving(&nk).await?;
    axum::Server::bind(&nk.server().sock())
        .serve(api::app().into_make_service())
        .await
        .unwrap();
    Ok(())
}
