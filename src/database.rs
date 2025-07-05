use sqlx::{SqlitePool, Error};

pub async fn init_db() -> Result<SqlitePool, Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = SqlitePool::connect(&database_url).await?;
    run_migrations(&pool).await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await

}