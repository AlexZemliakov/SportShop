document.addEventListener('DOMContentLoaded', function() {
    const urlParams = new URLSearchParams(window.location.search);
    const productId = urlParams.get('id');

    // Функция обновления счетчика корзины
    async function updateCartCounter() {
        try {
            const response = await fetch('/api/cart/count');
            if (response.ok) {
                const count = await response.json();
                const counter = document.getElementById('cart-counter');
                counter.textContent = count;
                counter.style.display = count > 0 ? 'flex' : 'none';
            }
        } catch (error) {
            console.error('Ошибка обновления счетчика:', error);
        }
    }

    // Обработчик добавления в корзину
    document.addEventListener('click', async (e) => {
        if (e.target.classList.contains('btn-cart')) {
            try {
                const response = await fetch('/api/cart', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        product_id: parseInt(productId),
                        quantity: 1
                    })
                });

                if (response.ok) {
                    alert('Товар добавлен в корзину!');
                    updateCartCounter();
                } else {
                    const error = await response.text();
                    alert('Ошибка: ' + error);
                }
            } catch (error) {
                console.error('Ошибка:', error);
                alert('Произошла ошибка при добавлении в корзину');
            }
        }
    });

    // Инициализация счетчика при загрузке
    updateCartCounter();
});