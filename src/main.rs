use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[derive(Serialize)]
struct Hello {
    message: &'static str,
}

async fn hello() -> Json<Hello> {
    Json(Hello { message: "Hello, world!" })
}

#[derive(Serialize)]
struct DbConnectResponse {
    ok: bool,
    message: String,
}

#[derive(Deserialize, Default)]
struct DbConnectParams {
    host: Option<String>,
    port: Option<u16>,
    user: Option<String>,
    password: Option<String>,
    db: Option<String>,
}

/// GET /db-connect: tries to connect to SQL Server using env variables and returns a JSON status
/// Required env vars:
/// - SQLSERVER_HOST (e.g., "localhost")
/// - SQLSERVER_PORT (e.g., "1433")
/// - SQLSERVER_USER
/// - SQLSERVER_PASSWORD
/// Optional:
/// - SQLSERVER_DB (database name)
async fn db_connect(Query(params): Query<DbConnectParams>) -> (StatusCode, Json<DbConnectResponse>) {
    match try_db_connect(params).await {
        Ok(msg) => (
            StatusCode::OK,
            Json(DbConnectResponse { ok: true, message: msg }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DbConnectResponse { ok: false, message: e }),
        ),
    }
}

async fn try_db_connect(params: DbConnectParams) -> Result<String, String> {
    // Prefer query params; fall back to env vars
    let host = params
        .host
        .or_else(|| std::env::var("SQLSERVER_HOST").ok())
        .ok_or_else(|| "SQLSERVER_HOST not set".to_string())?;
    let port: u16 = params
        .port
        .or_else(|| std::env::var("SQLSERVER_PORT").ok().and_then(|p| p.parse().ok()))
        .ok_or_else(|| "SQLSERVER_PORT not set or invalid".to_string())?;
    let user = params
        .user
        .or_else(|| std::env::var("SQLSERVER_USER").ok())
        .ok_or_else(|| "SQLSERVER_USER not set".to_string())?;
    let password = params
        .password
        .or_else(|| std::env::var("SQLSERVER_PASSWORD").ok())
        .ok_or_else(|| "SQLSERVER_PASSWORD not set".to_string())?;
    let database = params.db.or_else(|| std::env::var("SQLSERVER_DB").ok());

    // Build config
    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.authentication(tiberius::AuthMethod::sql_server(user, password));
    if let Some(db) = database { config.database(db); }
    // For local/dev ease. For production, validate server cert instead of trusting.
    config.trust_cert();

    // Connect
    let addr = config.get_addr();
    let tcp = TcpStream::connect(addr).await.map_err(|e| format!("tcp connect failed: {e}"))?;
    tcp.set_nodelay(true).map_err(|e| format!("set_nodelay failed: {e}"))?;
    let _client: Client<_> = Client::connect(config, tcp.compat_write())
        .await
        .map_err(|e| format!("tiberius connect failed: {e}"))?;

    Ok("Successfully connected to SQL Server".to_string())
}

#[tokio::main]
async fn main() {
  
    tracing_subscriber::fmt::init();


    let app = Router::new()
        .route("/hello", get(hello))
        .route("/db-connect", get(db_connect));


    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}
