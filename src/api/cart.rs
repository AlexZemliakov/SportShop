use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::SqlitePool;
use crate::models::CartItem;

#[derive(Debug, Deserialize)]
pub struct AddToCartRequest {
    pub product_id: i64,
    pub quantity: i64,
    pub user_id: Option<i64>,
}

#[post("/add")]
pub async fn add_to_cart(
    pool: web::Data<SqlitePool>,
    item: web::Json<AddToCartRequest>,
) -> impl Responder {
    match sqlx::query_as!(
        CartItem,
        "INSERT INTO cart (product_id, quantity) VALUES (?, ?) RETURNING id, product_id, quantity, user_id",
        item.product_id,
        item.quantity
    )
        .fetch_one(&**pool)
        .await {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[get("/items")]
pub async fn get_cart_items(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query_as!(
        CartItem,
        "SELECT id, product_id, quantity, user_id FROM cart"
    )
        .fetch_all(&**pool)
        .await {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/cart")
            .service(add_to_cart)
            .service(get_cart_items)
    );
}