use tracing::debug;
use walkdir::WalkDir;

use crate::config::NiKoDirConfig;
use crate::model::file;
use std::os::unix::fs::PermissionsExt;

pub async fn may_need_scanning(conf: &NiKoDirConfig<'_>) -> Result<(), sqlx::Error> {
    for entry in WalkDir::new(conf.full_path())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let parent = entry.path().parent().unwrap().display().to_string();
        let name: String = entry.file_name().to_string_lossy().into();
        match file::fetch_one_by_path(parent.clone(), name.clone()).await {
            Ok(fetched) => {
                let f = file::File {
                    id: fetched.id,
                    parent: parent,
                    name: name,
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
                let effected = file::update_by_id(f).await?;
                if effected != 1 {
                    panic!("update by id effected rows...")
                }
            }
            Err(sqlx::Error::RowNotFound) => {
                let f = file::File {
                    id: 0,
                    parent: parent,
                    name: name,
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
                debug!("create file {:?}", f);
                let id = file::create(f).await?;

                debug!("file record created.... {:?}", file::fetch_by_id(id).await?);
            }
            Err(err) => panic!("fetch record at error {:?}", err),
        }
    }
    Ok(())
}

pub fn start_notify(conf: &NiKoDirConfig) {}
