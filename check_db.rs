use sqlx::sqlite::SqlitePool;
use sqlx::Row;  // Add this import
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = SqlitePool::connect("sqlite:database/shop.db").await?;
    
    // Check if tables exist
    let tables = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )
    .fetch_all(&pool)
    .await?;
    
    println!("Tables in database:");
    for table in tables {
        let name: String = table.get(0);
        println!("- {}", name);
        
        // Get table schema
        let schema = sqlx::query(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name = ?"
        )
        .bind(&name)
        .fetch_one(&pool)
        .await;
        
        if let Ok(row) = schema {
            if let Some(sql) = row.get::<Option<String>, _>(0) {
                println!("  Schema: {}", sql);
            }
        }
    }
    
    Ok(())
}
