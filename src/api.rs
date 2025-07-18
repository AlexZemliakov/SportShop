use std::collections::HashMap;
use actix_web::{get, post, web, HttpResponse, Responder, delete, put};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool, Row};
use actix_session::Session;
use serde_json::json;
use uuid::Uuid;
use crate::AppState;
use crate::models::Product; // Переиспользуем Product из models.rs, чтобы не было рассинхрона структур

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

#[derive(Debug, Deserialize)]
pub struct UpdateCategory {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub image_url: Option<Option<String>>,
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
#[get("/products")]
#[doc = "// Получение списка продуктов"]
pub async fn list_products(
    state: web::Data<AppState>,
) -> impl Responder {
    let pool = &state.db_pool;
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
        .fetch_all(pool)
        .await
    {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => {
            eprintln!("Failed to fetch products: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch products")
        }
    }
}

// Обработчик для получения продукта по ID
#[get("/product/{id}")]
pub async fn get_product_handler(
    state: web::Data<AppState>,
    product_id: web::Path<i64>,
) -> impl Responder {
    let pool = &state.db_pool;
    let product_id = product_id.into_inner();
    println!("Получен запрос на продукт с ID: {}", product_id);

    // Проверяем существование таблицы products
    match sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='products'"
    )
        .fetch_one(pool)
        .await {
        Ok(count) if count > 0 => {
            println!("✅ Таблица products существует");
        },
        Ok(_) => {
            eprintln!("❌ Таблица products не найдена в базе данных");
            return HttpResponse::NotFound().json(json!({
                "error": "Таблица products не найдена в базе данных"
            }));
        },
        Err(e) => {
            eprintln!("❌ Ошибка при проверке существования таблицы products: {}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Ошибка при проверке существования таблицы products",
                "details": e.to_string()
            }));
        }
    }

    // Пытаемся получить продукт из базы данных
    println!("Выполняем запрос к базе данных для продукта с ID: {}", product_id);

    match sqlx::query_as::<_, Product>(
        r#"
        SELECT id, name, price, stock, description, image_url, category_id, created_at
        FROM products
        WHERE id = ?
        "#
    )
        .bind(product_id)
        .fetch_optional(pool)
        .await {
        Ok(Some(product)) => {
            println!("✅ Успешно получен продукт: {:?}", product);
            HttpResponse::Ok().json(product)
        },
        Ok(None) => {
            println!("⚠️ Продукт с ID {} не найден", product_id);
            HttpResponse::NotFound().json(json!({
                "error": format!("Продукт с ID {} не найден", product_id)
            }))
        },
        Err(e) => {
            eprintln!("❌ Ошибка при выполнении запроса: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Ошибка базы данных",
                "details": e.to_string(),
                "product_id": product_id
            }))
        }
    }
}

#[post("/products")]
#[doc = "// Создание нового продукта"]
pub async fn create_product(
    state: web::Data<AppState>,
    product: web::Json<CreateProduct>,
) -> impl Responder {
    let pool = &state.db_pool;
    println!("Received product: {:?}", product);

    if let Some(category_id) = product.category_id {
        let category_exists: bool = match sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = ?)"
        )
            .bind(category_id)
            .fetch_one(pool)
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
        .execute(pool)
        .await
    {
        Ok(_) => HttpResponse::Created().json("Product created"),
        Err(e) => {
            eprintln!("Failed to create product: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to create product: {}", e))
        }
    }
}

#[put("/products/{id}")]
#[doc = "// Обновление продукта"]
pub async fn update_product(
    state: web::Data<AppState>,
    product_id: web::Path<i64>,
    product: web::Json<CreateProduct>,
) -> impl Responder {
    let pool = &state.db_pool;
    let id = product_id.into_inner();
    println!("Updating product {}: {:?}", id, product);

    // Check if product exists
    let exists: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM products WHERE id = ?)"
    )
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(exists) => exists,
        Err(e) => {
            eprintln!("Failed to check product existence: {}", e);
            return HttpResponse::InternalServerError().json("Failed to check product");
        }
    };

    if !exists {
        return HttpResponse::NotFound().json("Product not found");
    }

    // Check if category exists (if provided)
    if let Some(category_id) = product.category_id {
        let category_exists: bool = match sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM categories WHERE id = ?)"
        )
            .bind(category_id)
            .fetch_one(pool)
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

    // Update product
    match sqlx::query(
        r#"
        UPDATE products
        SET name = ?, description = ?, price = ?, stock = ?, image_url = ?, category_id = ?
        WHERE id = ?
        "#
    )
        .bind(&product.name)
        .bind(&product.description)
        .bind(product.price)
        .bind(product.stock)
        .bind(&product.image_url)
        .bind(product.category_id)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Product updated"),
        Err(e) => {
            eprintln!("Failed to update product: {}", e);
            HttpResponse::InternalServerError().json(format!("Failed to update product: {}", e))
        }
    }
}

