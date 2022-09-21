use once_cell::sync::OnceCell;
use sqlx::MySql;
use sqlx::Pool;

use crate::config::global_config;

static POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub async fn init_db() {
    let url = global_config().db().db_url();
    POOL.set(Pool::<MySql>::connect(&url).await.unwrap())
        .unwrap();
}

pub fn global_pool() -> &'static Pool<MySql> {
    POOL.get().expect("get MySQL connection pool failed.")
}
