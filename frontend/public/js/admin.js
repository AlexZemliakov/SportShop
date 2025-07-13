// Инициализация при загрузке страницы
document.addEventListener('DOMContentLoaded', function() {
    loadCategoriesForSelect();
    loadProducts();
    loadCategories();

    // Обработчики событий
    document.getElementById('saveProductBtn').addEventListener('click', saveProduct);
    document.getElementById('cancelEditBtn').addEventListener('click', resetProductForm);
    document.getElementById('saveCategoryBtn').addEventListener('click', saveCategory);
    document.getElementById('cancelEditCategoryBtn').addEventListener('click', resetCategoryForm);
    document.getElementById('productCategoryFilter').addEventListener('change', filterProductsByCategory);

    // Обработчики вкладок
    document.querySelectorAll('.admin-tab').forEach(tab => {
        tab.addEventListener('click', switchTab);
    });
});

// ========== ОБЩИЕ ФУНКЦИИ ==========

function showSuccess(message) {
    const toast = document.createElement('div');
    toast.className = 'toast success';
    toast.textContent = message;
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

function showError(message) {
    const toast = document.createElement('div');
    toast.className = 'toast error';
    toast.textContent = message;
    document.body.appendChild(toast);
    setTimeout(() => toast.remove(), 3000);
}

// ========== РАБОТА С ВКЛАДКАМИ ==========

function switchTab(event) {
    const tab = event.currentTarget;
    document.querySelectorAll('.admin-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));

    tab.classList.add('active');
    document.getElementById(`${tab.dataset.tab}Tab`).classList.add('active');

    if (tab.dataset.tab === 'categories') {
        loadCategories();
    } else {
        loadProducts();
    }
}

// ========== РАБОТА С ТОВАРАМИ ==========

async function loadProducts(categoryId = null) {
    try {
        const url = categoryId && categoryId !== 'all'
            ? `/api/categories/${categoryId}/products`
            : '/api/products';

        const response = await fetch(url);
        if (!response.ok) throw new Error('Ошибка сервера');
        const products = await response.json();

        renderProductsTable(products);
    } catch (error) {
        console.error('Ошибка загрузки товаров:', error);
        showError('Ошибка загрузки товаров');
    }
}

function renderProductsTable(products) {
    const tbody = document.querySelector('#productsTable tbody');
    tbody.innerHTML = '';

    if (products.length === 0) {
        tbody.innerHTML = '<tr><td colspan="7" class="empty">Товары отсутствуют</td></tr>';
        return;
    }

    products.forEach(product => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${product.id}</td>
            <td>${product.name}</td>
            <td>${product.description || '-'}</td>
            <td>${product.price.toFixed(2)} ₽</td>
            <td>${product.stock}</td>
            <td>${getCategoryName(product.category_id)}</td>
            <td>
                <button class="btn btn-edit" onclick="editProduct(${product.id})">✏️</button>
                <button class="btn btn-danger" onclick="deleteProduct(${product.id})">🗑️</button>
            </td>
        `;
        tbody.appendChild(row);
    });
}

async function saveProduct() {
    const productId = document.getElementById('editProductId').value;
    const categorySelect = document.getElementById('productCategory');
    const categoryId = categorySelect.value ? parseInt(categorySelect.value) : null;

    const product = {
        name: document.getElementById('productName').value.trim(),
        description: document.getElementById('productDescription').value.trim(),
        price: parseFloat(document.getElementById('productPrice').value),
        stock: parseInt(document.getElementById('productStock').value),
        image_url: document.getElementById('productImage').value.trim() || null,
        category_id: categoryId
    };

    // Валидация
    if (!validateProduct(product)) return;

    try {
        const url = productId ? `/api/products/${productId}` : '/api/products';
        const method = productId ? 'PUT' : 'POST';

        const response = await fetch(url, {
            method: method,
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(product)
        });

        if (!response.ok) {
            const errorData = await response.json();
            throw new Error(errorData || 'Ошибка сервера');
        }

        resetProductForm();
        loadProducts(document.getElementById('productCategoryFilter').value);
        showSuccess(productId ? 'Товар обновлен' : 'Товар добавлен');
    } catch (error) {
        console.error('Ошибка сохранения товара:', error);
        showError(`Ошибка сохранения товара: ${error.message}`);
    }
}

function resetProductForm() {
    document.getElementById('editProductId').value = '';
    document.getElementById('productName').value = '';
    document.getElementById('productDescription').value = '';
    document.getElementById('productPrice').value = '';
    document.getElementById('productStock').value = '';
    document.getElementById('productImage').value = '';
    document.getElementById('productCategory').value = '';
    document.getElementById('saveProductBtn').textContent = 'Добавить товар';
    document.getElementById('cancelEditBtn').style.display = 'none';
}

async function editProduct(id) {
    try {
        const response = await fetch(`/api/products/${id}`);
        if (!response.ok) throw new Error('Ошибка сервера');
        const product = await response.json();

        document.getElementById('editProductId').value = product.id;
        document.getElementById('productName').value = product.name;
        document.getElementById('productDescription').value = product.description || '';
        document.getElementById('productPrice').value = product.price;
        document.getElementById('productStock').value = product.stock;
        document.getElementById('productImage').value = product.image_url || '';
        document.getElementById('productCategory').value = product.category_id || '';
        document.getElementById('saveProductBtn').textContent = 'Сохранить изменения';
        document.getElementById('cancelEditBtn').style.display = 'inline-block';
    } catch (error) {
        console.error('Ошибка загрузки товара:', error);
        showError('Ошибка загрузки товара');
    }
}

async function deleteProduct(productId) {
    if (!confirm('Вы уверены, что хотите удалить этот товар?')) return;

    try {
        const response = await fetch(`/api/products/${productId}`, {
            method: 'DELETE',
            headers: {
                'Accept': 'application/json',
            }
        });

        console.log('Delete response:', response); // Добавьте логирование

        if (response.status === 204) {
            showSuccess('Товар успешно удален');
            loadProducts();
        } else {
            const errorText = await response.text();
            console.error('Delete error:', errorText);
            throw new Error(errorText || 'Ошибка сервера');
        }
    } catch (error) {
        console.error('Ошибка удаления товара:', error);
        showError(error.message);
    }
}

function filterProductsByCategory() {
    const categoryId = document.getElementById('productCategoryFilter').value;
    loadProducts(categoryId === 'all' ? null : categoryId);
}

// ========== РАБОТА С КАТЕГОРИЯМИ ==========

async function loadCategoriesForSelect() {
    try {
        const response = await fetch('/api/categories');
        const categories = await response.json();

        const productCategorySelect = document.getElementById('productCategory');
        const filterCategorySelect = document.getElementById('productCategoryFilter');

        // Очищаем и добавляем опции
        productCategorySelect.innerHTML = '<option value="">Без категории</option>';
        filterCategorySelect.innerHTML = '<option value="all">Все категории</option>';

        categories.forEach(category => {
            const option = document.createElement('option');
            option.value = category.id;
            option.textContent = category.name;
            productCategorySelect.appendChild(option.cloneNode(true));
            filterCategorySelect.appendChild(option);
        });
    } catch (error) {
        console.error('Ошибка загрузки категорий:', error);
        showError('Не удалось загрузить категории');
    }
}

async function loadCategories() {
    try {
        const response = await fetch('/api/categories');
        if (!response.ok) throw new Error('Ошибка сервера');
        const categories = await response.json();

        renderCategoriesTable(categories);
    } catch (error) {
        console.error('Ошибка загрузки категорий:', error);
        showError('Ошибка загрузки категорий');
    }
}

function renderCategoriesTable(categories) {
    const tbody = document.querySelector('#categoriesTable tbody');
    tbody.innerHTML = '';

    if (categories.length === 0) {
        tbody.innerHTML = '<tr><td colspan="4" class="empty">Категории отсутствуют</td></tr>';
        return;
    }

    categories.forEach(category => {
        const row = document.createElement('tr');
        row.innerHTML = `
            <td>${category.id}</td>
            <td>${category.name}</td>
            <td>${category.description || '-'}</td>
            <td>
                <button class="btn btn-edit" onclick="editCategory(${category.id})">✏️</button>
                <button class="btn btn-danger" onclick="deleteCategory(${category.id})">🗑️</button>
            </td>
        `;
        tbody.appendChild(row);
    });
}

async function saveCategory() {
    const categoryId = document.getElementById('editCategoryId').value;
    const category = {
        name: document.getElementById('categoryName').value.trim(),
        description: document.getElementById('categoryDescription').value.trim() || null,
        image_url: document.getElementById('categoryImage').value.trim() || null
    };

    // Валидация
    if (!category.name) {
        showError('Пожалуйста, укажите название категории');
        return;
    }

    const url = categoryId ? `/api/categories/${categoryId}` : '/api/categories';
    const method = categoryId ? 'PUT' : 'POST';

    try {
        const response = await fetch(url, {
            method: method,
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(category)
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Ошибка сервера');
        }

        resetCategoryForm();
        loadCategories();
        loadCategoriesForSelect();
        showSuccess(categoryId ? 'Категория обновлена' : 'Категория добавлена');
    } catch (error) {
        console.error('Ошибка сохранения категории:', error);
        showError(error.message || 'Ошибка сохранения категории');
    }
}

function resetCategoryForm() {
    document.getElementById('editCategoryId').value = '';
    document.getElementById('categoryName').value = '';
    document.getElementById('categoryDescription').value = '';
    document.getElementById('categoryImage').value = '';
    document.getElementById('saveCategoryBtn').textContent = 'Добавить категорию';
    document.getElementById('cancelEditCategoryBtn').style.display = 'none';
}

async function editCategory(id) {
    try {
        const response = await fetch(`/api/categories/${id}`);
        if (!response.ok) throw new Error('Ошибка сервера');
        const category = await response.json();

        document.getElementById('editCategoryId').value = category.id;
        document.getElementById('categoryName').value = category.name;
        document.getElementById('categoryDescription').value = category.description || '';
        document.getElementById('categoryImage').value = category.image_url || '';
        document.getElementById('saveCategoryBtn').textContent = 'Сохранить изменения';
        document.getElementById('cancelEditCategoryBtn').style.display = 'inline-block';
    } catch (error) {
        console.error('Ошибка загрузки категории:', error);
        showError('Ошибка загрузки категории');
    }
}

async function deleteCategory(id) {
    if (confirm('Вы уверены, что хотите удалить эту категорию?')) {
        try {
            const response = await fetch(`/api/categories/${id}`, { method: 'DELETE' });
            if (!response.ok) {
                const error = await response.json();
                throw new Error(error.message || 'Ошибка сервера');
            }
            loadCategories();
            loadCategoriesForSelect();
            showSuccess('Категория удалена');
        } catch (error) {
            console.error('Ошибка удаления категории:', error);
            showError(error.message || 'Нельзя удалить категорию с товарами');
        }
    }
}

// ========== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ==========

function validateProduct(product) {
    if (!product.name || !product.description) {
        showError('Пожалуйста, заполните название и описание товара');
        return false;
    }

    if (isNaN(product.price) || product.price <= 0) {
        showError('Цена должна быть положительным числом');
        return false;
    }

    if (isNaN(product.stock) || product.stock < 0) {
        showError('Количество должно быть неотрицательным числом');
        return false;
    }

    return true;
}

function getCategoryName(categoryId) {
    if (!categoryId) return '-';
    const select = document.getElementById('productCategory');
    const option = select.querySelector(`option[value="${categoryId}"]`);
    return option ? option.textContent : 'Неизвестно';
}

document.addEventListener('DOMContentLoaded', function() {
    // Обработчик клика на товар
    document.querySelectorAll('.product-item').forEach(item => {
        item.addEventListener('click', function() {
            const productId = this.dataset.productId;
            window.location.href = `/product.html?id=${productId}`;
        });
    });

    // Если это страница товара - загружаем данные
    if (window.location.pathname.includes('/product.html')) {
        loadProductData();
    }
});

async function loadProductData() {
    const urlParams = new URLSearchParams(window.location.search);
    const productId = urlParams.get('id');

    if (!productId) {
        window.location.href = '/';
        return;
    }

    try {
        const response = await fetch(`/api/product/${productId}`);
        if (!response.ok) throw new Error('Product not found');

        const product = await response.json();
        renderProductPage(product);
    } catch (error) {
        console.error('Error loading product:', error);
        window.location.href = '/';
    }
}

function renderProductPage(product) {
    document.getElementById('product-name').textContent = product.name;
    document.getElementById('product-price').textContent = `$${product.price.toFixed(2)}`;
    document.getElementById('product-description').textContent = product.description;
    document.getElementById('product-stock').textContent = `In stock: ${product.stock}`;

    const imageContainer = document.getElementById('product-image');
    if (product.image_url) {
        imageContainer.innerHTML = `<img src="${product.image_url}" alt="${product.name}" class="product-image">`;
    } else {
        imageContainer.innerHTML = '<div class="no-image">No image available</div>';
    }

    // Кнопка "Add to cart"
    document.getElementById('add-to-cart').addEventListener('click', () => {
        addToCart(product.id);
    });
}

async function addToCart(productId) {
    try {
        const response = await fetch('/api/cart/add', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ product_id: productId, quantity: 1 })
        });

        if (response.ok) {
            alert('Product added to cart!');
        } else {
            throw new Error('Failed to add to cart');
        }
    } catch (error) {
        console.error('Error adding to cart:', error);
        alert('Error adding product to cart');
    }
}

document.addEventListener('DOMContentLoaded', function() {
    // Обработчик клика на продукт
    document.querySelectorAll('.product').forEach(product => {
        product.addEventListener('click', function() {
            const productId = this.dataset.productId;
            window.location.href = `/product.html?id=${productId}`;
        });
    });
});