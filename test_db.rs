use sqlx::{sqlite::SqlitePool, Row};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to database...");
    let pool = SqlitePool::connect("sqlite:database/shop.db").await?;
    println!("Successfully connected to database!");
    
    // Test categories table
    println!("\nTesting categories table...");
    match sqlx::query("SELECT COUNT(*) as count FROM categories")
        .fetch_one(&pool)
        .await {
            Ok(row) => {
                let count: i64 = row.get("count");
                println!("Found {} categories in the database", count);
            },
            Err(e) => println!("Error querying categories: {}", e)
        }
    
    // Test products table
    println!("\nTesting products table...");
    match sqlx::query("SELECT COUNT(*) as count FROM products")
        .fetch_one(&pool)
        .await {
            Ok(row) => {
                let count: i64 = row.get("count");
                println!("Found {} products in the database", count);
            },
            Err(e) => println!("Error querying products: {}", e)
        }
    
    // Test if we can insert a test category
    println!("\nTesting category insertion...");
    match sqlx::query(
        "INSERT OR IGNORE INTO categories (name, description) VALUES (?, ?) RETURNING id"
    )
    .bind("Test Category")
    .bind("Test Description")
    .fetch_one(&pool)
    .await {
        Ok(row) => {
            let id: i64 = row.get("id");
            println!("Successfully inserted test category with id: {}", id);
        },
        Err(e) => println!("Error inserting test category: {}", e)
    }
    
    Ok(())
}