#[delete("/products/{id}")]
#[doc = "// Удаление продукта"]
pub async fn delete_product(
    state: web::Data<AppState>,
    product_id: web::Path<i64>,
) -> impl Responder {
    let pool = &state.db_pool;
    let id = product_id.into_inner();

    // Сначала удаляем связанные записи из корзины
    match sqlx::query("DELETE FROM cart WHERE product_id = ?")
        .bind(id)
        .execute(pool)
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
        .execute(pool)
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
    state: web::Data<AppState>,
    category_id: web::Path<i64>,
) -> impl Responder {
    let pool = &state.db_pool;
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
        .fetch_all(pool)
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
#[get("/categories")]
#[doc = "// Получение списка категорий"]
pub async fn list_categories(
    state: web::Data<AppState>,
) -> impl Responder {
    let pool = &state.db_pool;
    match sqlx::query_as::<_, Category>("SELECT * FROM categories")
        .fetch_all(pool)
        .await
    {
        Ok(categories) => HttpResponse::Ok().json(categories),
        Err(e) => {
            eprintln!("Failed to fetch categories: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch categories")
        }
    }
}

#[get("/categories/{id}")]
#[doc = "// Получение категории по ID"]
pub async fn get_category(
    state: web::Data<AppState>,
    category_id: web::Path<i64>,
) -> impl Responder {
    let pool = &state.db_pool;
    match sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE id = ?")
        .bind(category_id.into_inner())
        .fetch_one(pool)
        .await
    {
        Ok(category) => HttpResponse::Ok().json(category),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().json("Category not found"),
        Err(e) => {
            eprintln!("Failed to fetch category: {}", e);
            HttpResponse::InternalServerError().json("Failed to fetch category")
        }
    }
}

#[post("/categories")]
#[doc = "// Создание новой категории"]
pub async fn create_category(
    state: web::Data<AppState>,
    category: web::Json<CreateCategory>,
) -> impl Responder {
    let pool = &state.db_pool;
    match sqlx::query(
        "INSERT INTO categories (name, description, image_url) VALUES (?, ?, ?)"
    )
        .bind(&category.name)
        .bind(&category.description)
        .bind(&category.image_url)
        .execute(pool)
        .await
    {
        Ok(_) => HttpResponse::Created().json("Category created"),
        Err(e) => {
            eprintln!("Failed to create category: {}", e);
            HttpResponse::InternalServerError().json("Failed to create category")
        }
    }
}

#[put("/categories/{id}")]
#[doc = "// Обновление категории"]
pub async fn update_category(
    state: web::Data<AppState>,
    category_id: web::Path<i64>,
    category: web::Json<UpdateCategory>,
) -> impl Responder {
    let pool = &state.db_pool;
    let id = category_id.into_inner();

    // Check if category exists
    let exists: bool = match sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM categories WHERE id = ?)"
    )
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(exists) => exists,
        Err(e) => {
            eprintln!("Failed to check category existence: {}", e);
            return HttpResponse::InternalServerError().json("Failed to check category");
        }
    };

    if !exists {
        return HttpResponse::NotFound().json("Category not found");
    }

    // Update category
    match sqlx::query(
        "UPDATE categories SET name = ?, description = ?, image_url = ? WHERE id = ?"
    )
        .bind(&category.name)
        .bind(&category.description)
        .bind(&category.image_url)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Category updated"),
        Err(e) => {
            eprintln!("Failed to update category: {}", e);
            HttpResponse::InternalServerError().json("Failed to update category")
        }
    }
}

#[delete("/categories/{id}")]
#[doc = "// Удаление категории"]
pub async fn delete_category(
    state: web::Data<AppState>,
    category_id: web::Path<i64>,
) -> impl Responder {
    let pool = &state.db_pool;
    let id = category_id.into_inner();

    // Check if category has products
    match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM products WHERE category_id = ?"
    )
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(count) if count > 0 => {
            return HttpResponse::BadRequest()
                .json("Cannot delete category with existing products");
        },
        Ok(_) => {
            // No products, safe to delete
            if let Err(e) = sqlx::query("DELETE FROM categories WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
            {
                eprintln!("Failed to delete category: {}", e);
                return HttpResponse::InternalServerError().json("Failed to delete category");
            }
            HttpResponse::NoContent().finish()
        },
        Err(e) => {
            eprintln!("Failed to check category products: {}", e);
            HttpResponse::InternalServerError().json("Failed to check category products")
        }
    }
}

