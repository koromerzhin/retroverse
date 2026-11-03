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
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.center();
        let _ = main.show();
        let _ = main.set_focus();
    }
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
    Ok(())
}
