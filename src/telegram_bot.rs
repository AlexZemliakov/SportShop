use crate::telegram_notifications::TelegramNotifier;
use crate::ton_payment::TonProcessor;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, ChatId, Message};
use teloxide::utils::command::BotCommands;
use std::sync::Arc;
use std::error::Error;

pub struct TelegramBot {
    bot: Bot,
    notifier: Arc<TelegramNotifier>,
    ton_processor: Arc<TonProcessor>,
}

impl TelegramBot {
    pub fn new(bot_token: String, notifier: Arc<TelegramNotifier>, ton_processor: Arc<TonProcessor>) -> Self {
        Self {
            bot: Bot::new(bot_token),
            notifier,
            ton_processor,
        }
    }

    pub async fn start(self) {
        let handler = dptree::entry()
            .branch(
                Update::filter_message()
                    .filter_command::<Command>()
                    .endpoint(Self::command_handler),
            )
            .branch(
                Update::filter_callback_query()
                    .endpoint(Self::callback_handler),
            );

        let bot_instance = Arc::new(self);
        let bot = bot_instance.bot.clone();

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![bot_instance])
            .build()
            .dispatch()
            .await;
    }

    async fn command_handler(
        bot: Bot,
        msg: Message,
        cmd: Command,
        bot_instance: Arc<Self>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match cmd {
            Command::Start(arg) => {
                if !arg.is_empty() && arg.starts_with("order_") {
                    let order_id = arg.trim_start_matches("order_").parse::<i64>()?;
                    bot_instance.notifier.notify_user_with_payment(order_id).await?;
                } else {
                    bot.send_message(msg.chat.id, "Добро пожаловать в SportShop! Используйте наш веб-магазин для покупок.").await?;
                }
            }
            Command::Help => {
                bot.send_message(msg.chat.id, "Это бот SportShop. Используйте наш веб-магазин для покупок.").await?;
            }
        }
        Ok(())
    }

    async fn callback_handler(
        bot: Bot,
        q: CallbackQuery,
        bot_instance: Arc<Self>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(data) = &q.data {
            if data.starts_with("pay_") {
                // Получить ID заказа
                let order_id = data[4..].parse::<i64>()?;

                // Получаем информацию о заказе
                let order = sqlx::query!(
                    "SELECT id, user_id, total_amount FROM orders WHERE id = ?",
                    order_id
                )
                    .fetch_one(&bot_instance.notifier.db_pool)
                    .await?;

                // Создать оплату
                let (_payment_id, payment_url) = bot_instance.ton_processor.create_payment(order.user_id, order.total_amount).await?;

                // Ответ пользователю
                bot.answer_callback_query(q.id).await?;
                bot.send_message(
                    ChatId(q.from.id.0 as i64),
                    format!("Оплата создана! Оплатите по ссылке: {}", payment_url)
                ).await?;
            } else if data.starts_with("complete_") {
                // Обработка завершения заказа
                bot_instance.notifier.handle_callback_query(q.clone()).await?;
            }
        }
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
enum Command {
    #[command(description = "Начать работу с ботом")]
    Start(String),
    #[command(description = "Показать помощь")]
    Help,
}