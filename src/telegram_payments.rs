use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use crate::models::{Product, Database};
use actix_web::{web, HttpResponse, Responder};

#[derive(Serialize, Deserialize, Debug)]
struct TelegramInvoice {
    title: String,
    description: String,
    payload: String,
    provider_token: String,
    currency: String,
    prices: Vec<LabeledPrice>,
    chat_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    photo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    photo_size: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct LabeledPrice {
    label: String,
    amount: i32,
}

#[derive(Deserialize)]
pub struct CreatePaymentRequest {
    product_ids: Vec<i32>,
    chat_id: i64,
}

pub async fn create_telegram_payment(
    db: web::Data<Database>,
    payload: CreatePaymentRequest,
) -> Result<String, Box<dyn std::error::Error>> {
    let provider_token = env::var("TELEGRAM_PAYMENT_TOKEN")?;
    let bot_name = env::var("TELEGRAM_BOT_NAME")?;

    let products = Product::find_by_ids(&db.connection, &payload.product_ids)?;
    let total = products.iter().map(|p| p.price).sum::<f64>();

    let invoice = TelegramInvoice {
        title: format!("Покупка {} товаров", products.len()),
        description: products.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", "),
        payload: format!("products:{}", payload.product_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",")),
        provider_token,
        currency: "USD".to_string(),
        prices: vec![LabeledPrice {
            label: "Итого".to_string(),
            amount: (total * 100.0) as i32,
        }],
        chat_id: payload.chat_id,
        photo_url: Some("https://your-shop.com/images/logo.png".to_string()),
        photo_size: Some(512),
    };

    let payment_url = format!("https://t.me/{}?start=payment_{}", bot_name, uuid::Uuid::new_v4());
    // Здесь будет реальный вызов Telegram API, пока заглушка

    Ok(payment_url)
}

#[post("/api/telegram-payment")]
async fn create_payment_handler(
    db: web::Data<Database>,
    web::Json(payload): web::Json<CreatePaymentRequest>,
) -> impl Responder {
    match create_telegram_payment(db, payload).await {
        Ok(url) => HttpResponse::Ok().json(json!({ "payment_url": url })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}