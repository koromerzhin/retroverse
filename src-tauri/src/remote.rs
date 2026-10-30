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

pub async fn connect(
    host: &str,
    port: u16,
    username: &str,
    key_path: &str,
) -> Result<RemoteSession> {
    let config = Arc::new(client::Config::default());
    let sh = ClientHandler;
    let mut session = client::connect(config, (host, port), sh)
        .await
        .map_err(|e| RetroError::Other(format!("ssh connect failed: {}", e)))?;

    // authenticate with key
    let key = keys::load_secret_key(key_path, None)
        .map_err(|e| RetroError::Other(format!("load key failed: {}", e)))?;
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
