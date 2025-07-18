use sqlx::{SqlitePool, migrate::MigrateError};

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Создаем папку database, если её нет
    std::fs::create_dir_all("database").ok();

    let database_url = "sqlite:database/shop.db";  // Изменили путь к базе

    let pool = SqlitePool::connect(&database_url).await?;

    // Проверяем существование таблиц
    if let Err(_e) = sqlx::query("SELECT 1 FROM sqlite_master LIMIT 1")
        .execute(&pool)
        .await
    {
        eprintln!("Database is empty, applying migrations...");
        run_migrations(&pool).await?;
    }

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
}
