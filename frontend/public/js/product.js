document.addEventListener('DOMContentLoaded', function() {
    // Получаем ID продукта из URL
    const urlParams = new URLSearchParams(window.location.search);
    const productId = urlParams.get('id');

    // Или альтернативный вариант - из data-атрибута
    // const productId = document.getElementById('add-to-cart').dataset.productId;

    // Проверяем, что ID получен
    if (!productId) {
        console.error('Product ID not found');
        return;
    }

    // Обработчик добавления в корзину
    document.getElementById('add-to-cart').addEventListener('click', async () => {
        try {
            const response = await fetch('/api/cart', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json'
                },
                body: JSON.stringify({
                    product_id: parseInt(productId),
                    quantity: 1
                })
            });

            if (response.ok) {
                alert('Товар добавлен в корзину!');
                updateCartCounter();
            } else {
                const error = await response.json();
                alert('Ошибка: ' + error);
            }
        } catch (error) {
            console.error('Error:', error);
            alert('Произошла ошибка при добавлении в корзину');
        }
    });

    // Функция обновления счетчика корзины
    async function updateCartCounter() {
        try {
            const response = await fetch('/api/cart/count');
            if (response.ok) {
                const count = await response.json();
                const counter = document.getElementById('cart-counter');
                if (counter) {
                    counter.textContent = count;
                    counter.style.display = count > 0 ? 'block' : 'none';
                }
            }
        } catch (error) {
            console.error('Failed to update cart counter:', error);
        }
    }

    // Инициализируем счетчик при загрузке страницы
    updateCartCounter();
});