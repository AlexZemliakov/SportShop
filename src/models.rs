use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub price: f64,
    pub stock: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>, // Изменено на String для простоты
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ProductCreate {
    pub category_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub image_url: Option<String>,
    pub stock: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub telegram_id: Option<i64>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreate {
    pub telegram_id: Option<i64>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub status: String, // "new", "processing", "completed", "cancelled"
    pub total_amount: f64,
    pub ton_address: Option<String>,
    pub payment_status: String, // "pending", "paid", "failed"
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCreate {
    pub user_id: i64,
    pub total_amount: f64,
    pub ton_address: Option<String>,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderUpdate {
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub comments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: i64,
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub price_at_order: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemCreate {
    pub order_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub price_at_order: f64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CartItem {
    pub id: i64,
    pub user_id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItemCreate {
    pub user_id: i64,
    pub product_id: i64,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Admin {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String, // "manager", "admin"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminCreate {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub order_id: i64,
    pub ton_amount: f64,
    pub return_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub status: String,
    pub payment_url: Option<String>,
    pub message: Option<String>,
}