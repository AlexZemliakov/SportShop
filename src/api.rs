use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Product {
    id: i64,
    name: String,
    price: f64,
    stock: i32,
}

#[derive(Deserialize)]
pub struct CreateProduct {
    name: String,
    price: f64,
    stock: i32,
}

pub async fn list_products(pool: web::Data<sqlx::SqlitePool>) -> HttpResponse {
    let products = sqlx::query_as::<_, Product>("SELECT id, name, price, stock FROM products")
        .fetch_all(&**pool)
        .await
        .unwrap();

    HttpResponse::Ok().json(products)
}

pub async fn create_product(
    pool: web::Data<sqlx::SqlitePool>,
    product: web::Json<CreateProduct>,
) -> HttpResponse {
    sqlx::query!(
        "INSERT INTO products (name, price, stock) VALUES (?, ?, ?)",
        product.name,
        product.price,
        product.stock
    )
        .execute(&**pool)
        .await
        .unwrap();

    HttpResponse::Created().finish()
}

pub async fn delete_product(
    pool: web::Data<sqlx::SqlitePool>,
    id: web::Path<i64>,
) -> HttpResponse {
    sqlx::query!("DELETE FROM products WHERE id = ?", *id)
        .execute(&**pool)
        .await
        .unwrap();

    HttpResponse::NoContent().finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/products", web::get().to(list_products))
            .route("/products", web::post().to(create_product))
            .route("/products/{id}", web::delete().to(delete_product)),
    );
}