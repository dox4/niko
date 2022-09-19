use notify::event::{CreateKind, ModifyKind, RemoveKind};
use notify::Watcher;
use tracing::*;
use walkdir::WalkDir;

use crate::config::NiKoDirConfig;
use crate::model::file;
use std::sync::mpsc;

pub async fn may_need_scanning(conf: &NiKoDirConfig<'_>) -> Result<(), sqlx::Error> {
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
                debug!("create file {:?}", f);
                let id = file::create(f).await?;
                debug!("file record created.... {:?}", file::fetch_by_id(id).await?);
            }
            Err(err) => panic!("fetch record at error {:?}", err),
        }
    }
    Ok(())
}

async fn process_remove_event(
    rk: RemoveKind,
    event: notify::event::Event,
) -> Result<(), Box<dyn std::error::Error>> {
    match rk {
        notify::event::RemoveKind::File => todo!(),
        notify::event::RemoveKind::Folder => todo!(),
        _ => warn!("unhandled remove event occurred: {:?}", event),
        // notify::event::RemoveKind::Other => todo!(),
        // notify::event::RemoveKind::Any => todo!(),
    };
    Ok(())
}

async fn process_event(event: notify::event::Event) -> Result<(), Box<dyn std::error::Error>> {
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

pub async fn start_notify(conf: &NiKoDirConfig<'_>) -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(
        std::path::Path::new(conf.full_path()),
        notify::RecursiveMode::Recursive,
    )?;
    loop {
        match rx.recv() {
            Ok(result) => match result {
                Ok(event) => process_event(event).await?,
                Err(err) => error!("received an error as an event: {:?}", err),
            },
            Err(err) => error!("received an error while watching the path: {:?}", err),
        }
    }
}
