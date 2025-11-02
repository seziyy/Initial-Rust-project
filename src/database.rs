use sqlx::{Pool, Error};
use tracing::info;

pub async fn create_pool() -> Result<Pool<sqlx::Mssql>, Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    info!("Connecting to SQL Server: {}", mask_password(&database_url));
    
    let pool = Pool::<sqlx::Mssql>::connect(&database_url).await?;
    
    info!("SQL Server connected successfully");
    
    Ok(pool)
}

fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            let password_end = if let Some(slash_pos) = url[colon_pos..].find('/') {
                colon_pos + slash_pos
            } else {
                at_pos
            };
            masked.replace_range(colon_pos + 1..password_end, "***");
            return masked;
        }
    }
    url.to_string()
}

pub async fn test_connection(pool: &Pool<sqlx::Mssql>) -> Result<(), Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    info!("Test query executed successfully");
    Ok(())
}

