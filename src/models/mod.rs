use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: i64,
    pub image_url: Option<String>,
    pub category_id: Option<i64>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CartItem {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub user_id: Option<i64>,
}