#[post("/cart")]
pub async fn add_to_cart(
    state: web::Data<AppState>,
    item: web::Json<CartItemRequest>,
    session: Session,
) -> impl Responder {
    let pool = &state.db_pool;
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

    // Проверяем существование продукта
    let product_id = item.product_id;
    match sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM products WHERE id = ?")
        .bind(product_id)
        .fetch_one(pool)
        .await {
        Ok(count) if count == 0 => {
            return HttpResponse::NotFound().json("Product not found");
        },
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
        .fetch_optional(pool)
        .await {
        Ok(Some(existing_item)) => {
            // Update quantity if already exists
            let new_quantity = existing_item.quantity + item.quantity;
            match sqlx::query(
                "UPDATE cart SET quantity = ? WHERE id = ?"
            )
                .bind(new_quantity)
                .bind(existing_item.id)
                .execute(pool)
                .await {
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
                .execute(pool)
                .await {
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
    state: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    let pool = &state.db_pool;
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
        .fetch_all(pool)
        .await {
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
    state: web::Data<AppState>,
    item_id: web::Path<i64>,
    quantity: web::Json<HashMap<String, i32>>,  // Изменили тип параметра
    session: Session,
) -> impl Responder {
    let pool = &state.db_pool;
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
        .execute(pool)
        .await {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::Ok().json("Quantity updated"),
        Ok(_) => HttpResponse::NotFound().json("Item not found"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e))
    }
}

// Удаление товара из корзины
#[delete("/cart/{id}")]
pub async fn remove_cart_item(
    state: web::Data<AppState>,
    item_id: web::Path<i64>,
    session: Session,
) -> impl Responder {
    let pool = &state.db_pool;
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().json("Session required"),
    };

    match sqlx::query(
        "DELETE FROM cart WHERE id = ? AND session_id = ?"
    )
        .bind(item_id.into_inner())
        .bind(session_id)
        .execute(pool)
        .await {
        Ok(result) if result.rows_affected() > 0 => HttpResponse::NoContent().finish(),
        Ok(_) => HttpResponse::NotFound().json("Item not found"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e))
    }
}


pub async fn get_cart_count(
    state: web::Data<AppState>,
    session: Session,
) -> impl Responder {
    let pool = &state.db_pool;
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Ok().json(0),
    };

    match sqlx::query_scalar::<_, Option<i64>>(
        "SELECT SUM(quantity) FROM cart WHERE session_id = ?"
    )
        .bind(&session_id)
        .fetch_one(pool)
        .await {
        Ok(Some(count)) => HttpResponse::Ok().json(count),
        Ok(None) => HttpResponse::Ok().json(0),  // Если SUM вернул NULL (корзина пуста)
        Err(e) => {
            eprintln!("Failed to get cart count: {}", e);
            HttpResponse::InternalServerError().json("Failed to get cart count")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderRequest {
    pub user_id: i64,
    pub delivery_address: String,
    pub telegram_username: Option<String>,
}

#[post("/orders")]
pub async fn create_order(
    state: web::Data<AppState>,
    order_data: web::Json<OrderRequest>,
    session: Session
) -> Result<HttpResponse, actix_web::Error> {
    println!("=== Получен запрос на создание заказа ===");
    println!("Order data: user_id={}, delivery_address={:?}, telegram_username={:?}", 
             order_data.user_id, order_data.delivery_address, order_data.telegram_username);
    
    let pool = &state.db_pool;

    // Получаем ID сессии
    let session_id = match session.get::<String>("session_id") {
        Ok(Some(id)) => {
            println!("Найден существующий session_id: {}", id);
            id
        },
        _ => {
            // Генерируем новый ID сессии, если его нет
            let new_id = Uuid::new_v4().to_string();
            println!("Создан новый session_id: {}", new_id);
            session.insert("session_id", new_id.clone())?;
            new_id
        }
    };

    // Получаем товары из корзины
    println!("Ищем товары в корзине для session_id: {}", session_id);
    let cart_items = sqlx::query(
        r#"
        SELECT c.product_id, c.quantity, p.name, p.price
        FROM cart c
        JOIN products p ON c.product_id = p.id
        WHERE c.session_id = ?
        "#
    )
    .bind(&session_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        eprintln!("Ошибка получения корзины: {:?}", e);
        actix_web::error::ErrorInternalServerError("Ошибка получения корзины")
    })?;

    println!("Найдено товаров в корзине: {}", cart_items.len());

    if cart_items.is_empty() {
        println!("ОШИБКА: Корзина пуста для session_id: {}", session_id);
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Корзина пуста. Добавьте товары в корзину перед оформлением заказа."
        })));
    }

    // Вычисляем общую сумму
    let total_amount: f64 = cart_items.iter()
        .map(|item| {
            let price: f64 = item.get("price");
            let quantity: i32 = item.get("quantity");
            price * quantity as f64
        })
        .sum();

    // Создаем новый заказ
    let order_id = sqlx::query(
        r#"
        INSERT INTO orders (user_id, total_amount, status, delivery_address)
        VALUES (?, ?, 'pending', ?)
        RETURNING id
        "#
    )
    .bind(order_data.user_id)
    .bind(total_amount)
    .bind(&order_data.delivery_address)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        eprintln!("Ошибка создания заказа: {:?}", e);
        actix_web::error::ErrorInternalServerError("Ошибка создания заказа")
    })?
    .get("id");

    // Добавляем товары в order_items
    for item in &cart_items {
        let product_id: i64 = item.get("product_id");
        let quantity: i32 = item.get("quantity");
        let price: f64 = item.get("price");
        
        sqlx::query(
            r#"
            INSERT INTO order_items (order_id, product_id, quantity, price)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(order_id)
        .bind(product_id)
        .bind(quantity)
        .bind(price)
        .execute(pool)
        .await
        .map_err(|e| {
            eprintln!("Ошибка добавления товара в заказ: {:?}", e);
            actix_web::error::ErrorInternalServerError("Ошибка добавления товара в заказ")
        })?;
    }

    // Очищаем корзину
    sqlx::query("DELETE FROM cart WHERE session_id = ?")
        .bind(&session_id)
        .execute(pool)
        .await
        .map_err(|e| {
            eprintln!("Ошибка очистки корзины: {:?}", e);
            actix_web::error::ErrorInternalServerError("Ошибка очистки корзины")
        })?;

    // Отправляем сообщение пользователю с подтверждением заказа и кнопкой оплаты
    match state.telegram_notifier.send_order_confirmation(
        order_id,
        order_data.user_id,
        &cart_items.iter().map(|item| crate::telegram_notifications::CartItemData {
            product_id: item.get("product_id"),
            quantity: item.get("quantity"),
            name: item.get("name"),
            price: item.get("price"),
        }).collect::<Vec<_>>(),
        &order_data.delivery_address,
        total_amount,
        order_data.telegram_username.as_deref()
    ).await {
        Ok(_) => {
            println!("✅ Сообщение с подтверждением заказа отправлено пользователю");
        },
        Err(e) => {
            eprintln!("⚠️ Не удалось отправить сообщение в Telegram: {:?}", e);
            println!("ℹ️ Это нормально при тестировании с фиктивным user_id");
            println!("ℹ️ В продакшене WebApp будет запущен внутри Telegram с реальным user_id");
        }
    }

    println!("✅ Заказ №{} успешно создан на сумму {:.2} TON", order_id, total_amount);

    Ok(HttpResponse::Ok().json(json!({
        "order_id": order_id,
        "total_amount": total_amount,
        "status": "pending",
        "message": "Заказ создан успешно"
    })))
}

