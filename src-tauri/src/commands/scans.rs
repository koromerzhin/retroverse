use crate::error::Result;
use crate::models::ScanResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn scan_source_cmd(state: State<AppState>, source_id: String) -> Result<ScanResult> {
    crate::scanner::scan_source(
        &state,
        &source_id,
        None,
        state.cancel_scan.clone(),
        None,
        None,
    )
}
