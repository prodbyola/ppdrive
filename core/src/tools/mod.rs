use std::path::{Path, PathBuf};

use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce, aead::Aead};
use rbatis::RBatis;
use secrets::AppSecrets;
use uuid::Uuid;

use crate::{CoreResult, errors::CoreError, models::client::Clients};

pub mod secrets;

/// creates a new client and returns the client's key
pub async fn create_client(rb: &RBatis, secrets: &AppSecrets, name: &str) -> CoreResult<String> {
    let client_id = Uuid::new_v4();
    let client_id = client_id.to_string();

    let token = generate_token(secrets, &client_id)?;
    Clients::create(rb, client_id, name.to_string()).await?;
    Ok(token)
}

/// generate a token for client's id
fn generate_token(secrets: &AppSecrets, client_id: &str) -> CoreResult<String> {
    let key = secrets.secret_key();
    let nonce_key = secrets.nonce();

    let nonce = XNonce::from_slice(nonce_key);
    let cipher = XChaCha20Poly1305::new(key.into());

    let encrypt = cipher.encrypt(nonce, client_id.as_bytes())?;
    let encode = hex::encode(&encrypt);

    Ok(encode)
}

/// validate that a given client token exists
pub async fn verify_client(rb: &RBatis, secrets: &AppSecrets, token: &str) -> CoreResult<u64> {
    let decode =
        hex::decode(token).map_err(|err| CoreError::AuthorizationError(err.to_string()))?;

    let key = secrets.secret_key();
    let nonce_key = secrets.nonce();

    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = XNonce::from_slice(nonce_key);

    let decrypt = cipher.decrypt(nonce, decode.as_slice())?;

    let id =
        String::from_utf8(decrypt).map_err(|err| CoreError::AuthorizationError(err.to_string()))?;

    let client = Clients::get(rb, &id).await?;
    Ok(client.id())
}

/// Regenerate token for a given client
pub async fn regenerate_token(
    rb: &RBatis,
    secrets: &AppSecrets,
    client_id: &str,
) -> CoreResult<String> {
    let client = Clients::get(rb, client_id).await?;
    generate_token(secrets, client.pid())
}

pub fn install_dir() -> CoreResult<PathBuf> {
    let exec_path = std::env::current_exe()?;
    let path = exec_path.parent().ok_or(CoreError::ServerError(
        "unable to get install dir".to_string(),
    ))?;

    Ok(path.to_owned())
}

/// compute total size (in bytes) of a folder.
pub async fn check_folder_size(folder_path: &str, size: &mut u64) -> Result<(), CoreError> {
    let path = Path::new(folder_path);

    if path.is_file() {
        return Err(CoreError::ServerError(
            "provided path is not a folder path".to_string(),
        ));
    }

    let mut rd = tokio::fs::read_dir(path).await?;

    while let Ok(Some(entry)) = rd.next_entry().await {
        let path = entry.path();

        if path.is_file() {
            let m = path.metadata()?;
            *size += m.len()
        } else if let Some(folder) = path.to_str() {
            Box::pin(check_folder_size(folder, size)).await?;
        }
    }

    Ok(())
}
