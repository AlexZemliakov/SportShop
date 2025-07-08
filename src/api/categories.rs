use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use crate::models::Category;

#[derive(Debug, Serialize, FromRow)]
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[get("")]
pub async fn list_categories(pool: web::Data<SqlitePool>) -> impl Responder {
    match sqlx::query_as!(CategoryResponse,
        "SELECT id, name, description, image_url FROM categories"
    )
        .fetch_all(&**pool)
        .await {
        Ok(categories) => HttpResponse::Ok().json(categories),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

#[post("")]
pub async fn create_category(
    pool: web::Data<SqlitePool>,
    category: web::Json<CreateCategory>,
) -> impl Responder {
    match sqlx::query!(
        "INSERT INTO categories (name, description, image_url) VALUES (?, ?, ?)",
        category.name,
        category.description,
        category.image_url
    )
        .execute(&**pool)
        .await {
        Ok(_) => HttpResponse::Created().json("Category created"),
        Err(e) => HttpResponse::InternalServerError().json(e.to_string()),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/categories")
            .service(list_categories)
            .service(create_category)
    );
}