use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i64>,
    #[serde(skip)]
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProduct {
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i32,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub category_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
}

// Products handlers
pub async fn list_products(pool: web::Data<sqlx::SqlitePool>) -> impl Responder {
    match sqlx::query_as::<_, Product>(
        r#"
        SELECT 
            id, 
            name,
            description,
            price, 
            stock, 
            image_url,
            category_id,
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
        INSERT INTO products (name, description, price, stock, image_url, category_id)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
        .bind(&product.name)
        .bind(&product.description)
        .bind(product.price)
        .bind(product.stock)
        .bind(&product.image_url)
        .bind(product.category_id)
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

pub async fn get_products_by_category(
    pool: web::Data<sqlx::SqlitePool>,
    category_id: web::Path<i64>,
) -> impl Responder {
    match sqlx::query_as::<_, Product>(
        r#"
        SELECT 
            id, 
            name,
            description,
            price, 
            stock, 
            image_url,
            category_id,
            strftime('%Y-%m-%d %H:%M:%S', created_at) as created_at
        FROM products
        WHERE category_id = ?
        "#
    )
        .bind(*category_id)
        .fetch_all(&**pool)
        .await
    {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => {
            eprintln!("Failed to fetch products by category: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch products by category")
        }
    }
}

// Categories handlers
pub async fn list_categories(pool: web::Data<sqlx::SqlitePool>) -> impl Responder {
    match sqlx::query_as::<_, Category>("SELECT * FROM categories")
        .fetch_all(&**pool)
        .await
    {
        Ok(categories) => HttpResponse::Ok().json(categories),
        Err(e) => {
            eprintln!("Failed to fetch categories: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch categories")
        }
    }
}

pub async fn create_category(
    pool: web::Data<sqlx::SqlitePool>,
    category: web::Json<CreateCategory>,
) -> impl Responder {
    match sqlx::query(
        "INSERT INTO categories (name, description, image_url) VALUES (?, ?, ?)"
    )
        .bind(&category.name)
        .bind(&category.description)
        .bind(&category.image_url)
        .execute(&**pool)
        .await
    {
        Ok(_) => HttpResponse::Created().json("Category created"),
        Err(e) => {
            eprintln!("Failed to create category: {}", e);
            HttpResponse::InternalServerError().json("Failed to create category")
        }
    }
}

pub async fn delete_category(
    pool: web::Data<sqlx::SqlitePool>,
    category_id: web::Path<i64>,
) -> impl Responder {
    // Check if category has products
    match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM products WHERE category_id = ?"
    )
        .bind(*category_id)
        .fetch_one(&**pool)
        .await
    {
        Ok(count) if count > 0 => {
            HttpResponse::BadRequest().json("Cannot delete category with products")
        },
        Ok(_) => {
            match sqlx::query("DELETE FROM categories WHERE id = ?")
                .bind(*category_id)
                .execute(&**pool)
                .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => {
                    eprintln!("Failed to delete category: {}", e);
                    HttpResponse::InternalServerError().json("Failed to delete category")
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to check category products: {}", e);
            HttpResponse::InternalServerError().json("Failed to check category products")
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Products routes
            .route("/products", web::get().to(list_products))
            .route("/products", web::post().to(create_product))
            .route("/products/{id}", web::delete().to(delete_product))
            // Categories routes
            .route("/categories", web::get().to(list_categories))
            .route("/categories", web::post().to(create_category))
            .route("/categories/{id}", web::delete().to(delete_category))
            .route("/categories/{id}/products", web::get().to(get_products_by_category)),
    );
}