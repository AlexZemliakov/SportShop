document.addEventListener('DOMContentLoaded', function() {
    // Обновление счетчика корзины
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

    // Инициализация счетчика
    updateCartCounter();
});