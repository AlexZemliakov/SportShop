use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use crate::models::Product;

#[derive(Debug, Serialize, FromRow)]
pub struct ProductResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i64,
    pub image_url: Option<String>,
    pub category_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i64,
    pub image_url: Option<String>,
    pub category_id: Option<i64>,
}

#[get("")]
pub async fn list_products(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query_as!(ProductResponse,
        "SELECT id, name, description, price, stock, image_url, category_id FROM products"
    )
        .fetch_all(&**pool)
        .await {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("")]
pub async fn create_product(
    pool: web::Data<SqlitePool>,
    product: web::Json<CreateProduct>,
) -> impl Responder {
    match sqlx::query!(
        "INSERT INTO products (name, description, price, stock, image_url, category_id) VALUES (?, ?, ?, ?, ?, ?)",
        product.name,
        product.description,
        product.price,
        product.stock,
        product.image_url,
        product.category_id
    )
        .execute(&**pool)
        .await {
        Ok(_) => HttpResponse::Created().json("Product created"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/products")
            .service(list_products)
            .service(create_product)
    );
}