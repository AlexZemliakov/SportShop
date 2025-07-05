use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use std::env;

mod api;
mod models;
mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = database::init_db().await
        .expect("Failed to initialize database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(api::config)
            .service(
                actix_files::Files::new("/", "./frontend/public")
                    .index_file("index.html"),
            )
            .service(
                actix_files::Files::new("/admin", "./frontend/public/admin")
                    .index_file("index.html"),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}