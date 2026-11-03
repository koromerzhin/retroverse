use crate::error::Result;
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn app_ready(state: State<'_, AppState>, app: AppHandle) -> Result<()> {
    // Guard against duplicate calls.
    if state.ready.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }

    tracing::info!("app_ready: waiting for minimum splash duration");
    let elapsed = state.started_at.elapsed();
    if elapsed < Duration::from_secs(15) {
        tokio::time::sleep(Duration::from_secs(15) - elapsed).await;
    }

    tracing::info!("app_ready: closing splashscreen");
    if let Some(splash) = app.get_webview_window("splashscreen") {
        tracing::info!("app_ready: splash close={:?}", splash.close());
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_always_on_top(true);
        tracing::info!("app_ready: show={:?} focus={:?} attn={:?}", main.show(), main.set_focus(), main.request_user_attention(Some(tauri::UserAttentionType::Critical)));
        let _ = main.set_always_on_top(false);
    } else {
        tracing::error!("app_ready: main window not found");
    }
    Ok(())
}
