document.addEventListener('DOMContentLoaded', loadProducts);

async function loadProducts() {
    const response = await fetch('/api/products');
    const products = await response.json();

    const tbody = document.querySelector('#productsTable tbody');
    tbody.innerHTML = '';

    products.forEach(product => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${product.id}</td>
            <td>${product.name}</td>
            <td>${product.price}</td>
            <td>${product.stock}</td>
            <td>
                <button onclick="deleteProduct(${product.id})">Удалить</button>
            </td>
        `;
        tbody.appendChild(row);
    });
}

async function addProduct() {
    const name = document.getElementById('productName').value;
    const price = parseFloat(document.getElementById('productPrice').value);
    const stock = parseInt(document.getElementById('productStock').value);

    await fetch('/api/products', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, price, stock })
    });

    loadProducts();
}

async function deleteProduct(id) {
    await fetch(`/api/products/${id}`, { method: 'DELETE' });
    loadProducts();
}