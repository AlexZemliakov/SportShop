use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files::Files;
use actix_cors::Cors;
use actix_web::http::header;
use std::path::Path;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;

mod api;
mod models;
mod database;

async fn serve_cart() -> impl Responder {
    match std::fs::read_to_string("frontend/public/cart.html") {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(content),
        Err(_) => HttpResponse::NotFound()
            .body("Cart page not found")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let public_dir = Path::new("frontend/public")
        .canonicalize()
        .expect("Failed to resolve public directory");

    println!("Serving files from: {}", public_dir.display());

    let pool = database::init_db().await
        .expect("Failed to initialize database");

    // Для production используйте фиксированный ключ из конфига!
    let secret_key = Key::generate();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .service(api::checkout)
            .wrap(cors)
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    secret_key.clone()
                )
                    .cookie_secure(false)  // Для разработки
                    .build()
            )
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::BadRequest().json("Invalid JSON"),
                ).into()
            }))
            .app_data(web::Data::new(pool.clone()))
            .configure(api::config)
            .service(
                web::resource("/cart")
                    .route(web::get().to(serve_cart))
            )
            .service(
                Files::new("/", &public_dir)
                    .index_file("index.html")
                    .show_files_listing()
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}