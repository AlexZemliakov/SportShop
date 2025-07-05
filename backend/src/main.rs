use actix_web::{web, App, HttpServer};
use sqlx::sqlite::SqlitePool;

mod api;
mod models;
mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let pool = SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Failed to connect to database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(api::config)
            // Раздаём статику из папки frontend
            .service(
                actix_files::Files::new("/", "../frontend/public")
                    .index_file("index.html"),
            )
            .service(
                actix_files::Files::new("/admin", "../frontend/public/admin")
                    .index_file("index.html"),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}