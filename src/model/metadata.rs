use serde_derive::Serialize;
use sqlx::types::chrono::{DateTime, Local};

use crate::db;

pub const KEY_WALKING_DIR: &str = "WALKING_DIR";

#[derive(Debug, Serialize, Default, Clone, sqlx::FromRow)]
pub struct Metadata {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Local>,
}

pub async fn find_key_updated_at(key: String) -> Result<DateTime<Local>, sqlx::Error> {
    Ok(
        sqlx::query_as::<_, Metadata>("SELECT * FROM metadata where `key` = ?")
            .bind(key)
            .fetch_one(db::global_pool())
            .await?
            .updated_at,
    )
}

pub async fn create_key(key: String, value: String) -> Result<DateTime<Local>, sqlx::Error> {
    sqlx::query("INSERT INTO metadata VALUES (?, ?, ?)")
        .bind(key.clone())
        .bind(value)
        .bind(Local::now())
        .execute(db::global_pool())
        .await?;
    Ok(find_key_updated_at(key).await?)
}

pub async fn update_key(key: String, value: String) -> Result<DateTime<Local>, sqlx::Error> {
    Ok(sqlx::query_as::<_, Metadata>(
        "UPDATE metadata SET `value` = ?, `updated_at` = ? WHERE `key` = ?",
    )
    .bind(value)
    .bind(Local::now())
    .bind(key)
    .fetch_one(db::global_pool())
    .await?
    .updated_at)
}
