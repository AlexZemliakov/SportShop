document.addEventListener('DOMContentLoaded', function() {
    // Инициализация корзины
    initCart();

    async function initCart() {
        await loadCartItems();
        await updateCartCounter();
        setupEventListeners();
    }

    // Загрузка товаров в корзине
    async function loadCartItems() {
        try {
            const response = await fetch('/api/cart');
            if (!response.ok) throw new Error('Ошибка загрузки корзины');

            const items = await response.json();
            renderCartItems(items);
            updateTotal(items);
        } catch (error) {
            console.error('Ошибка:', error);
            showError('Ошибка загрузки корзины');
        }
    }

    // Обновление счетчика товаров
    async function updateCartCounter() {
        try {
            const response = await fetch('/api/cart/count');
            if (!response.ok) return;

            const count = await response.json();
            const counter = document.getElementById('cart-counter');
            if (counter) {
                counter.textContent = count;
                counter.style.display = count > 0 ? 'flex' : 'none';
            }
        } catch (error) {
            console.error('Ошибка счетчика:', error);
        }
    }

    // Отрисовка товаров
    function renderCartItems(items) {
        const container = document.getElementById('cartItems');

        if (!items || items.length === 0) {
            container.innerHTML = `
                <div class="empty-cart">
                    <p>Ваша корзина пуста</p>
                    <a href="/" class="btn">Вернуться к покупкам</a>
                </div>
            `;
            return;
        }

        container.innerHTML = items.map(item => `
            <div class="cart-item" data-id="${item.id}">
                <div class="cart-item-image">
                    ${item.image_url ?
            `<img src="${item.image_url}" alt="${item.name}">` :
            '<div class="no-image">Нет изображения</div>'}
                </div>
                <div class="cart-item-info">
                    <div class="cart-item-title">${item.name}</div>
                    <div class="cart-item-price">${(item.price * item.quantity).toFixed(2)} ₽</div>
                    <div class="quantity-controls">
                        <button class="quantity-btn quantity-minus">-</button>
                        <input type="text" class="quantity-input" value="${item.quantity}" readonly>
                        <button class="quantity-btn quantity-plus">+</button>
                        <button class="remove-btn">Удалить</button>
                    </div>
                </div>
            </div>
        `).join('');
    }

    // Настройка обработчиков событий
    function setupEventListeners() {
        document.addEventListener('click', async (e) => {
            const itemElement = e.target.closest('.cart-item');
            if (!itemElement) return;

            const itemId = itemElement.dataset.id;
            const input = itemElement.querySelector('.quantity-input');

            if (e.target.classList.contains('quantity-minus')) {
                await changeQuantity(itemId, parseInt(input.value) - 1);
            }
            else if (e.target.classList.contains('quantity-plus')) {
                await changeQuantity(itemId, parseInt(input.value) + 1);
            }
            else if (e.target.classList.contains('remove-btn')) {
                if (confirm('Удалить товар из корзины?')) {
                    await removeItem(itemId);
                }
            }
        });

        document.getElementById('checkoutBtn')?.addEventListener('click', checkout);
    }

    // Изменение количества товара
    async function changeQuantity(itemId, newQuantity) {
        if (newQuantity < 1) newQuantity = 1;

        try {
            const response = await fetch(`/api/cart/${itemId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
                body: JSON.stringify({ quantity: newQuantity })  // Теперь точно правильный формат
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Ошибка сервера');
            }

            // Обновляем интерфейс
            const input = document.querySelector(`.cart-item[data-id="${itemId}"] .quantity-input`);
            if (input) {
                input.value = newQuantity;
            }

            // Перезагружаем данные корзины
            await loadCartItems();
            await updateCartCounter();

        } catch (error) {
            console.error('Ошибка при изменении количества:', error);
            showError(error.message || 'Не удалось изменить количество');

            // Восстанавливаем предыдущее значение
            const input = document.querySelector(`.cart-item[data-id="${itemId}"] .quantity-input`);
            if (input) {
                input.value = input.defaultValue;
            }
        }
    }

    // Удаление товара
    async function removeItem(itemId) {
        try {
            const response = await fetch(`/api/cart/${itemId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                await Promise.all([loadCartItems(), updateCartCounter()]);
            }
        } catch (error) {
            console.error('Ошибка:', error);
            showError('Не удалось удалить товар');
        }
    }

    // Подсчет общей суммы
    function updateTotal(items) {
        if (!items || items.length === 0) {
            document.getElementById('cartTotal').textContent = '0';
            return;
        }

        const total = items.reduce((sum, item) => sum + (item.price * item.quantity), 0);
        document.getElementById('cartTotal').textContent = total.toFixed(2);
    }

    // Функция для принудительной инициализации Telegram WebApp
    function initializeTelegramWebApp() {
        showDiagnostic('=== ПРИНУДИТЕЛЬНАЯ ИНИЦИАЛИЗАЦИЯ WEBAPP ===');
        
        // Проверяем загрузку скрипта
        const script = document.querySelector('script[src*="telegram-web-app"]');
        if (script) {
            showDiagnostic(`Скрипт найден: ${script.src}`);
            showDiagnostic(`Скрипт загружен: ${script.readyState || 'unknown'}`);
        }
        
        // Ждем загрузки скрипта
        return new Promise((resolve) => {
            let attempts = 0;
            const maxAttempts = 50; // 5 секунд
            
            const checkTelegram = () => {
                attempts++;
                showDiagnostic(`Попытка ${attempts}/${maxAttempts}: window.Telegram = ${!!window.Telegram}`);
                
                if (window.Telegram && window.Telegram.WebApp) {
                    showDiagnostic('✅ Telegram WebApp найден!');
                    resolve(true);
                } else if (attempts >= maxAttempts) {
                    showDiagnostic('❌ Telegram WebApp не инициализировался за 5 секунд', true);
                    resolve(false);
                } else {
                    setTimeout(checkTelegram, 100);
                }
            };
            
            checkTelegram();
        });
    }

    // Оформление заказа
    async function checkout() {
        // Очищаем предыдущую диагностику
        const oldDiagnostic = document.getElementById('telegram-diagnostic');
        if (oldDiagnostic) {
            oldDiagnostic.remove();
        }
        
        showDiagnostic('НАЧАЛО ОФОРМЛЕНИЯ ЗАКАЗА');
        
        const cartItems = document.getElementById('cartItems');
        if (!cartItems || cartItems.children.length === 0) {
            alert('Корзина пуста!');
            return;
        }

        // Получаем адрес доставки от пользователя
        const deliveryAddress = prompt('Введите адрес доставки:');
        if (!deliveryAddress || deliveryAddress.trim() === '') {
            alert('Адрес доставки обязателен!');
            return;
        }

        // Принудительная инициализация WebApp
        showDiagnostic('Ожидание инициализации Telegram WebApp...');
        const webAppReady = await initializeTelegramWebApp();

        // Инициализируем Telegram WebApp
        let userId = null;
        let telegramUsername = null;
        
        showDiagnostic('=== ДИАГНОСТИКА TELEGRAM WEBAPP ===');
        showDiagnostic(`1. window.Telegram существует: ${!!window.Telegram}`);
        showDiagnostic(`2. window.Telegram.WebApp существует: ${!!(window.Telegram && window.Telegram.WebApp)}`);
        showDiagnostic(`3. User Agent: ${navigator.userAgent}`);
        showDiagnostic(`4. Текущий URL: ${window.location.href}`);
        showDiagnostic(`5. Referrer: ${document.referrer}`);
        showDiagnostic(`6. WebApp готов: ${webAppReady}`);
        
        // Дополнительные проверки
        showDiagnostic(`7. window.location.protocol: ${window.location.protocol}`);
        showDiagnostic(`8. window.location.hostname: ${window.location.hostname}`);
        showDiagnostic(`9. document.domain: ${document.domain}`);
        showDiagnostic(`10. window.parent === window: ${window.parent === window}`);
        
        // Проверяем доступность Telegram WebApp
        if (window.Telegram && window.Telegram.WebApp) {
            const tg = window.Telegram.WebApp;
            
            showDiagnostic('=== TELEGRAM WEBAPP ДАННЫЕ ===');
            showDiagnostic(`WebApp version: ${tg.version}`);
            showDiagnostic(`WebApp platform: ${tg.platform}`);
            showDiagnostic(`WebApp colorScheme: ${tg.colorScheme}`);
            showDiagnostic(`WebApp isExpanded: ${tg.isExpanded}`);
            showDiagnostic(`WebApp viewportHeight: ${tg.viewportHeight}`);
            showDiagnostic(`WebApp initData length: ${tg.initData ? tg.initData.length : 0}`);
            showDiagnostic(`WebApp initDataUnsafe: ${JSON.stringify(tg.initDataUnsafe, null, 2)}`);
            
            // Инициализируем WebApp
            tg.ready();
            showDiagnostic('WebApp.ready() вызван');
            
            // Расширяем WebApp на весь экран
            tg.expand();
            showDiagnostic('WebApp.expand() вызван');
            
            // Получаем данные пользователя
            if (tg.initDataUnsafe && tg.initDataUnsafe.user) {
                const user = tg.initDataUnsafe.user;
                userId = user.id;
                telegramUsername = user.username;
                
                showDiagnostic('=== ПОЛЬЗОВАТЕЛЬ НАЙДЕН ===');
                showDiagnostic(`ID: ${userId}`);
                showDiagnostic(`Username: ${telegramUsername}`);
                showDiagnostic(`First Name: ${user.first_name}`);
                showDiagnostic(`Last Name: ${user.last_name}`);
                showDiagnostic(`Language: ${user.language_code}`);
                showDiagnostic(`Is Premium: ${user.is_premium}`);
            } else {
                showDiagnostic('ДАННЫЕ ПОЛЬЗОВАТЕЛЯ НЕ НАЙДЕНЫ', true);
                showDiagnostic(`initDataUnsafe содержимое: ${JSON.stringify(tg.initDataUnsafe)}`, true);
                
                // Дополнительная проверка
                if (!tg.initData || tg.initData === '') {
                    showDiagnostic('initData пустой - WebApp запущен не из Telegram', true);
                } else {
                    showDiagnostic(`initData присутствует (${tg.initData.length} символов), но не распарсился`, true);
                    showDiagnostic(`Первые 100 символов initData: ${tg.initData.substring(0, 100)}`, true);
                }
            }
        } else {
            showDiagnostic('TELEGRAM WEBAPP НЕ ДОСТУПЕН', true);
            showDiagnostic('Возможные причины:', true);
            showDiagnostic('1. Сайт открыт не через Telegram WebApp', true);
            showDiagnostic('2. Скрипт telegram-web-app.js не загрузился', true);
            showDiagnostic('3. Неправильная настройка в BotFather', true);
            showDiagnostic('4. Конфликт между Menu Button и Mini App', true);
            showDiagnostic('5. Проблемы с HTTPS или CORS', true);
            
            // Проверяем загрузку скрипта
            const scripts = document.querySelectorAll('script[src*="telegram-web-app"]');
            showDiagnostic(`Telegram WebApp скрипт найден: ${scripts.length > 0}`, scripts.length === 0);
            if (scripts.length > 0) {
                showDiagnostic(`Скрипт URL: ${scripts[0].src}`);
            }
        }

        // Если не удалось получить user_id, показываем подробную ошибку
        if (!userId) {
            showDiagnostic('=== ОШИБКА: НЕ УДАЛОСЬ ПОЛУЧИТЬ USER_ID ===', true);
            showDiagnostic('ОСНОВНАЯ ПРОБЛЕМА: X-Frame-Options: DENY блокирует iframe', true);
            showDiagnostic('ЧТО ПРОВЕРИТЬ:', true);
            showDiagnostic('1. Обратитесь к хостинг-провайдеру для отключения X-Frame-Options', true);
            showDiagnostic('2. Или разрешите перезапись .htaccess', true);
            showDiagnostic('3. Проверьте что домен 24musoroff.ru доступен по HTTPS', true);
            showDiagnostic('4. Возможно нужны дополнительные настройки сервера', true);
            
            // ВРЕМЕННОЕ РЕШЕНИЕ для тестирования
            showDiagnostic('=== РЕЖИМ ТЕСТИРОВАНИЯ ===', true);
            showDiagnostic('Используем фиктивный user_id для проверки остальной логики', true);
            
            const useTestMode = confirm('Сервер блокирует Telegram WebApp (X-Frame-Options: DENY).\n\nИспользовать тестовый режим с фиктивным user_id для проверки остальной логики?\n\n⚠️ В продакшене это не будет работать!');
            
            if (useTestMode) {
                userId = 12345; // Фиктивный ID для тестирования
                telegramUsername = 'test_user';
                showDiagnostic('🧪 ТЕСТОВЫЙ РЕЖИМ АКТИВИРОВАН', true);
                showDiagnostic(`🧪 Используем тестовый user_id: ${userId}`, true);
                showDiagnostic('🧪 В продакшене нужно исправить X-Frame-Options!', true);
            } else {
                // Добавляем кнопку для закрытия диагностики
                setTimeout(() => {
                    const diagnosticDiv = document.getElementById('telegram-diagnostic');
                    if (diagnosticDiv) {
                        const closeBtn = document.createElement('button');
                        closeBtn.textContent = 'Закрыть диагностику';
                        closeBtn.style.cssText = 'margin-top: 10px; padding: 5px 10px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;';
                        closeBtn.onclick = () => diagnosticDiv.remove();
                        diagnosticDiv.appendChild(closeBtn);
                    }
                }, 1000);
                
                return;
            }
        }

        const orderData = {
            user_id: userId,
            delivery_address: deliveryAddress.trim(),
            telegram_username: telegramUsername
        };

        showDiagnostic('=== ОТПРАВКА ЗАКАЗА ===');
        showDiagnostic(`Order data: ${JSON.stringify(orderData, null, 2)}`);

        try {
            const response = await fetch('/api/orders', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(orderData)
            });

            if (response.ok) {
                const result = await response.json();
                showDiagnostic(`Заказ успешно создан: ${JSON.stringify(result)}`);
                alert(`Заказ №${result.order_id} оформлен! Проверьте Telegram для подтверждения и оплаты.`);
                await Promise.all([loadCartItems(), updateCartCounter()]);
                
                // Закрываем WebApp после успешного заказа
                if (window.Telegram && window.Telegram.WebApp) {
                    showDiagnostic('Закрытие WebApp...');
                    setTimeout(() => {
                        window.Telegram.WebApp.close();
                    }, 2000); // Даем время прочитать диагностику
                }
            } else {
                const error = await response.json();
                showDiagnostic(`Ошибка сервера: ${JSON.stringify(error)}`, true);
                showError(error.error || 'Ошибка оформления заказа');
            }
        } catch (error) {
            showDiagnostic(`Ошибка сети: ${error.message}`, true);
            showError('Ошибка оформления заказа');
        }
    }

    // Функция для показа диагностической информации на странице
    function showDiagnostic(message, isError = false) {
        let diagnosticDiv = document.getElementById('telegram-diagnostic');
        if (!diagnosticDiv) {
            diagnosticDiv = document.createElement('div');
            diagnosticDiv.id = 'telegram-diagnostic';
            diagnosticDiv.style.cssText = `
                position: fixed;
                top: 10px;
                left: 10px;
                right: 10px;
                background: ${isError ? '#ffebee' : '#e8f5e8'};
                border: 2px solid ${isError ? '#f44336' : '#4caf50'};
                border-radius: 8px;
                padding: 10px;
                font-family: monospace;
                font-size: 12px;
                z-index: 10000;
                max-height: 300px;
                overflow-y: auto;
                white-space: pre-wrap;
            `;
            document.body.appendChild(diagnosticDiv);
        }
        diagnosticDiv.innerHTML += (isError ? '❌ ' : '✅ ') + message + '\n';
        diagnosticDiv.scrollTop = diagnosticDiv.scrollHeight;
    }

    // Показать ошибку
    function showError(message) {
        const errorElement = document.createElement('div');
        errorElement.className = 'error-message';
        errorElement.textContent = message;

        const cartContainer = document.getElementById('cartItems');
        if (cartContainer) {
            cartContainer.prepend(errorElement);
            setTimeout(() => errorElement.remove(), 5000);
        }
    }
});