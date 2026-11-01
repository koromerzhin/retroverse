use crate::error::Result;
use crate::state::AppState;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn app_ready(state: State<'_, AppState>, app: AppHandle) -> Result<()> {
    tracing::info!("app_ready invoked");
    // Guard against duplicate calls.
    if state.ready.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(());
    }

    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
    Ok(())
}
