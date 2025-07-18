use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use chrono::{NaiveDateTime, DateTime, Utc};

// Кастомная сериализация/десериализация для Option<NaiveDateTime>
pub mod naive_datetime_serde {
    use super::*;
    use serde::{Serializer, Deserializer, de::Error};

    pub fn serialize<S>(
        date: &Option<NaiveDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => serializer.serialize_i64(dt.and_utc().timestamp()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let timestamp: Option<i64> = Option::deserialize(deserializer)?;
        match timestamp {
            Some(ts) => Ok(Some(
                DateTime::from_timestamp(ts, 0)
                    .ok_or_else(|| D::Error::custom("invalid timestamp"))?
                    .naive_utc(),
            )),
            None => Ok(None),
        }
    }
}

// Вспомогательная структура для работы с SQLx
#[derive(sqlx::FromRow)]
struct RawDateTime(Option<DateTime<Utc>>);

impl From<RawDateTime> for Option<NaiveDateTime> {
    fn from(raw: RawDateTime) -> Self {
        raw.0.map(|dt| dt.naive_local())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub stock: i32,
    pub description: String,
    pub image_url: String,
    pub category_id: i32,
    #[serde(with = "naive_datetime_serde", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
}

// Реализация FromRow для Product с ручным маппингом дат
impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for Product {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        let created_at: Option<DateTime<Utc>> = row.try_get("created_at")?;

        Ok(Product {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            price: row.try_get("price")?,
            stock: row.try_get("stock")?,
            description: row.try_get("description")?,
            image_url: row.try_get("image_url")?,
            category_id: row.try_get("category_id")?,
            created_at: created_at.map(|dt| dt.naive_local()),
        })
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub telegram_id: Option<i64>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(with = "naive_datetime_serde")]
    pub created_at: Option<NaiveDateTime>,
}
// Остальные модели по аналогии...

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreate {
    pub telegram_id: Option<i64>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: i64,
    pub user_id: i64,
    pub total_amount: f64,
    pub status: String,
    pub delivery_address: Option<String>,
    pub dialog_active: Option<bool>,
    pub telegram_message_id: Option<i64>,
    #[serde(with = "naive_datetime_serde", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<NaiveDateTime>,
    #[serde(with = "naive_datetime_serde")]
    pub updated_at: Option<NaiveDateTime>,
    pub ton_address: Option<String>,
    pub payment_status: String,
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

#[derive(Debug, FromRow, Serialize, Deserialize)]
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

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct CartItem {
    pub id: i64,
    pub product_id: i64,
    pub quantity: i32,
    pub user_session: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CartItemRequest {
    pub product_id: i64,
    pub quantity: i32,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CartItemCreate {
    pub user_id: i64,
    pub product_id: i64,
    pub quantity: i32,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Admin {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Payment {
    pub id: i32,
    pub order_id: String,
    pub user_id: i64,
    pub amount: f64,
    pub wallet_address: String,
    pub status: String,
    #[serde(with = "naive_datetime_serde")]
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewPayment {
    pub order_id: String,
    pub user_id: i64,
    pub amount: f64,
    pub wallet_address: String,
}