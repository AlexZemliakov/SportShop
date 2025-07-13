use std::collections::HashMap;
use actix_web::{get, post, web, HttpResponse, Responder, delete, put};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use actix_session::Session;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
#[allow(dead_code)]
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

// Cart models
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CartItem {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CartItemWithProduct {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub name: String,
    pub price: f64,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CartItemRequest {
    pub product_id: i64,
    pub quantity: i32,
}

// Products handlers
pub async fn list_products(pool: web::Data<SqlitePool>) -> impl Responder {
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

#[get("/product/{id}")]
pub async fn get_product(
    pool: web::Data<SqlitePool>,
    product_id: web::Path<i64>,
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
        WHERE id = ?
        "#
    )
        .bind(product_id.into_inner())
        .fetch_one(&**pool)
        .await
    {
        Ok(product) => HttpResponse::Ok().json(product),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json("Product not found"),
        Err(e) => {
            eprintln!("Failed to fetch product: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch product")
        }
    }
}

pub async fn create_product(
    pool: web::Data<SqlitePool>,
    product: web::Json<CreateProduct>,
) -> impl Responder {
    println!("Received product: {:?}", product);

    if let Some(category_id) = product.category_id {
        let category_exists: bool = match sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = ?)"
        )
            .bind(category_id)
            .fetch_one(&**pool)
            .await
        {
            Ok(exists) => exists,
            Err(e) => {
                eprintln!("Failed to check category existence: {}", e);
                return HttpResponse::BadRequest().json("Failed to check category");
            }
        };

        if !category_exists {
            return HttpResponse::BadRequest().json("Category does not exist");
        }
    }

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
            HttpResponse::InternalServerError().json(format!("Failed to create product: {}", e))
        }
    }
}

#[delete("/products/{id}")]
pub async fn delete_product(
    pool: web::Data<SqlitePool>,
    product_id: web::Path<i64>,
) -> impl Responder {
    let id = product_id.into_inner();

    // Сначала удаляем связанные записи из корзины
    match sqlx::query("DELETE FROM cart WHERE product_id = ?")
        .bind(id)
        .execute(&**pool)
        .await
    {
        Ok(_) => println!("DEBUG: Removed cart items for product {}", id),
        Err(e) => {
            eprintln!("DEBUG: Failed to remove cart items: {}", e);
            return HttpResponse::InternalServerError()
                .json(format!("Failed to remove cart items: {}", e));
        }
    }

    // Затем удаляем сам товар
    match sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(id)
        .execute(&**pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json("Product not found"),
        Err(e) => {
            eprintln!("DEBUG: Product delete error: {}", e);
            HttpResponse::InternalServerError().json(format!("Delete error: {}", e))
        }
    }
}
pub async fn get_products_by_category(
    pool: web::Data<SqlitePool>,
    category_id: web::Path<i64>,
) -> impl Responder {
    let cat_id = category_id.into_inner();

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
        .bind(cat_id)
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
pub async fn list_categories(pool: web::Data<SqlitePool>) -> impl Responder {
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
    pool: web::Data<SqlitePool>,
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
    pool: web::Data<SqlitePool>,
    category_id: web::Path<i64>,
) -> impl Responder {
    let id = category_id.into_inner();

    // Check if category has products
    match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM products WHERE category_id = ?"
    )
        .bind(id)
        .fetch_one(&**pool)
        .await
    {
        Ok(count) if count > 0 => {
            HttpResponse::BadRequest().json("Cannot delete category with products")
        },
        Ok(_) => {
            match sqlx::query("DELETE FROM categories WHERE id = ?")
                .bind(id)
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

// Cart handlers
#[post("/cart")]
pub async fn add_to_cart(
    pool: web::Data<SqlitePool>,
    item: web::Json<CartItemRequest>,
    session: Session,
) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        Ok(None) => {
            let new_id = uuid::Uuid::new_v4().to_string();
            session.insert("session_id", &new_id).unwrap();
            new_id
        }
        Err(e) => {
            eprintln!("Failed to get session: {}", e);
            return HttpResponse::InternalServerError().json("Session error");
        }
    };

    // Check if product exists
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM products WHERE id = ?")
        .bind(item.product_id)
        .fetch_one(&**pool)
        .await
    {
        Ok(count) if count == 0 => {
            return HttpResponse::NotFound().json("Product not found");
        }
        Err(e) => {
            eprintln!("Failed to check product: {}", e);
            return HttpResponse::InternalServerError().json("Failed to check product");
        }
        _ => {}
    };

    // Check if item already in cart
    match sqlx::query_as::<_, CartItem>(
        "SELECT * FROM cart WHERE product_id = ? AND session_id = ?"
    )
        .bind(item.product_id)
        .bind(&session_id)
        .fetch_optional(&**pool)
        .await
    {
        Ok(Some(existing_item)) => {
            // Update quantity if already exists
            let new_quantity = existing_item.quantity + item.quantity;
            match sqlx::query(
                "UPDATE cart SET quantity = ? WHERE id = ?"
            )
                .bind(new_quantity)
                .bind(existing_item.id)
                .execute(&**pool)
                .await
            {
                Ok(_) => HttpResponse::Ok().json("Cart updated"),
                Err(e) => {
                    eprintln!("Failed to update cart: {}", e);
                    HttpResponse::InternalServerError().json("Failed to update cart")
                }
            }
        }
        Ok(None) => {
            // Add new item to cart
            match sqlx::query(
                "INSERT INTO cart (product_id, quantity, session_id) VALUES (?, ?, ?)"
            )
                .bind(item.product_id)
                .bind(item.quantity)
                .bind(&session_id)
                .execute(&**pool)
                .await
            {
                Ok(_) => HttpResponse::Created().json("Item added to cart"),
                Err(e) => {
                    eprintln!("Failed to add to cart: {}", e);
                    HttpResponse::InternalServerError().json("Failed to add to cart")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to check cart: {}", e);
            HttpResponse::InternalServerError().json("Failed to check cart")
        }
    }
}

#[get("/cart")]
pub async fn get_cart(
    pool: web::Data<SqlitePool>,
    session: Session,
) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Ok().json(Vec::<CartItemWithProduct>::new()),
    };

    match sqlx::query_as::<_, CartItemWithProduct>(
        r#"
        SELECT c.id, c.product_id, c.quantity, p.name, p.price, p.image_url
        FROM cart c
        JOIN products p ON c.product_id = p.id
        WHERE c.session_id = ?
        "#
    )
        .bind(&session_id)
        .fetch_all(&**pool)
        .await
    {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(e) => {
            eprintln!("Failed to fetch cart: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch cart")
        }
    }
}

// Обновление количества товара в корзине
#[put("/cart/{id}")]
pub async fn update_cart_item(
    pool: web::Data<SqlitePool>,
    item_id: web::Path<i64>,
    quantity: web::Json<HashMap<String, i32>>,  // Изменили тип параметра
    session: Session,
) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().json("Session required"),
    };

    // Получаем quantity из JSON
    let new_quantity = quantity.get("quantity").cloned().unwrap_or(1);

    match sqlx::query(
        "UPDATE cart SET quantity = ? WHERE id = ? AND session_id = ?"
    )
        .bind(new_quantity)
        .bind(item_id.into_inner())
        .bind(session_id)
        .execute(&**pool)
        .await {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().json("Quantity updated"),
        Ok(_) => HttpResponse::NotFound().json("Item not found"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e))
    }
}

