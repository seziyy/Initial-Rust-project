use sqlx::{Pool, Error};
use tracing::info;

/// SQL Server veritabanı bağlantı pool'u oluşturur
/// Connection string DATABASE_URL environment variable'ından okunur
pub async fn create_pool() -> Result<Pool<sqlx::Mssql>, Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    info!("SQL Server bağlantısı kuruluyor: {}", mask_password(&database_url));
    
    let pool = Pool::<sqlx::Mssql>::connect(&database_url).await?;
    
    info!("SQL Server bağlantısı başarıyla kuruldu");
    
    Ok(pool)
}

/// Connection string'deki şifreyi gizler (loglama için)
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

/// Veritabanı bağlantısını test eder
pub async fn test_connection(pool: &Pool<sqlx::Mssql>) -> Result<(), Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    info!("Veritabanı bağlantısı test edildi ve başarılı");
    Ok(())
}

