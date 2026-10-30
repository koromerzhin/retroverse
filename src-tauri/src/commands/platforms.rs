use crate::error::Result;
use crate::identify;
use crate::models::Platform;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn list_platforms(state: State<AppState>) -> Result<Vec<Platform>> {
    let conn = state.db.lock().unwrap();
    identify::load_platforms(&conn)
}
