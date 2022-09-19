use serde_derive::Serialize;
use sqlx::types::chrono::{DateTime, Local};
use std::{fs::Metadata, os::unix::fs::PermissionsExt, path::Path};

use crate::db;

#[derive(Debug, Serialize, Default, Clone, sqlx::FromRow)]
pub struct Entry {
    pub id: i32,
    pub parent: String,
    pub name: String,
    pub is_dir: bool,
    pub size: i32,
    pub permission: i32,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub deleted_at: Option<DateTime<Local>>,
}

impl Entry {
    pub fn from_metadata(md: Metadata, parent: String, name: String) -> Self {
        Self {
            id: 0,
            parent,
            name,
            is_dir: md.is_dir(),
            size: md.len() as i32,
            permission: md.permissions().mode() as i32,
            created_at: md.created().unwrap().into(),
            updated_at: md.modified().unwrap().into(),
            deleted_at: None,
        }
    }

    pub fn full_path(&self) -> String {
        Path::new(self.parent.as_str())
            .join(self.name.clone())
            .to_string_lossy()
            .into()
    }
}

pub async fn paged_fetch(page_num: u64, page_size: u64) -> Result<Vec<Entry>, sqlx::Error> {
    let offset = (page_num - 1) * page_size;
    let limit = offset + page_size;
    let rows = sqlx::query_as::<_, Entry>("SELECT * FROM files where deleted_at IS NULL LIMIT ?, ?")
        .bind(offset)
        .bind(limit)
        .fetch_all(db::global_pool())
        .await?;
    Ok(rows)
}

pub enum CreateOrUpdate {
    Create(u64),
    Update(u64),
}

pub async fn create_or_update(mut file: Entry) -> Result<CreateOrUpdate, sqlx::Error> {
    let row = fetch_one_by_path(file.parent.clone(), file.name.clone()).await;
    match row {
        Ok(file_in_db) => {
            file.id = file_in_db.id;
            Ok(CreateOrUpdate::Update(update_by_id(file).await?))
        }
        Err(sqlx::Error::RowNotFound) => Ok(CreateOrUpdate::Create(create(file).await?)),
        Err(err) => {
            tracing::error!("error occurred while quering by path: {:?}", err);
            Err(err)
        }
    }
}

pub async fn create(file: Entry) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(
        r#"INSERT INTO `files`
        (`parent`, `name`, `is_dir`, `size`, `permission`, `created_at`, `updated_at`)
        VALUES (?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(file.parent)
    .bind(file.name)
    .bind(file.is_dir)
    .bind(file.size)
    .bind(file.permission)
    .bind(file.created_at)
    .bind(file.updated_at)
    .execute(db::global_pool())
    .await?
    .last_insert_id())
}

pub async fn fetch_by_id(id: u64) -> Result<Entry, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, Entry>("SELECT * FROM `files` WHERE `deleted_at` IS NULL AND `id` = ?")
            .bind(id)
            .fetch_one(db::global_pool())
            .await?,
    )
}

pub async fn fetch_one_by_path(parent: String, name: String) -> Result<Entry, sqlx::Error> {
    Ok(sqlx::query_as::<_, Entry>(
        "SELECT * FROM `file` WHERE `deleted_at` IS NULL AND `parent` = ? AND `name` = ?",
    )
    .bind(parent)
    .bind(name)
    .fetch_one(db::global_pool())
    .await?)
}

pub async fn update_by_id(f: Entry) -> Result<u64, sqlx::Error> {
    let upd_sql = r#"UPDATE `files` SET
        `parent` = ?,
        `name` = ?,
        `is_dir` = ?,
        `size` = ?,
        `permission` = ?,
        `created_at` = ?,
        `updated_at` = ?
    WHERE `id` = ?"#;
    Ok(sqlx::query(upd_sql)
        .bind(f.parent)
        .bind(f.name)
        .bind(f.is_dir)
        .bind(f.size)
        .bind(f.permission)
        .bind(f.created_at)
        .bind(f.updated_at)
        .bind(f.id)
        .execute(db::global_pool())
        .await?
        .rows_affected())
}

pub async fn delete_by_path(file: Entry) -> Result<u64, sqlx::Error> {
    let del_sql = r#"UPDATE `files` SET
    deleted_at = ?
    WHERE `parent` = ? AND `name` = ? AND deleted_at IS NULL"#;
    let now = Local::now();
    Ok(sqlx::query(del_sql)
        .bind(Some(now.clone()))
        .bind(file.parent)
        .bind(file.name)
        .execute(db::global_pool())
        .await?
        .rows_affected())
}

pub async fn delete_by_parent(entry: Entry) -> Result<u64, sqlx::Error> {
    let del_sql = r#"UPDATE `files` SET
    deleted_at = ?
    WHERE `parent` = ? AND deleted_at IS NULL"#;
    let now = Local::now();
    Ok(sqlx::query(del_sql)
        .bind(Some(now.clone()))
        .bind(entry.full_path())
        .execute(db::global_pool())
        .await?
        .rows_affected())
}