#[derive(Debug, Deserialize)]
pub struct PaymentConfirmation {
    pub order_id: i64,
    pub transaction_hash: String,
    pub amount: f64,
    pub wallet_address: String,
}

#[post("/payment-confirmation")]
pub async fn confirm_payment(
    state: web::Data<AppState>,
    payment_data: web::Json<PaymentConfirmation>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool = &state.db_pool;

    // Проверяем существование заказа
    let order = sqlx::query(
        "SELECT id, user_id, total_amount, status FROM orders WHERE id = ?"
    )
    .bind(payment_data.order_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        eprintln!("Ошибка получения заказа: {:?}", e);
        actix_web::error::ErrorInternalServerError("Ошибка получения заказа")
    })?;

    let order = match order {
        Some(order) => order,
        None => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Заказ не найден"
            })));
        }
    };

    // Проверяем, что заказ еще не оплачен
    let status: String = order.get("status");
    if status != "pending" {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Заказ уже обработан"
        })));
    }

    // Проверяем сумму платежа (с небольшой погрешностью)
    let order_total: f64 = order.get::<f64, _>("total_amount");
    if (payment_data.amount - order_total).abs() > 0.01 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Неверная сумма платежа"
        })));
    }

    // Создаем запись о платеже
    sqlx::query(
        r#"
        INSERT INTO payments (order_id, user_id, amount, wallet_address, status, transaction_hash)
        VALUES (?, ?, ?, ?, 'confirmed', ?)
        "#
    )
    .bind(payment_data.order_id.to_string())
    .bind(order.get::<i64, _>("user_id"))
    .bind(payment_data.amount)
    .bind(&payment_data.wallet_address)
    .bind(&payment_data.transaction_hash)
    .execute(pool)
    .await
    .map_err(|e| {
        eprintln!("Ошибка создания платежа: {:?}", e);
        actix_web::error::ErrorInternalServerError("Ошибка создания платежа")
    })?;

    // Получаем username пользователя (если есть)
    let username = None; // TODO: Можно добавить поле username в orders или получать из Telegram API

    // Уведомляем администраторов о новом заказе
    if let Err(e) = state.telegram_notifier.notify_admin_new_order(payment_data.order_id, username).await {
        eprintln!("Ошибка уведомления администраторов: {:?}", e);
        // Продолжаем выполнение, даже если уведомление не отправлено
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "message": "Платеж подтвержден"
    })))
}

