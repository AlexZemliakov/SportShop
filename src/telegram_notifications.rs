use crate::models::{Order, Payment};
use sqlx::SqlitePool;
use teloxide::prelude::*;
use teloxide::types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
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

pub struct TelegramNotifier {
    bot: Bot,
    admin_chat_id: i64,
    pub db_pool: SqlitePool,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, admin_chat_id: i64, db_pool: SqlitePool) -> Self {
        Self {
            bot: Bot::new(bot_token),
            admin_chat_id,
            db_pool,
        }
    }

    pub async fn notify_admin_new_order(&self, order_id: i64) -> Result<(), NotificationError> {
        // Получаем заказ из базы данных
        let order = sqlx::query!(
            "SELECT id, user_id, total_amount, status FROM orders WHERE id = ?",
            order_id
        )
            .fetch_one(&self.db_pool)
            .await?;

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Выполнено", format!("complete_{}", order_id))
        ]]);

        // Отправляем сообщение в группу администраторов
        self.bot.send_message(
            ChatId(self.admin_chat_id),
            format!("Новый заказ №{} на сумму {} ₽.", order.id, order.total_amount)
        )
            .reply_markup(keyboard)
            .send()
            .await?;

        Ok(())
    }

    pub async fn handle_callback_query(&self, callback_query: teloxide::types::CallbackQuery) -> Result<(), NotificationError> {
        if let Some(data) = callback_query.data {
            if data.starts_with("complete_") {
                let order_id: i64 = data[9..].parse::<i64>()
                    .map_err(|e: std::num::ParseIntError| NotificationError::ParseError(e.to_string()))?;

                // Обновляем статус заказа
                sqlx::query!(
                    "UPDATE orders SET status = 'completed' WHERE id = ?",
                    order_id
                )
                    .execute(&self.db_pool)
                    .await?;

                // Получаем информацию о заказе
                let order = sqlx::query!(
                    "SELECT id, user_id, total_amount, status FROM orders WHERE id = ?",
                    order_id
                )
                    .fetch_one(&self.db_pool)
                    .await?;

                // Отвечаем на callback query
                self.bot.answer_callback_query(callback_query.id).await?;

                // Отправляем сообщение администратору
                self.bot.send_message(
                    ChatId(callback_query.from.id.0 as i64),
                    "Заказ отмечен как выполненный"
                ).await?;

                // Отправляем сообщение пользователю
                self.bot.send_message(
                    ChatId(order.user_id),
                    format!("Ваш заказ №{} отправлен. Спасибо за покупку!", order_id)
                ).await?;
            }
        }

        Ok(())
    }

    pub async fn notify_new_order(&self, order: &Order) -> Result<(), NotificationError> {
        let message = format!(
            "🛒 *Новый заказ \\#{}*\n\
            \n\
            *Пользователь:* {}\n\
            *Сумма:* {:.2} TON\n\
            *Статус:* {}\n\
            \n\
            [Просмотреть заказ](https://your-admin-panel.com/orders/{})",
            order.id,
            order.user_id,
            order.total_amount,
            order.status,
            order.id
        );

        self.bot
            .send_message(ChatId(self.admin_chat_id), message)
            .parse_mode(ParseMode::MarkdownV2)
            .reply_markup(self.order_actions_keyboard(order.id))
            .await?;

        Ok(())
    }

    pub async fn notify_payment_received(&self, payment: &Payment) -> Result<(), NotificationError> {
        // Получаем заказ по ID платежа
        let order = sqlx::query!(
            "SELECT id, user_id, total_amount, status FROM orders WHERE id = ?",
            payment.order_id
        )
            .fetch_one(&self.db_pool)
            .await?;

        let message = format!(
            "💰 *Оплата получена*\n\
            \n\
            *Заказ \\#{}*\n\
            *Сумма:* {:.2} TON\n\
            *Кошелек:* `{}`\n\
            \n\
            [Подробнее](https://your-admin-panel.com/payments/{})",
            order.id,
            payment.amount,
            payment.wallet_address,
            payment.id
        );

        self.bot
            .send_message(ChatId(self.admin_chat_id), message)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }

    pub async fn notify_order_completed(
        &self,
        order_id: i64,
        admin_comment: Option<String>,
    ) -> Result<(), NotificationError> {
        let order = sqlx::query!(
            "SELECT id, user_id, status FROM orders WHERE id = ?",
            order_id
        )
            .fetch_one(&self.db_pool)
            .await?;

        let mut message = format!(
            "✅ *Заказ \\#{} выполнен*\n\
            \n\
            *Статус:* {}\n",
            order.id,
            order.status
        );

        if let Some(comment) = admin_comment {
            message.push_str(&format!("\n*Комментарий:* {}", comment));
        }

        self.bot
            .send_message(ChatId(order.user_id), message)
            .parse_mode(ParseMode::MarkdownV2)
            .await?;

        Ok(())
    }

    pub async fn notify_user_with_payment(
        &self,
        order_id: i64
    ) -> Result<(), NotificationError> {
        let order = sqlx::query!(
            "SELECT id, user_id, total_amount FROM orders WHERE id = ?",
            order_id
        )
            .fetch_one(&self.db_pool)
            .await?;

        let keyboard = InlineKeyboardMarkup::new(vec![vec![
            InlineKeyboardButton::callback("Оплатить", format!("pay_{}", order_id))
        ]]);

        self.bot.send_message(
            ChatId(order.user_id),
            format!("Ваш заказ №{} на сумму {} ₽.", order.id, order.total_amount)
        )
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }

    fn order_actions_keyboard(&self, order_id: i64) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(vec![
            vec![InlineKeyboardButton::callback(
                "✅ Выполнено",
                format!("complete_{}", order_id)
            )],
            vec![InlineKeyboardButton::callback(
                "✏️ Комментировать",
                format!("comment_{}", order_id)
            )],
            vec![InlineKeyboardButton::url(
                "🔍 Просмотреть",
                Url::parse(&format!("https://your-admin-panel.com/orders/{}", order_id))
                    .unwrap_or_else(|_| Url::parse("https://your-admin-panel.com").unwrap()),
            )],
        ])
    }
}