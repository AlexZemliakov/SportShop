# Настройка Telegram WebApp для SportShop

## 🚀 Быстрая настройка

### 1. Настройка WebApp в BotFather

1. Откройте [@BotFather](https://t.me/BotFather) в Telegram
2. Отправьте команду `/mybots`
3. Выберите вашего бота
4. Нажмите **"Bot Settings"** → **"Menu Button"**
5. Выберите **"Configure menu button"**
6. Введите:
   - **Button text**: `🛒 Магазин`
   - **Web App URL**: `https://yourdomain.com` (замените на ваш домен)

### 2. Альтернативный способ - команда /setmenubutton

```
/setmenubutton
@your_bot_username
🛒 Магазин
https://yourdomain.com
```

### 3. Для локального тестирования

Если у вас нет домена, можете использовать:
- **ngrok**: `ngrok http 8080` (получите HTTPS URL)
- **localtunnel**: `npx localtunnel --port 8080`

## 🔧 Проверка настройки

### В Telegram боте:
1. Найдите вашего бота
2. Внизу должна появиться кнопка **"🛒 Магазин"**
3. При нажатии должен открыться WebApp

### В консоли браузера:
После открытия WebApp проверьте:
```javascript
console.log(window.Telegram.WebApp.initDataUnsafe);
```

Должно показать данные пользователя:
```json
{
  "user": {
    "id": 123456789,
    "first_name": "John",
    "username": "john_doe",
    "language_code": "ru"
  }
}
```

## ⚠️ Важные моменты

1. **HTTPS обязателен** - Telegram WebApp работает только по HTTPS
2. **Реальный домен** - localhost не работает для WebApp
3. **Правильный user_id** - только при запуске из Telegram
4. **Инициализация** - `window.Telegram.WebApp.ready()` должен быть вызван

## 🐛 Решение проблем

### Проблема: "user_id=12345" в логах
**Решение**: WebApp запущен не из Telegram или неправильно настроен

### Проблема: "ChatNotFound" ошибка
**Решение**: Пользователь должен написать боту `/start` перед заказом

### Проблема: Кнопка "Оплатить" не появляется
**Решение**: Проверьте что user_id реальный (не 12345)

## 📱 Тестирование

1. Откройте бота в Telegram
2. Нажмите кнопку **"🛒 Магазин"**
3. Добавьте товары в корзину
4. Нажмите **"Оформить заказ"**
5. Проверьте консоль браузера на наличие реального user_id
6. Заполните адрес доставки
7. Должно прийти сообщение с кнопкой **"Оплатить"**

## 🚀 Деплой на продакшн

### Рекомендуемые VPS провайдеры:
- **DigitalOcean** (от $5/месяц)
- **Vultr** (от $3.50/месяц)
- **Linode** (от $5/месяц)

### Настройка домена:
1. Купите домен (например, на Namecheap)
2. Настройте A-запись на IP вашего сервера
3. Установите SSL сертификат (Let's Encrypt)
4. Обновите URL в BotFather

### Переменные окружения:
```bash
export TELEGRAM_BOT_TOKEN="your_bot_token"
export ADMIN_CHAT_ID="-1002502108391"
export DATABASE_URL="sqlite:sportshop.db"
```
