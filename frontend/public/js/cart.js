document.addEventListener('DOMContentLoaded', function() {
    loadCartItems();

    // Обновление счетчика корзины
    async function updateCartCounter() {
        const response = await fetch('/api/cart/count');
        if (response.ok) {
            const count = await response.json();
            const counter = document.getElementById('cart-counter');
            if (counter) {
                counter.textContent = count;
                counter.style.display = count > 0 ? 'flex' : 'none';
            }
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
                    <div class="cart-item-price">${item.price.toFixed(2)} ₽</div>
                    <div class="cart-item-quantity">
                        <button class="quantity-btn minus">-</button>
                        <input type="text" class="quantity-input" value="${item.quantity}">
                        <button class="quantity-btn plus">+</button>
                        <span class="remove-item">Удалить</span>
                    </div>
                </div>
            </div>
        `).join('');

        // Добавляем обработчики событий
        document.querySelectorAll('.quantity-btn.minus').forEach(btn => {
            btn.addEventListener('click', decreaseQuantity);
        });

        document.querySelectorAll('.quantity-btn.plus').forEach(btn => {
            btn.addEventListener('click', increaseQuantity);
        });

        document.querySelectorAll('.remove-item').forEach(btn => {
            btn.addEventListener('click', removeItem);
        });
    }

    // Уменьшение количества
    async function decreaseQuantity(e) {
        const itemElement = e.target.closest('.cart-item');
        const itemId = itemElement.dataset.id;
        const input = itemElement.querySelector('.quantity-input');
        let quantity = parseInt(input.value) - 1;

        if (quantity < 1) quantity = 1;

        await updateCartItem(itemId, quantity);
    }

    // Увеличение количества
    async function increaseQuantity(e) {
        const itemElement = e.target.closest('.cart-item');
        const itemId = itemElement.dataset.id;
        const input = itemElement.querySelector('.quantity-input');
        const quantity = parseInt(input.value) + 1;

        await updateCartItem(itemId, quantity);
    }

    // Обновление количества
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
        }
    }

    // Удаление товара
    async function removeItem(e) {
        const itemElement = e.target.closest('.cart-item');
        const itemId = itemElement.dataset.id;

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
        }
    }

    // Подсчет общей суммы
    function calculateTotal(items) {
        const total = items.reduce((sum, item) => sum + (item.price * item.quantity), 0);
        document.getElementById('cartTotal').textContent = total.toFixed(2);
    }

    // Оформление заказа
    document.getElementById('checkoutBtn')?.addEventListener('click', () => {
        alert('Заказ оформлен!');
        // Здесь можно добавить реальное оформление заказа
    });
});