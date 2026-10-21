use crate::{application::*, domain::*};
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, AeadCore, OsRng, Payload},
};
use async_trait::async_trait;
use sqlx::{
    Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

pub struct SqliteVault {
    pool: SqlitePool,
    cipher: Aes256Gcm,
}
fn unavailable() -> AppError {
    AppError(
        ErrorKind::Internal,
        "Saved connections are unavailable. Check the local store and encryption key.",
    )
}
impl SqliteVault {
    pub async fn open(path: &str, key: &[u8; 32]) -> AppResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(true)
                    .pragma("secure_delete", "ON"),
            )
            .await
            .map_err(|_| unavailable())?;
        sqlx::query("CREATE TABLE IF NOT EXISTS saved_connections (id TEXT PRIMARY KEY, name TEXT NOT NULL, nonce BLOB NOT NULL, ciphertext BLOB NOT NULL)")
            .execute(&pool).await.map_err(|_| unavailable())?;
        Ok(Self {
            pool,
            cipher: Aes256Gcm::new_from_slice(key).map_err(|_| unavailable())?,
        })
    }
}
#[async_trait]
impl ConnectionRepository for SqliteVault {
    async fn list(&self) -> AppResult<Vec<SavedConnection>> {
        let rows = sqlx::query("SELECT id, name FROM saved_connections ORDER BY name, id")
            .fetch_all(&self.pool)
            .await
            .map_err(|_| unavailable())?;
        Ok(rows
            .iter()
            .map(|r| SavedConnection {
                id: r.get("id"),
                name: r.get("name"),
            })
            .collect())
    }
    async fn save(&self, name: &str, input: &ConnectionInput) -> AppResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let plaintext = serde_json::to_vec(input).map_err(|_| unavailable())?;
        let ciphertext = self
            .cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: &plaintext,
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| unavailable())?;
        sqlx::query("INSERT INTO saved_connections (id, name, nonce, ciphertext) SELECT ?, ?, ?, ? WHERE (SELECT COUNT(*) FROM saved_connections) < 100")
            .bind(&id).bind(name).bind(&nonce[..]).bind(ciphertext)
            .execute(&self.pool).await.map_err(|_| unavailable())?
            .rows_affected().eq(&1).then_some(id).ok_or_else(|| invalid("Remove a saved connection before adding more (limit 100)."))
    }
    async fn load(&self, id: &str) -> AppResult<ConnectionInput> {
        let row = sqlx::query("SELECT nonce, ciphertext FROM saved_connections WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| unavailable())?
            .ok_or(AppError(ErrorKind::NotFound, "Saved connection not found."))?;
        let nonce: Vec<u8> = row.get("nonce");
        let nonce: [u8; 12] = nonce.try_into().map_err(|_| unavailable())?;
        let ciphertext: Vec<u8> = row.get("ciphertext");
        let plaintext = self
            .cipher
            .decrypt(
                &Nonce::from(nonce),
                Payload {
                    msg: &ciphertext,
                    aad: id.as_bytes(),
                },
            )
            .map_err(|_| unavailable())?;
        serde_json::from_slice(&plaintext).map_err(|_| unavailable())
    }
    async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM saved_connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|_| unavailable())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn encrypted_roundtrip_tampering_and_delete() {
        let vault = SqliteVault::open(":memory:", &[7; 32]).await.unwrap();
        let input: ConnectionInput = serde_json::from_value(serde_json::json!({"provider":"postgres","host":"private-host","port":5432,"database":"","allDatabases":true,"username":"alice","password":"secret-password","ssl":true})).unwrap();
        vault.save("Work", &input).await.unwrap();
        let saved = vault.list().await.unwrap();
        assert_eq!(saved[0].name, "Work");
        let id = &saved[0].id;
        assert_eq!(vault.load(id).await.unwrap().password, "secret-password");
        let bytes: Vec<u8> = sqlx::query_scalar("SELECT ciphertext FROM saved_connections")
            .fetch_one(&vault.pool)
            .await
            .unwrap();
        assert!(!bytes.windows(15).any(|w| w == b"secret-password"));
        sqlx::query("UPDATE saved_connections SET id = 'tampered'")
            .execute(&vault.pool)
            .await
            .unwrap();
        assert!(vault.load("tampered").await.is_err());
        vault.delete("tampered").await.unwrap();
        assert!(vault.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn persists_across_restart_and_rejects_wrong_key() {
        let path = std::env::temp_dir().join(format!("webdb-test-{}.sqlite", uuid::Uuid::new_v4()));
        let path_str = path.to_str().unwrap();
        let vault = SqliteVault::open(path_str, &[3; 32]).await.unwrap();
        let input: ConnectionInput = serde_json::from_value(serde_json::json!({"provider":"postgres","host":"host","port":5432,"database":"app","username":"user","password":"test-secret","ssl":true})).unwrap();
        vault.save("Private", &input).await.unwrap();
        let id = vault.list().await.unwrap().remove(0).id;
        vault.pool.close().await;
        let bytes = std::fs::read(&path).unwrap();
        assert!(!bytes.windows(11).any(|w| w == b"test-secret"));
        let wrong = SqliteVault::open(path_str, &[4; 32]).await.unwrap();
        assert!(wrong.load(&id).await.is_err());
        wrong.pool.close().await;
        let restored = SqliteVault::open(path_str, &[3; 32]).await.unwrap();
        assert_eq!(restored.load(&id).await.unwrap().password, "test-secret");
        restored.delete(&id).await.unwrap();
        restored.pool.close().await;
        std::fs::remove_file(path).unwrap();
    }
}
