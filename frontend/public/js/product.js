document.addEventListener('DOMContentLoaded', function() {
    const urlParams = new URLSearchParams(window.location.search);
    const productId = urlParams.get('id');

    if (productId) {
        fetch(`/products/${productId}`)
            .then(response => response.json())
            .then(product => {
                document.getElementById('product-name').textContent = product.name;
                document.getElementById('product-image').src = product.image_url;
                document.getElementById('product-price').textContent = `${product.price} P`;
                document.getElementById('product-stock').textContent = `Остаток: ${product.stock} шт.`;
                document.getElementById('product-description').textContent = product.description;
            })
            .catch(error => console.error('Error:', error));
    }
});