use crate::telegram_notifications::TelegramNotifier;
use crate::ton_payment::TonProcessor;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, Message, Update};
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
                    .branch(
                        dptree::filter(|msg: Message| msg.text().is_some())
                            .filter_command::<Command>()
                            .endpoint(Self::command_handler)
                    )
                    .branch(
                        dptree::filter(|msg: Message| msg.text().is_some())
                            .endpoint(Self::message_handler)
                    )
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
        _bot_instance: Arc<Self>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match cmd {
            Command::Start => {
                bot.send_message(msg.chat.id, "Добро пожаловать в SportShop! 🏃‍♂️\n\nИспользуйте WebApp для просмотра товаров и оформления заказов.")
                    .await?;
            }
            Command::Help => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
        }
        Ok(())
    }

    async fn message_handler(
        bot: Bot,
        msg: Message,
        bot_instance: Arc<Self>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Some(text) = msg.text() {
            let user_id = msg.from.as_ref().map(|user| user.id.0 as i64).unwrap_or(0);
            
            // Проверяем, есть ли активный диалог у пользователя
            if let Ok(is_active) = bot_instance.notifier.is_dialog_active(user_id).await {
                if is_active {
                    // Пересылаем сообщение пользователя администраторам
                    if let Err(e) = bot_instance.notifier.forward_user_message_to_admin(user_id, text).await {
                        eprintln!("Ошибка пересылки сообщения администраторам: {:?}", e);
                    }
                } else {
                    // Диалог неактивен - сообщаем пользователю
                    bot.send_message(
                        msg.chat.id,
                        "У вас нет активных заказов. Оформите заказ через WebApp для начала диалога с поддержкой."
                    ).await?;
                }
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
            let user_id = q.from.id.0 as i64;

            if data.starts_with("pay_") {
                // Обработка нажатия кнопки "Оплатить"
                if let Ok(order_id) = data[4..].parse::<i64>() {
                    if let Err(e) = bot_instance.notifier.handle_payment_request(order_id, user_id).await {
                        eprintln!("Ошибка обработки запроса на оплату: {:?}", e);
                        bot.answer_callback_query(q.id)
                            .text("Произошла ошибка при создании платежа")
                            .await?;
                    } else {
                        bot.answer_callback_query(q.id)
                            .text("Ссылка на оплату отправлена")
                            .await?;
                    }
                }
            } else if data.starts_with("complete_") {
                // Обработка нажатия кнопки "Выполнено" (только для администраторов)
                if let Ok(order_id) = data[9..].parse::<i64>() {
                    if let Some(message) = &q.message {
                        if let Err(e) = bot_instance.notifier.handle_order_completion(order_id, message.id()).await {
                            eprintln!("Ошибка завершения заказа: {:?}", e);
                            bot.answer_callback_query(q.id)
                                .text("Произошла ошибка при завершении заказа")
                                .await?;
                        } else {
                            bot.answer_callback_query(q.id)
                                .text("Заказ отмечен как выполненный")
                                .await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    // Метод для обработки подтверждения оплаты TON (вызывается извне при получении платежа)
    pub async fn handle_payment_confirmation(&self, order_id: i64, username: Option<&str>) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Уведомляем администраторов о новом заказе
        if let Err(e) = self.notifier.notify_admin_new_order(order_id, username).await {
            eprintln!("Ошибка уведомления администраторов: {:?}", e);
        }
        Ok(())
    }

    // Метод для обработки комментариев администраторов (вызывается при получении комментария в канале)
    pub async fn handle_admin_comment(&self, order_id: i64, comment_text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Err(e) = self.notifier.forward_admin_comment_to_user(order_id, comment_text).await {
            eprintln!("Ошибка пересылки комментария пользователю: {:?}", e);
        }
        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
enum Command {
    #[command(description = "Начать работу с ботом")]
    Start,
    #[command(description = "Показать справку")]
    Help,
}