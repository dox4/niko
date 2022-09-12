use once_cell::sync::OnceCell;
use sqlx::MySql;
use sqlx::Pool;

use crate::config::NiKoDbConfig;

static POOL: OnceCell<Pool<MySql>> = OnceCell::new();

pub async fn init_db(db: &NiKoDbConfig<'_>) {
    let url = db.db_url();
    POOL.set(Pool::<MySql>::connect(&url).await.unwrap())
        .unwrap();
}

pub fn global_pool() -> &'static Pool<MySql> {
    POOL.get().expect("get MySQL connection pool failed.")
}
