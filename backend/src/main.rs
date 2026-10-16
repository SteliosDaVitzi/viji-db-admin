#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::var("WEBDB_BIND").unwrap_or_else(|_| "127.0.0.1:34617".into());
    let secure = std::env::var("WEBDB_SECURE_COOKIES").as_deref() == Ok("true");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("WebDB API listening on {address}");
    axum::serve(listener, webdb::router(secure))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