#[derive(Debug, Deserialize)]
pub struct TelegramWebhookUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub reply_to_message: Option<Box<TelegramMessage>>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
}

#[post("/telegram-webhook")]
pub async fn telegram_webhook(
    state: web::Data<AppState>,
    update: web::Json<TelegramWebhookUpdate>,
) -> Result<HttpResponse, actix_web::Error> {
    // Проверяем, что сообщение пришло из админ-канала
    if let Some(message) = &update.message {
        if message.chat.id == -1002502108391 { // ID админ-канала
            // Проверяем, является ли это ответом на сообщение о заказе
            if let Some(reply_to) = &message.reply_to_message {
                if let Some(text) = &reply_to.text {
                    // Извлекаем ID заказа из оригинального сообщения
                    if let Some(order_id) = extract_order_id_from_text(text) {
                        if let Some(comment_text) = &message.text {
                            // Пересылаем комментарий пользователю
                            if let Err(e) = state.telegram_notifier.forward_admin_comment_to_user(order_id, comment_text).await {
                                eprintln!("Ошибка пересылки комментария администратора: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "status": "ok"
    })))
}

// Вспомогательная функция для извлечения ID заказа из текста сообщения
fn extract_order_id_from_text(text: &str) -> Option<i64> {
    // Ищем паттерн "ID: число)" в тексте
    if let Some(start) = text.find("(ID: ") {
        let start = start + 5; // Длина "(ID: "
        if let Some(end) = text[start..].find(')') {
            if let Ok(order_id) = text[start..start + end].parse::<i64>() {
                return Some(order_id);
            }
        }
    }
    None
}

// Helper function to list all tables in the database
async fn list_all_tables(pool: &SqlitePool) -> Result<Vec<String>, sqlx::Error> {
    println!("\nListing all tables in the database...");
    let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type='table'")
        .fetch_all(pool)
        .await?;

    let mut table_names = Vec::new();
    for table in tables {
        let name: String = table.get(0);
        println!("- {}", name);
        table_names.push(name);
    }

    if table_names.is_empty() {
        println!("⚠️  No tables found in the database");
    } else {
        println!("Found {} tables", table_names.len());
    }

    Ok(table_names)
}

////////////////////////////////////////////
// Обновление количества товара в корзин

// Конфигурация маршрутов
// Simple health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json("Server is running!")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Health check endpoint
            .service(
                web::resource("/health")
                    .route(web::get().to(health_check))
            )
            // Products routes - using service() for handlers with macros
            .service(list_products)
            .service(create_product)
            .service(update_product)
            .service(delete_product)
            .service(get_product_handler)  // Добавили get_product_handler в корень конфигурации
            .service(
                web::resource("/categories/{id}/products")
                    .route(web::get().to(get_products_by_category))
            )

            // Categories routes - using service() for handlers with macros
            .service(list_categories)
            .service(create_category)
            .service(get_category)
            .service(update_category)
            .service(delete_category)

            // Cart routes - using service() for handlers with macros
            .service(add_to_cart)
            .service(get_cart)
            .service(update_cart_item)
            .service(remove_cart_item)

            // Cart count route - using route() for handlers without macros
            .service(
                web::resource("/cart/count")
                    .route(web::get().to(get_cart_count))
            )

            // Order and payment routes - using service() for handlers with macros
            .service(create_order)
            .service(confirm_payment)
            .service(telegram_webhook) // Добавили telegram_webhook в корень конфигурации
    );
}