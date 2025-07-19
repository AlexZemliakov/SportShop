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

    // Оформление заказа
    async function checkout() {
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

        // Инициализируем Telegram WebApp
        let userId = null;
        let telegramUsername = null;
        
        console.log('=== Telegram WebApp Debug Info ===');
        console.log('window.Telegram:', window.Telegram);
        console.log('window.Telegram.WebApp:', window.Telegram?.WebApp);
        
        // Проверяем доступность Telegram WebApp
        if (window.Telegram && window.Telegram.WebApp) {
            const tg = window.Telegram.WebApp;
            
            console.log('Telegram WebApp version:', tg.version);
            console.log('Telegram WebApp initData:', tg.initData);
            console.log('Telegram WebApp initDataUnsafe:', tg.initDataUnsafe);
            
            // Инициализируем WebApp
            tg.ready();
            
            // Получаем данные пользователя
            if (tg.initDataUnsafe && tg.initDataUnsafe.user) {
                const user = tg.initDataUnsafe.user;
                userId = user.id;
                telegramUsername = user.username;
                
                console.log('✅ Получен реальный user_id:', userId);
                console.log('✅ Username:', telegramUsername);
            } else {
                console.warn('⚠️ Данные пользователя не найдены в initDataUnsafe');
                console.log('Доступные данные:', tg.initDataUnsafe);
            }
        } else {
            console.error('❌ Telegram WebApp не доступен');
            console.log('Возможные причины:');
            console.log('1. WebApp запущен не из Telegram');
            console.log('2. Неправильная настройка в BotFather');
            console.log('3. Проблемы с загрузкой Telegram Web App API');
        }

        // Если не удалось получить user_id, показываем ошибку
        if (!userId) {
            alert('Ошибка: Не удалось получить данные пользователя из Telegram.\n\nПожалуйста, убедитесь что:\n1. Вы открыли WebApp из Telegram бота\n2. WebApp правильно настроен в BotFather');
            console.error('❌ Не удалось получить user_id из Telegram WebApp');
            return;
        }

        const orderData = {
            user_id: userId,
            delivery_address: deliveryAddress.trim(),
            telegram_username: telegramUsername
        };

        console.log('=== Отправка заказа ===');
        console.log('Order data:', orderData);

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
                alert(`Заказ №${result.order_id} оформлен! Проверьте Telegram для подтверждения и оплаты.`);
                await Promise.all([loadCartItems(), updateCartCounter()]);
                
                // Закрываем WebApp после успешного заказа
                if (window.Telegram && window.Telegram.WebApp) {
                    window.Telegram.WebApp.close();
                }
            } else {
                const error = await response.json();
                console.error('Ошибка сервера:', error);
                showError(error.error || 'Ошибка оформления заказа');
            }
        } catch (error) {
            console.error('Ошибка:', error);
            showError('Ошибка оформления заказа');
        }
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