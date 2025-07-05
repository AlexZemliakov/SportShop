use actix_web::{web, App, HttpServer};
use actix_files::Files;
use std::path::Path;

mod api;
mod models;
mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Получаем абсолютный путь к frontend/public
    let public_dir = Path::new("frontend/public")
        .canonicalize()
        .expect("Failed to resolve public directory");

    println!("Serving files from: {}", public_dir.display());

    let pool = database::init_db().await
        .expect("Failed to initialize database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(api::config)
            .service(
                Files::new("/", &public_dir)
                    .index_file("index.html")
                    .show_files_listing(),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}