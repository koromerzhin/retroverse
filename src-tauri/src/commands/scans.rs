use crate::error::Result;
use crate::models::ScanResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn scan_source_cmd(state: State<AppState>, source_id: String) -> Result<ScanResult> {
    let mut conn = state.db.lock().unwrap();
    crate::scanner::scan_source(&mut conn, &source_id)
}
