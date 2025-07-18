use crate::models::{Order, Payment};
use sqlx::SqlitePool;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode, MessageId};
use thiserror::Error;
use reqwest::Url;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Telegram API error: {0}")]
    TelegramError(#[from] teloxide::RequestError),
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[derive(Debug)]
pub struct CartItemData {
    pub product_id: i64,
    pub quantity: i32,
    pub name: String,
    pub price: f64,
}

pub struct TelegramNotifier {
    bot: Bot,
    admin_chat_id: i64,
    pub db_pool: SqlitePool,
    payment_wallet: String,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, admin_chat_id: i64, db_pool: SqlitePool) -> Self {
        Self {
            bot: Bot::new(bot_token),
            admin_chat_id,
            db_pool,
            payment_wallet: "UQCbShhQNTKUd3GvKJsBxeiwLHuJghq9r7FQrkC5mSOfLXgy".to_string(),
        }
    }

    // 1. Отправка подтверждения заказа пользователю с кнопкой оплаты
    pub async fn send_order_confirmation(
        &self,
        order_id: i64,
        user_id: i64,
        cart_items: &[CartItemData],
        delivery_address: &str,
        total_amount: f64,
        _username: Option<&str>,
    ) -> Result<(), NotificationError> {
        let mut order_text = "🛒 *Ваш заказ*:\n".to_string();
        
        for item in cart_items {
            order_text.push_str(&format!("- {} (×{})\n", item.name, item.quantity));
        }
        
        order_text.push_str(&format!(
            "📦 *Адрес доставки*: {}\n💰 *Сумма к оплате*: {:.2} TON",
            delivery_address, total_amount
        ));

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Оплатить", format!("pay_{}", order_id))
        ]]);

        let message = self.bot
            .send_message(ChatId(user_id), order_text)
            .parse_mode(ParseMode::Markdown)
            .reply_markup(keyboard)
            .send()
            .await?;

        // Сохраняем message_id для дальнейшего использования
        sqlx::query!(
            "UPDATE orders SET telegram_message_id = ?, dialog_active = TRUE WHERE id = ?",
            message.id.0,
            order_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // 2. Обработка нажатия кнопки "Оплатить" - создание TON платежа
    pub async fn handle_payment_request(&self, order_id: i64, user_id: i64) -> Result<(), NotificationError> {
        // Получаем данные заказа
        let order = sqlx::query!(
            "SELECT total_amount FROM orders WHERE id = ? AND user_id = ?",
            order_id,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Создаем TON платеж (здесь должна быть интеграция с TON)
        let payment_url = format!(
            "ton://transfer/{}?amount={}&text=Order_{}",
            self.payment_wallet,
            (order.total_amount * 1_000_000_000.0) as i64, // Convert to nanotons
            order_id
        );

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::url("Оплатить в TON", Url::parse(&payment_url).unwrap())
        ]]);

        self.bot
            .send_message(
                ChatId(user_id),
                format!("💳 *Оплата заказа №{}*\n\nСумма: {:.2} TON\n\nНажмите кнопку ниже для оплаты:", order_id, order.total_amount)
            )
            .parse_mode(ParseMode::Markdown)
            .reply_markup(keyboard)
            .send()
            .await?;

        Ok(())
    }

    // 3. Уведомление администраторов о новом заказе после оплаты
    pub async fn notify_admin_new_order(&self, order_id: i64, username: Option<&str>) -> Result<MessageId, NotificationError> {
        // Получаем полную информацию о заказе
        let order = sqlx::query!(
            "SELECT o.id, o.user_id, o.total_amount, o.delivery_address, o.created_at 
             FROM orders o WHERE o.id = ?",
            order_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Получаем товары заказа
        let order_items = sqlx::query!(
            "SELECT oi.quantity, p.name 
             FROM order_items oi 
             JOIN products p ON oi.product_id = p.id 
             WHERE oi.order_id = ?",
            order_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut message_text = format!(
            "🚀 *Новый заказ* (ID: {})\n👤 *Покупатель*: {}\n📦 *Адрес*: {}\n🛒 *Состав заказа*:\n",
            order.id,
            username.map(|u| format!("@{}", u)).unwrap_or_else(|| format!("ID: {}", order.user_id)),
            order.delivery_address.unwrap_or_else(|| "Не указан".to_string())
        );

        for item in order_items {
            message_text.push_str(&format!("- {} (×{})\n", item.name, item.quantity));
        }

        message_text.push_str(&format!("💰 *Оплачено*: {:.2} TON", order.total_amount));

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Выполнено", format!("complete_{}", order_id))
        ]]);

        let message = self.bot
            .send_message(ChatId(self.admin_chat_id), message_text)
            .parse_mode(ParseMode::Markdown)
            .reply_markup(keyboard)
            .send()
            .await?;

        // Сохраняем ID сообщения администраторов для связи с комментариями
        sqlx::query!(
            "UPDATE orders SET status = 'paid' WHERE id = ?",
            order_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(message.id)
    }

    // 4. Обработка нажатия кнопки "Выполнено"
    pub async fn handle_order_completion(&self, order_id: i64, message_id: MessageId) -> Result<(), NotificationError> {
        // Получаем информацию о заказе
        let order = sqlx::query!(
            "SELECT user_id, dialog_active FROM orders WHERE id = ?",
            order_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Обновляем статус заказа и закрываем диалог
        sqlx::query!(
            "UPDATE orders SET status = 'completed', dialog_active = FALSE WHERE id = ?",
            order_id
        )
        .execute(&self.db_pool)
        .await?;

        // Редактируем сообщение в канале администраторов - убираем кнопку
        if let Err(e) = self.bot
            .edit_message_reply_markup(ChatId(self.admin_chat_id), message_id)
            .reply_markup(InlineKeyboardMarkup::new(Vec::<Vec<InlineKeyboardButton>>::new()))
            .send()
            .await {
            eprintln!("Ошибка редактирования сообщения: {:?}", e);
        }

        // Отправляем уведомление пользователю
        self.bot
            .send_message(
                ChatId(order.user_id),
                "✅ *Ваш заказ выполнен!*\nСпасибо за покупку! Диалог закрыт."
            )
            .parse_mode(ParseMode::Markdown)
            .send()
            .await?;

        Ok(())
    }

    // 5. Пересылка комментария администратора пользователю
    pub async fn forward_admin_comment_to_user(&self, order_id: i64, comment_text: &str) -> Result<(), NotificationError> {
        let order = sqlx::query!(
            "SELECT user_id, dialog_active FROM orders WHERE id = ?",
            order_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Проверяем, активен ли диалог
        if !order.dialog_active.unwrap_or(false) {
            return Ok(()); // Диалог закрыт, не пересылаем
        }

        self.bot
            .send_message(
                ChatId(order.user_id),
                format!("📢 *Ответ от поддержки*:\n{}", comment_text)
            )
            .parse_mode(ParseMode::Markdown)
            .send()
            .await?;

        Ok(())
    }

    // 6. Пересылка сообщения пользователя как комментарий в канал
    pub async fn forward_user_message_to_admin(&self, user_id: i64, message_text: &str) -> Result<(), NotificationError> {
        // Находим активный заказ пользователя
        let order = sqlx::query!(
            "SELECT id FROM orders WHERE user_id = ? AND dialog_active = TRUE ORDER BY created_at DESC LIMIT 1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(order) = order {
            // Отправляем сообщение как комментарий в канал администраторов
            self.bot
                .send_message(
                    ChatId(self.admin_chat_id),
                    format!("💬 *Сообщение от пользователя* (Заказ #{}):\n{}", order.id, message_text)
                )
                .parse_mode(ParseMode::Markdown)
                .send()
                .await?;
        }

        Ok(())
    }

    // 7. Проверка активности диалога
    pub async fn is_dialog_active(&self, user_id: i64) -> Result<bool, NotificationError> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM orders WHERE user_id = ? AND dialog_active = TRUE",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result.count > 0)
    }

    // Остальные методы для совместимости...
    pub async fn notify_new_order(&self, _order: &Order) -> Result<(), NotificationError> {
        // Deprecated - use send_order_confirmation instead
        Ok(())
    }

    pub async fn notify_payment_received(&self, payment: &Payment) -> Result<(), NotificationError> {
        if let Ok(order_id) = payment.order_id.parse::<i64>() {
            self.notify_admin_new_order(order_id, None).await?;
        }
        Ok(())
    }

    pub async fn notify_order_completed(
        &self,
        _order_id: i64,
        _admin_comment: Option<String>,
    ) -> Result<(), NotificationError> {
        // Deprecated - use handle_order_completion instead
        Ok(())
    }

    pub async fn notify_user_with_payment(
        &self,
        _order_id: i64
    ) -> Result<(), NotificationError> {
        // Deprecated - use send_order_confirmation instead
        Ok(())
    }
}