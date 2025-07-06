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

// Добавьте в admin.js
async function loadCategories() {
    const response = await fetch('/api/categories');
    const categories = await response.json();
    const list = document.getElementById('categories-list');
    list.innerHTML = categories.map(cat => `
        <div class="category-item">
            <h3>${cat.name}</h3>
            <p>${cat.description || ''}</p>
            <button onclick="deleteCategory(${cat.id})">Удалить</button>
        </div>
    `).join('');
}

async function deleteCategory(id) {
    const response = await fetch(`/api/categories/${id}`, { method: 'DELETE' });
    const success = await response.json();
    if (success) {
        loadCategories();
    } else {
        alert('Нельзя удалить категорию с товарами!');
    }
}

document.getElementById('category-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const category = {
        name: document.getElementById('category-name').value,
        description: document.getElementById('category-description').value,
        image_url: document.getElementById('category-image').value
    };
    await fetch('/api/categories', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(category)
    });
    loadCategories();
    e.target.reset();
});

// Вызовите loadCategories() при загрузке страницы