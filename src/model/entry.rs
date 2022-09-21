use serde_derive::Serialize;
use sqlx::types::chrono::{DateTime, Local};
use std::{fs::Metadata, os::unix::fs::PermissionsExt, path::PathBuf};

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
}

pub async fn paged_fetch(page_num: u64, page_size: u64) -> Result<Vec<Entry>, sqlx::Error> {
    let offset = (page_num - 1) * page_size;
    let limit = offset + page_size;
    let rows =
        sqlx::query_as::<_, Entry>("SELECT * FROM `entry` where deleted_at IS NULL LIMIT ?, ?")
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

pub async fn create_or_update(mut entry: Entry) -> Result<CreateOrUpdate, sqlx::Error> {
    let row = fetch_one_by_path(entry.parent.clone(), entry.name.clone()).await;
    match row {
        Ok(entry_in_db) => {
            entry.id = entry_in_db.id;
            Ok(CreateOrUpdate::Update(update_by_id(entry).await?))
        }
        Err(sqlx::Error::RowNotFound) => Ok(CreateOrUpdate::Create(create(entry).await?)),
        Err(err) => {
            tracing::error!("error occurred while quering by path: {:?}", err);
            Err(err)
        }
    }
}

pub async fn create(entry: Entry) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(
        r#"INSERT INTO `entry`
        (`parent`, `name`, `is_dir`, `size`, `permission`, `created_at`, `updated_at`)
        VALUES (?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(entry.parent)
    .bind(entry.name)
    .bind(entry.is_dir)
    .bind(entry.size)
    .bind(entry.permission)
    .bind(entry.created_at)
    .bind(entry.updated_at)
    .execute(db::global_pool())
    .await?
    .last_insert_id())
}

pub async fn fetch_by_id(id: u64) -> Result<Entry, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, Entry>("SELECT * FROM `entry` WHERE `deleted_at` IS NULL AND `id` = ?")
            .bind(id)
            .fetch_one(db::global_pool())
            .await?,
    )
}

pub async fn fetch_one_by_path(parent: String, name: String) -> Result<Entry, sqlx::Error> {
    Ok(sqlx::query_as::<_, Entry>(
        "SELECT * FROM `entry` WHERE `deleted_at` IS NULL AND `parent` = ? AND `name` = ?",
    )
    .bind(parent)
    .bind(name)
    .fetch_one(db::global_pool())
    .await?)
}

pub async fn update_by_id(e: Entry) -> Result<u64, sqlx::Error> {
    let upd_sql = r#"UPDATE `entry` SET
        `parent` = ?,
        `name` = ?,
        `is_dir` = ?,
        `size` = ?,
        `permission` = ?,
        `created_at` = ?,
        `updated_at` = ?
    WHERE `id` = ?"#;
    Ok(sqlx::query(upd_sql)
        .bind(e.parent)
        .bind(e.name)
        .bind(e.is_dir)
        .bind(e.size)
        .bind(e.permission)
        .bind(e.created_at)
        .bind(e.updated_at)
        .bind(e.id)
        .execute(db::global_pool())
        .await?
        .rows_affected())
}

pub async fn delete_by_path(pb: PathBuf) -> Result<u64, sqlx::Error> {
    let del_sql = r#"UPDATE `entry` SET
    deleted_at = ?
    WHERE `parent` = ? AND `name` = ? AND deleted_at IS NULL"#;
    let now = Local::now();
    Ok(sqlx::query(del_sql)
        .bind(Some(now))
        .bind(pb.parent().unwrap().to_str())
        .bind(pb.file_name().unwrap().to_str())
        .execute(db::global_pool())
        .await?
        .rows_affected())
}

pub async fn delete_by_parent(pb: PathBuf) -> Result<u64, sqlx::Error> {
    let del_sql = r#"UPDATE `entry` SET
    deleted_at = ?
    WHERE `parent` = ? AND deleted_at IS NULL"#;
    let now = Local::now();
    Ok(sqlx::query(del_sql)
        .bind(Some(now.clone()))
        .bind(pb.as_path().to_str())
        .execute(db::global_pool())
        .await?
        .rows_affected())
}
