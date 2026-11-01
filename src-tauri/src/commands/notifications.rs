use crate::error::Result;

#[tauri::command]
pub fn show_notification(title: String, body: String) -> Result<()> {
    use notify_rust::Notification;
    let _ = Notification::new()
        .summary(&title)
        .body(&body)
        .appname("RETROVERSE")
        .show();
    Ok(())
}
