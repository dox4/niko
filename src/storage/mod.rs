use chrono::Local;
use notify::event::RemoveKind;
use notify::Watcher;
use tokio::sync::mpsc::channel;
use tracing::*;
use walkdir::WalkDir;

use crate::config::NiKoDirConfig;
use crate::model::{file, metadata};

pub async fn may_need_scanning(dir: &NiKoDirConfig<'_>) -> Result<(), sqlx::Error> {
    let result = metadata::find_key_updated_at(metadata::KEY_WALKING_DIR.into()).await;
    match result {
        Ok(time) => {
            info!("last updated at {:?}", time);
            let now = Local::now();
            let offset_hours = now.signed_duration_since(time).num_hours();
            if offset_hours > 12 {
                info!("it's over 12 hours after last scanning, we will rescan the target dir....");
                scan(dir).await?;
            }
        }
        Err(err) => {
            warn!("error occured while query the updating time, {:?}", err);
            info!("we will scanning the target dir before serving.");
            scan(dir).await?;
            metadata::create_key(metadata::KEY_WALKING_DIR.to_string(), "done".into()).await?;
        }
    };
    Ok(())
}
async fn scan(conf: &NiKoDirConfig<'_>) -> Result<(), sqlx::Error> {
    for entry in WalkDir::new(conf.full_path())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let parent = entry.path().parent().unwrap().display().to_string();
        let name: String = entry.file_name().to_string_lossy().into();
        let md = entry.metadata().unwrap();
        match file::fetch_one_by_path(parent.clone(), name.clone()).await {
            Ok(fetched) => {
                let mut f = file::Entry::from_metadata(md, parent, name);
                f.id = fetched.id;
                file::update_by_id(f)
                    .await
                    .and_then(|x| Ok(assert_eq!(x, 1, "rows affected more than 1: {:?}", x)))
                    .unwrap()
            }
            Err(sqlx::Error::RowNotFound) => {
                let f = file::Entry::from_metadata(md, parent, name);
                debug!("start creating entry: {:?}", f);
                let id = file::create(f).await?;
                info!(
                    "entry record created.... {:?}",
                    file::fetch_by_id(id).await?
                );
            }
            Err(err) => panic!("fetch record at error {:?}", err),
        }
    }
    Ok(())
}

async fn process_remove_event(
    rk: RemoveKind,
    event: notify::event::Event,
) -> Result<(), sqlx::Error> {
    match rk {
        notify::event::RemoveKind::File => {
            for pb in event.paths {
                file::delete_by_path(pb).await?;
            }
        }
        notify::event::RemoveKind::Folder => {
            for pb in event.paths {
                file::delete_by_parent(pb).await?;
            }
        }
        _ => warn!("unhandled remove event occurred: {:?}", event),
    };
    Ok(())
}

async fn process_event(event: notify::event::Event) -> Result<(), sqlx::Error> {
    match event.kind.clone() {
        notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
            for file_path in event.paths {
                let parent = file_path.parent().unwrap().to_string_lossy().into();
                let name = file_path.file_name().unwrap().to_string_lossy().into();
                let md = std::fs::metadata(file_path).unwrap();
                let f = file::Entry::from_metadata(md, parent, name);
                file::create_or_update(f).await?;
            }
        }
        notify::EventKind::Remove(rk) => process_remove_event(rk, event).await?,
        _ => warn!("unhandled event kind received: {:?}", event),
    };
    Ok(())
}

pub async fn start_notify(conf: &NiKoDirConfig<'_>) -> Result<(), sqlx::Error> {
    let (tx, mut rx) = channel(100);
    let mut watcher = notify::recommended_watcher(move |res| {
        tx.blocking_send(res).unwrap();
    })
    .expect("failed to initialize a recommended watcher.");
    watcher
        .watch(
            std::path::Path::new(conf.full_path()),
            notify::RecursiveMode::Recursive,
        )
        .expect("failed to start watching.");
    loop {
        match rx.recv().await {
            Some(result) => match result {
                Ok(event) => process_event(event).await?,
                Err(err) => error!("received an error as an event: {:?}", err),
            },
            None => info!("received nothing..."),
        }
    }
}
