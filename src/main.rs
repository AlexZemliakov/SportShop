use actix_web::{web, App, HttpServer};
use actix_files::Files;
use actix_cors::Cors;
use std::path::Path;

mod api;
mod models;
mod services;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let public_dir = Path::new("frontend/public")
        .canonicalize()
        .expect("Failed to resolve public directory");

    let pool = services::database::init_db().await
        .expect("Failed to initialize database");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .configure(api::config)
            .service(
                Files::new("/", &public_dir)
                    .index_file("index.html")
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}