// Удаление товара из корзины
#[delete("/cart/{id}")]
pub async fn remove_cart_item(
    pool: web::Data<SqlitePool>,
    item_id: web::Path<i64>,
    session: Session,
) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().json("Session required"),
    };

    match sqlx::query(
        "DELETE FROM cart WHERE id = ? AND session_id = ?"
    )
        .bind(item_id.into_inner())
        .bind(session_id)
        .execute(&**pool)
        .await {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json("Item not found"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e))
    }
}


#[get("/cart/count")]
pub async fn get_cart_count(
    pool: web::Data<SqlitePool>,
    session: Session,
) -> impl Responder {
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Ok().json(0),
    };

    match sqlx::query_scalar::<_, Option<i64>>(
        "SELECT SUM(quantity) FROM cart WHERE session_id = ?"
    )
        .bind(&session_id)
        .fetch_one(&**pool)
        .await
    {
        Ok(Some(count)) => HttpResponse::Ok().json(count),
        Ok(None) => HttpResponse::Ok().json(0),  // Если SUM вернул NULL (корзина пуста)
        Err(e) => {
            eprintln!("Failed to get cart count: {}", e);
            HttpResponse::InternalServerError().json("Failed to get cart count")
        }
    }
}
////////////////////////////////////////////
// Обновление количества товара в корзин


// Конфигурация маршрутов
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Products routes
            .service(get_product)
            .route("/products", web::get().to(list_products))
            .route("/products", web::post().to(create_product))
            .service(delete_product)
            .route("/categories/{id}/products", web::get().to(get_products_by_category))
            // Categories routes
            .route("/categories", web::get().to(list_categories))
            .route("/categories", web::post().to(create_category))
            .route("/categories/{id}", web::delete().to(delete_category))
            // Cart routes
            .service(add_to_cart)
            .service(get_cart)
            .service(remove_cart_item)  // Исправлено с remove_from_cart на remove_cart_item
            .service(get_cart_count)
            .service(update_cart_item)
    );
}