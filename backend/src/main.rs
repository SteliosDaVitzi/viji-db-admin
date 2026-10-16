#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = std::env::var("WEBDB_BIND").unwrap_or_else(|_| "127.0.0.1:34617".into());
    let secure = std::env::var("WEBDB_SECURE_COOKIES").as_deref() == Ok("true");
    let connections: Option<std::sync::Arc<dyn webdb::application::ConnectionRepository>> =
        match std::env::var("WEBDB_VAULT_KEY") {
            Ok(value) => {
                let key: Vec<u8> = (0..value.len())
                    .step_by(2)
                    .map(|i| {
                        value
                            .get(i..i + 2)
                            .and_then(|s| u8::from_str_radix(s, 16).ok())
                    })
                    .collect::<Option<Vec<_>>>()
                    .ok_or("WEBDB_VAULT_KEY must contain 64 hexadecimal characters")?;
                let key: [u8; 32] = key
                    .try_into()
                    .map_err(|_| "WEBDB_VAULT_KEY must contain 64 hexadecimal characters")?;
                let path = std::env::var("WEBDB_STORE_PATH")
                    .map_err(|_| "Set WEBDB_STORE_PATH to a private writable SQLite file path")?;
                Some(std::sync::Arc::new(
                    webdb::adapters::vault::SqliteVault::open(&path, &key)
                        .await
                        .map_err(|_| "Unable to open the local connection store")?,
                ))
            }
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => return Err("Invalid WEBDB_VAULT_KEY configuration".into()),
        };
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("WebDB API listening on {address}");
    axum::serve(listener, webdb::router_with_repository(secure, connections))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
