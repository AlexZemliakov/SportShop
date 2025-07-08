use sqlx::{SqlitePool, migrate::MigrateError};

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    std::fs::create_dir_all("database").ok();
    let database_url = "sqlite:database/shop.db";
    let pool = SqlitePool::connect(&database_url).await?;

    if let Err(_) = sqlx::query("SELECT 1 FROM sqlite_master LIMIT 1")
        .execute(&pool)
        .await
    {
        run_migrations(&pool).await?;
    }

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
}