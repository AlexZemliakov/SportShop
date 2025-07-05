use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub stock: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip)]
    pub created_at: Option<String>, // Изменено на String вместо NaiveDateTime
}

#[derive(Debug, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub price: f64,
    pub stock: i32,
    #[serde(default)]
    pub image_url: Option<String>,
}

pub async fn list_products(pool: web::Data<sqlx::SqlitePool>) -> impl Responder {
    match sqlx::query_as::<_, Product>(
        r#"
        SELECT 
            id, 
            name, 
            price, 
            stock, 
            image_url,
            strftime('%Y-%m-%d %H:%M:%S', created_at) as created_at
        FROM products
        "#
    )
        .fetch_all(&**pool)
        .await
    {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => {
            eprintln!("Failed to fetch products: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch products")
        }
    }
}

pub async fn create_product(
    pool: web::Data<sqlx::SqlitePool>,
    product: web::Json<CreateProduct>,
) -> impl Responder {
    match sqlx::query(
        r#"
        INSERT INTO products (name, price, stock, image_url)
        VALUES (?, ?, ?, ?)
        "#
    )
        .bind(&product.name)
        .bind(product.price)
        .bind(product.stock)
        .bind(&product.image_url)
        .execute(&**pool)
        .await
    {
        Ok(_) => HttpResponse::Created().json("Product created"),
        Err(e) => {
            eprintln!("Failed to create product: {}", e);
            HttpResponse::InternalServerError().json("Failed to create product")
        }
    }
}

pub async fn delete_product(
    pool: web::Data<sqlx::SqlitePool>,
    product_id: web::Path<i64>,
) -> impl Responder {
    match sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(*product_id)
        .execute(&**pool)
        .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            eprintln!("Failed to delete product: {}", e);
            HttpResponse::InternalServerError().json("Failed to delete product")
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/products", web::get().to(list_products))
            .route("/products", web::post().to(create_product))
            .route("/products/{id}", web::delete().to(delete_product)),
    );
}