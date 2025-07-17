use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    let database_url = "sqlite:database/shop.db";
    println!("Connecting to database at: {}", database_url);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
        
    println!("Successfully connected to database!");
    
    // Check if products table exists
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='products')"
    )
    .fetch_one(&pool)
    .await?;
    
    if !table_exists {
        println!("Error: 'products' table does not exist!");
        return Ok(());
    }
    
    println!("\nProducts table exists. Checking for products...");
    
    // Get count of products
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM products")
        .fetch_one(&pool)
        .await?;
        
    println!("Found {} products in the database.", count);
    
    if count > 0 {
        // Get first few products
        let products = sqlx::query(
            "SELECT id, name, price, stock FROM products LIMIT 5"
        )
        .fetch_all(&pool)
        .await?;
        
        println!("\nFirst few products:");
        for product in products {
            let id: i64 = product.get(0);
            let name: String = product.get(1);
            let price: f64 = product.get(2);
            let stock: i32 = product.get(3);
            println!("ID: {}, Name: {}, Price: {}, Stock: {}", id, name, price, stock);
        }
    }
    
    Ok(())
}
