use crate::error::{Result, RetroError};
use russh::client;
use russh::keys;
use russh_sftp::client::SftpSession;
use std::sync::Arc;

pub struct RemoteSession {
    pub session: client::Handle<ClientHandler>,
    pub sftp: SftpSession,
}

pub struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKeyOrCertificate,
    ) -> std::result::Result<bool, Self::Error> {
        // Trust on first use (TOFU). Production should verify against stored fingerprint.
        Ok(true)
    }
}

pub async fn compute_hashes(
    sftp: &russh_sftp::client::SftpSession,
    path: &str,
    allowed: &[String],
) -> crate::identify::Hashes {
    use tokio::io::AsyncReadExt;
    let mut f = match sftp.open(path).await {
        Ok(f) => f,
        Err(_) => return (None, None, None, None, None),
    };
    let mut buf = Vec::new();
    // Timeout per file to avoid hanging on stalled connections.
    let read_result = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        f.read_to_end(&mut buf),
    )
    .await;
    match read_result {
        Ok(Ok(_)) => {}
        _ => return (None, None, None, None, None),
    }
    let lower = path.to_lowercase();
    let filename = std::path::Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path);
    if lower.ends_with(".7z") {
        let tmp = std::env::temp_dir().join(format!("retroverse_remote_{}.7z", path.len()));
        if std::fs::write(&tmp, &buf).is_ok() {
            let r = crate::identify::compute_hashes(tmp.to_str().unwrap_or(""), allowed);
            let _ = std::fs::remove_file(&tmp);
            return r;
        }
    }
    crate::identify::compute_hashes_from_bytes(&buf, filename, allowed)
}

pub fn resolve_key_path(key_path: &str) -> String {
    if !key_path.trim().is_empty() {
        return key_path.trim().to_string();
    }
    // try common default keys
    if let Some(home) = dirs::home_dir() {
        for name in ["id_ed25519", "id_rsa", "id_ecdsa", "id_dsa"] {
            let p = home.join(".ssh").join(name);
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
    }
    key_path.trim().to_string()
}

pub async fn connect(
    host: &str,
    port: u16,
    username: &str,
    key_path: &str,
) -> Result<RemoteSession> {
    let key_path = resolve_key_path(key_path);
    let config = Arc::new(client::Config::default());
    let sh = ClientHandler;
    let mut session = client::connect(config, (host, port), sh)
        .await
        .map_err(|e| RetroError::Other(format!("ssh connect failed: {}", e)))?;

    // authenticate with key
    let key = keys::load_secret_key(&key_path, None)
        .map_err(|e| RetroError::Other(format!("load key failed ({}): {}", key_path, e)))?;
    let hash = session
        .best_supported_rsa_hash()
        .await
        .map_err(|e| RetroError::Other(e.to_string()))?
        .flatten();
    let auth = session
        .authenticate_publickey(
            username,
            russh::keys::PrivateKeyWithHashAlg::new(Arc::new(key), hash),
        )
        .await
        .map_err(|e| RetroError::Other(format!("auth failed: {}", e)))?;
    if !auth.success() {
        return Err(RetroError::Other("authentication failed".into()));
    }

    // open sftp subsystem
    let channel = session
        .channel_open_session()
        .await
        .map_err(|e| RetroError::Other(e.to_string()))?;
    channel
        .request_subsystem(true, "sftp")
        .await
        .map_err(|e| RetroError::Other(e.to_string()))?;
    let sftp = SftpSession::new(channel.into_stream())
        .await
        .map_err(|e| RetroError::Other(format!("sftp init failed: {}", e)))?;

    Ok(RemoteSession { session, sftp })
}

pub fn fingerprint_placeholder() -> Option<String> {
    None
}
