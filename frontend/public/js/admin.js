document.addEventListener('DOMContentLoaded', function() {
    loadProducts();
    loadCategories();
    loadCategoriesForSelect();

    // Добавляем обработчик кнопки
    document.getElementById('saveProductBtn').addEventListener('click', addProduct);
});

async function loadProducts() {
    try {
        const response = await fetch('/api/products');
        if (!response.ok) throw new Error('Ошибка сервера');
        const products = await response.json();

        const tbody = document.querySelector('#productsTable tbody');
        tbody.innerHTML = '';

        products.forEach(product => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${product.id}</td>
                <td>${product.name}</td>
                <td>${product.description || '-'}</td>
                <td>${product.price.toFixed(2)}</td>
                <td>${product.stock}</td>
                <td>${product.category_id || '-'}</td>
                <td>
                    <button class="btn-edit" onclick="editProduct(${product.id})">✏️</button>
                    <button class="btn-danger" onclick="deleteProduct(${product.id})">🗑️</button>
                </td>
            `;
            tbody.appendChild(row);
        });
    } catch (error) {
        console.error('Ошибка загрузки товаров:', error);
        alert('Ошибка загрузки товаров');
    }
}

async function addProduct() {
    const product = {
        name: document.getElementById('productName').value.trim(),
        description: document.getElementById('productDescription').value.trim(),
        price: parseFloat(document.getElementById('productPrice').value),
        stock: parseInt(document.getElementById('productStock').value),
        image_url: document.getElementById('productImage').value.trim() || null,
        category_id: document.getElementById('productCategory').value
            ? parseInt(document.getElementById('productCategory').value)
            : null
    };

    // Валидация
    if (!product.name || !product.description || isNaN(product.price) || isNaN(product.stock)) {
        alert('Пожалуйста, заполните все обязательные поля');
        return;
    }

    try {
        const response = await fetch('/api/products', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(product)
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Ошибка сервера');
        }

        resetProductForm();
        loadProducts();
        alert('Товар успешно добавлен!');
    } catch (error) {
        console.error('Ошибка при добавлении товара:', error);
        alert(`Ошибка: ${error.message}`);
    }
}

function resetProductForm() {
    document.getElementById('productName').value = '';
    document.getElementById('productDescription').value = '';
    document.getElementById('productPrice').value = '';
    document.getElementById('productStock').value = '';
    document.getElementById('productImage').value = '';
    document.getElementById('productCategory').value = '';
}

async function deleteProduct(id) {
    if (confirm('Вы уверены, что хотите удалить этот товар?')) {
        try {
            const response = await fetch(`/api/products/${id}`, { method: 'DELETE' });
            if (!response.ok) throw new Error('Ошибка сервера');
            loadProducts();
            alert('Товар успешно удален!');
        } catch (error) {
            console.error('Ошибка удаления товара:', error);
            alert('Ошибка удаления товара');
        }
    }
}

async function loadCategories() {
    try {
        const response = await fetch('/api/categories');
        if (!response.ok) throw new Error('Ошибка сервера');
        const categories = await response.json();

        const tbody = document.querySelector('#categoriesTable tbody');
        tbody.innerHTML = '';

        categories.forEach(category => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${category.id}</td>
                <td>${category.name}</td>
                <td>${category.description || '-'}</td>
                <td>
                    <button class="btn-edit" onclick="editCategory(${category.id})">✏️</button>
                    <button class="btn-danger" onclick="deleteCategory(${category.id})">🗑️</button>
                </td>
            `;
            tbody.appendChild(row);
        });
    } catch (error) {
        console.error('Ошибка загрузки категорий:', error);
        alert('Ошибка загрузки категорий');
    }
}

async function loadCategoriesForSelect() {
    try {
        const response = await fetch('/api/categories');
        const categories = await response.json();
        const select = document.getElementById('productCategory');

        select.innerHTML = '<option value="">Без категории</option>';

        categories.forEach(category => {
            const option = document.createElement('option');
            option.value = category.id;
            option.textContent = category.name;
            select.appendChild(option);
        });
    } catch (error) {
        console.error('Ошибка загрузки категорий:', error);
    }
}

async function deleteCategory(id) {
    if (confirm('Вы уверены, что хотите удалить эту категорию?')) {
        try {
            const response = await fetch(`/api/categories/${id}`, { method: 'DELETE' });
            const result = await response.json();

            if (response.ok) {
                loadCategories();
                loadCategoriesForSelect();
                alert('Категория удалена!');
            } else {
                throw new Error(result.error || 'Нельзя удалить категорию с товарами');
            }
        } catch (error) {
            console.error('Ошибка удаления категории:', error);
            alert(error.message);
        }
    }
}

// Переключение вкладок
document.querySelectorAll('.admin-tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.admin-tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

        tab.classList.add('active');
        document.getElementById(`${tab.dataset.tab}Tab`).classList.add('active');

        if (tab.dataset.tab === 'categories') {
            loadCategories();
        } else {
            loadProducts();
        }
    });
});