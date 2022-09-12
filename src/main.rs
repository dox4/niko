use notify::{RecommendedWatcher, Watcher};
use std::thread::sleep;
use std::time::Duration;
use std::{error::Error, path::Path};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tracing::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};
use walkdir::WalkDir;

use niko::{
    db,
    model::{self, file::File},
};

async fn init_file_table() -> Result<(), sqlx::Error> {
    for entry in WalkDir::new(".")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f = File {
            id: 0,
            parent: entry.path().parent().unwrap().display().to_string(),
            name: entry.file_name().to_string_lossy().into(),
            is_dir: entry.file_type().is_dir(),
            size: if entry.file_type().is_dir() {
                0
            } else {
                entry.metadata().unwrap().len() as i32
            },
            permission: entry.metadata().unwrap().permissions().mode() as i32,
            created_at: entry.metadata().unwrap().created().unwrap().into(),
            updated_at: entry.metadata().unwrap().modified().unwrap().into(),
            deleted_at: None,
        };
        println!("create file {:?}", f);
        let id = niko::model::file::create(f).await?;

        println!(
            "file record created.... {:?}",
            niko::model::file::fetch_by_id(id).await?
        );
    }
    Ok(())
}

async fn initialize_before_serving() -> Result<(), sqlx::Error> {
    let result =
        model::metadata::find_key_updated_at(model::metadata::KEY_WALKING_DIR.into()).await;
    match result {
        Ok(time) => {
            println!("last updated at {:?}", time);
        }
        Err(err) => {
            println!("error occured while query the updating time, {:?}", err);
            println!("we will scanning the target dir before serving.");
            init_file_table().await?;
            model::metadata::create_key(
                model::metadata::KEY_WALKING_DIR.to_string(),
                "done".into(),
            )
            .await?;
        }
    };
    Ok(())
}

async fn notify_dir() -> Result<(), notify::Error> {
    let mut watcher =
    // To make sure that the config lives as long as the function
    // we need to move the ownership of the config inside the function
    // To learn more about move please read [Using move Closures with Threads](https://doc.rust-lang.org/book/ch16-01-threads.html?highlight=move#using-move-closures-with-threads)
    RecommendedWatcher::new(move |result: Result<notify::Event, notify::Error>| {
        let event = result.unwrap();
        println!("event: {:?}", event);
    },notify::Config::default())?;
    watcher.watch(Path::new("."), notify::RecursiveMode::Recursive)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let conf_data = fs::read("niko.toml").unwrap();
    let nk = niko::config::from_slice(&conf_data[..]);
    // tracing_subscriber::fmt::format().with_level(true);
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    db::init_db(&nk.db).await;
    debug!("{:?}", db::global_pool());
    println!("{:?}", model::file::paged_fetch(1, 20).await.unwrap());
    initialize_before_serving().await?;
    notify_dir().await?;
    loop {
        sleep(Duration::new(1, 2000));
    }
    Ok(())
}
