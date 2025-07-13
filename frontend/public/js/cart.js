document.addEventListener('DOMContentLoaded', function() {
    loadCartItems();
    updateCartCounter();

    // Обновление счетчика корзины
    async function updateCartCounter() {
        try {
            const response = await fetch('/api/cart/count');
            if (response.ok) {
                const count = await response.json();
                const counter = document.getElementById('cart-counter');
                if (counter) {
                    counter.textContent = count;
                    counter.style.display = count > 0 ? 'flex' : 'none';
                }
            }
        } catch (error) {
            console.error('Ошибка обновления счетчика:', error);
        }
    }

    // Загрузка товаров в корзине
    async function loadCartItems() {
        try {
            const response = await fetch('/api/cart');
            if (!response.ok) throw new Error('Ошибка загрузки корзины');
            const items = await response.json();

            renderCartItems(items);
            calculateTotal(items);
        } catch (error) {
            console.error('Ошибка:', error);
            document.getElementById('cartItems').innerHTML =
                '<div class="error">Ошибка загрузки корзины</div>';
        }
    }

    // Отображение товаров
    function renderCartItems(items) {
        const container = document.getElementById('cartItems');

        if (items.length === 0) {
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
                        <span class="remove-item">Удалить</span>
                    </div>
                </div>
            </div>
        `).join('');
    }

    // Обработчики событий
    document.addEventListener('click', async (e) => {
        const itemElement = e.target.closest('.cart-item');
        if (!itemElement) return;

        const itemId = itemElement.dataset.id;
        const input = itemElement.querySelector('.quantity-input');
        let quantity = parseInt(input.value);

        if (e.target.classList.contains('quantity-minus')) {
            quantity = Math.max(1, quantity - 1);
            await updateCartItem(itemId, quantity);
        }
        else if (e.target.classList.contains('quantity-plus')) {
            quantity++;
            await updateCartItem(itemId, quantity);
        }
        else if (e.target.classList.contains('remove-item')) {
            if (confirm('Удалить товар из корзины?')) {
                await removeItem(itemId);
            }
        }
    });

    // Обновление количества товара
    async function updateCartItem(itemId, quantity) {
        try {
            const response = await fetch(`/api/cart/${itemId}`, {
                method: 'PUT',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ quantity })
            });

            if (response.ok) {
                loadCartItems();
                updateCartCounter();
            }
        } catch (error) {
            console.error('Ошибка:', error);
            alert('Не удалось обновить количество');
        }
    }

    // Удаление товара
    async function removeItem(itemId) {
        try {
            const response = await fetch(`/api/cart/${itemId}`, {
                method: 'DELETE'
            });

            if (response.ok) {
                loadCartItems();
                updateCartCounter();
            }
        } catch (error) {
            console.error('Ошибка:', error);
            alert('Не удалось удалить товар');
        }
    }

    // Подсчет общей суммы
    function calculateTotal(items) {
        const total = items.reduce((sum, item) => sum + (item.price * item.quantity), 0);
        document.getElementById('cartTotal').textContent = total.toFixed(2);
    }

    // Оформление заказа
    document.getElementById('checkoutBtn')?.addEventListener('click', () => {
        if (document.getElementById('cartItems').children.length === 0) {
            alert('Корзина пуста!');
            return;
        }
        alert('Заказ оформлен! Спасибо за покупку!');
        // Здесь можно добавить реальное оформление заказа
    });
});