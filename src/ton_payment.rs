use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, FromRow};
use chrono::NaiveDateTime;
use reqwest::{Client, header};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum TonPaymentError {
    #[error("API request failed: {0}")]
    ApiError(#[from] reqwest::Error),
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Payment verification failed")]
    VerificationFailed,
    #[error("Invalid payment amount")]
    InvalidAmount,
    #[error("Environment variable error: {0}")]
    EnvError(String),
    #[error("JSON parsing error: {0}")]
    JsonError(String),
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: i32,
    pub order_id: String,
    pub user_id: i64,
    pub amount: f64,
    pub wallet_address: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub ton_payment_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct TonInvoiceRequest {
    amount: f64,
    order_id: String,
    wallet: String,
    callback_url: String,
}

#[derive(Debug, Deserialize)]
struct TonInvoiceResponse {
    payment_url: String,
    payment_id: String,
}

#[derive(Debug, Deserialize)]
struct TonPaymentStatus {
    status: String,
}

pub struct TonProcessor {
    client: Client,
    api_url: String,
    api_key: String,
    merchant_wallet: String,
    callback_url: String,
    db_pool: SqlitePool,
}

impl TonProcessor {


    pub fn new(db_pool: SqlitePool) -> Result<Self, TonPaymentError> {
        Ok(Self {
            client: Client::new(),
            api_url: std::env::var("TON_API_URL")
                .map_err(|e| TonPaymentError::EnvError(e.to_string()))?,
            api_key: std::env::var("TON_API_KEY")
                .map_err(|e| TonPaymentError::EnvError(e.to_string()))?,
            merchant_wallet: std::env::var("MERCHANT_WALLET")
                .unwrap_or_else(|_| "UQCbShhQNTKUd3GvKJsBxeiwLHuJghq9r7FQrkC5mSOfLXgy".to_string()),
            callback_url: std::env::var("CALLBACK_URL")
                .unwrap_or_else(|_| "https://yourdomain.com/api/payment-callback".to_string()),
            db_pool,
        })
    }

    pub async fn create_payment(
        &self,
        user_id: i64,
        amount: f64,
    ) -> Result<(String, String), TonPaymentError> {
        if amount <= 0.0 {
            return Err(TonPaymentError::InvalidAmount);
        }

        let order_id = Uuid::new_v4().to_string();

        // Сохраняем платеж в БД
        let payment_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO payments (order_id, user_id, amount, wallet_address, status)
            VALUES (?, ?, ?, ?, 'pending')
            RETURNING id
            "#
        )
            .bind(&order_id)
            .bind(user_id)
            .bind(amount)
            .bind(&self.merchant_wallet)
            .fetch_one(&self.db_pool)
            .await?;

        // Создаем инвойс в TON
        let payment_request = TonInvoiceRequest {
            amount,
            order_id: order_id.clone(),
            wallet: self.merchant_wallet.clone(),
            callback_url: self.callback_url.clone(),
        };

        let response = self
            .client
            .post(&format!("{}/api/v1/invoice", self.api_url))
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&payment_request)  // Теперь работает благодаря правильным трейтам
            .send()
            .await?;

        let response: TonInvoiceResponse = response
            .json::<TonInvoiceResponse>()  // Явное указание типа
            .await
            .map_err(|e| TonPaymentError::JsonError(e.to_string()))?;

        // Обновляем платеж с TON payment_id
        sqlx::query(
            "UPDATE payments SET ton_payment_id = ? WHERE id = ?",
        )
            .bind(&response.payment_id)
            .bind(payment_id)
            .execute(&self.db_pool)
            .await?;

        Ok((order_id, response.payment_url))
    }

    pub async fn verify_payment(
        &self,
        order_id: &str,
    ) -> Result<bool, TonPaymentError> {
        let payment: Payment = sqlx::query_as(
            "SELECT * FROM payments WHERE order_id = ?",
        )
            .bind(order_id)
            .fetch_one(&self.db_pool)
            .await?;

        if payment.status == "paid" {
            return Ok(true);
        }

        let payment_id = payment.ton_payment_id
            .as_ref()
            .ok_or(TonPaymentError::VerificationFailed)?;

        let response = self
            .client
            .get(&format!("{}/api/v1/payments/{}", self.api_url, payment_id))
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let status: TonPaymentStatus = response
            .json::<TonPaymentStatus>()  // Явное указание типа
            .await
            .map_err(|e| TonPaymentError::JsonError(e.to_string()))?;

        if status.status == "paid" {
            sqlx::query(
                "UPDATE payments SET status = 'paid' WHERE order_id = ?",
            )
                .bind(order_id)
                .execute(&self.db_pool)
                .await?;
            return Ok(true);
        }

        // Этот код должен быть в обработчике callback_query в telegram_bot.rs
    // Перемещен туда для правильной организации кода

        Ok(false)
    }
}
