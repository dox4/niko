use serde_derive::Serialize;
use sqlx::types::chrono::{DateTime, Local};

use crate::db;

#[derive(Debug, Serialize, Default, Clone, sqlx::FromRow)]
pub struct File {
    #[sqlx()]
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

pub async fn paged_fetch(page_num: u64, page_size: u64) -> Result<Vec<File>, sqlx::Error> {
    let offset = (page_num - 1) * page_size;
    let limit = offset + page_size;
    let rows = sqlx::query_as::<_, File>("SELECT * FROM files where deleted_at IS NULL LIMIT ?, ?")
        .bind(offset)
        .bind(limit)
        .fetch_all(db::global_pool())
        .await?;
    Ok(rows)
}

pub async fn create(file: File) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(
        r#"INSERT INTO files
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
    // .fetch_one(db::global_pool())
    .await?
    .last_insert_id())
}

pub async fn fetch_by_id(id: u64) -> Result<File, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, File>("SELECT * FROM `files` WHERE `id` = ?")
            .bind(id)
            .fetch_one(db::global_pool())
            .await?,
    )
}
