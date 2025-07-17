use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_files::Files;
use actix_cors::Cors;
use actix_web::http::header;
use std::path::Path;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::cookie::Key;
use sqlx::{Pool, Sqlite};
use crate::telegram_notifications::TelegramNotifier;
use crate::ton_payment::TonProcessor;
use crate::telegram_bot::TelegramBot;
use std::sync::Arc;

mod api;
mod models;
mod database;
mod ton_payment;
mod telegram_notifications;
mod telegram_bot;

pub struct AppState {
    db_pool: Pool<Sqlite>,
    ton_processor: Arc<TonProcessor>,
    telegram_notifier: Arc<TelegramNotifier>,
}

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

    // Получаем токен бота и ID админ-чата
    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .expect("TELEGRAM_BOT_TOKEN must be set");

    let admin_chat_id = std::env::var("ADMIN_CHAT_ID")
        .expect("ADMIN_CHAT_ID must be set")
        .parse::<i64>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    // Инициализируем компоненты
    let telegram_notifier = Arc::new(TelegramNotifier::new(
        bot_token.clone(),
        admin_chat_id,
        pool.clone(),
    ));

    let ton_processor = Arc::new(TonProcessor::new(pool.clone())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?);

    // Создаем и запускаем Telegram бота в отдельном потоке
    let bot_notifier = telegram_notifier.clone();
    let bot_processor = ton_processor.clone();
    tokio::spawn(async move {
        let bot = TelegramBot::new(bot_token, bot_notifier, bot_processor);
        bot.start().await;
    });

    // Для production используйте фиксированный ключ из конфига!
    let secret_key = Key::generate();

    // Создаем AppState для веб-сервера
    let app_state = web::Data::new(AppState {
        db_pool: pool.clone(),
        ton_processor: ton_processor.clone(),
        telegram_notifier: telegram_notifier.clone(),
    });

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
            .app_data(app_state.clone())
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
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
