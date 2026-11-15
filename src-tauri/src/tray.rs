use tauri::AppHandle;
use tauri::Manager;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

pub fn spawn_tray(app: AppHandle) {
    let menu = build_menu(&app).unwrap_or_else(|_| Menu::new(&app).unwrap());
    let icon = Image::from_bytes(include_bytes!("../icons/icon.png")).ok();

    let app_handle = app.clone();
    let tray = TrayIconBuilder::new()
        .icon(icon.unwrap_or_else(|| Image::new(&[], 1, 1)))
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "toggle" => {
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
            "scan" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<crate::state::AppState>().inner().clone();
                    state
                        .cancel_scan
                        .store(false, std::sync::atomic::Ordering::SeqCst);
                    let cancel = state.cancel_scan.clone();

                    let sources: Vec<(String, String)> = {
                        let conn = state.db.lock().unwrap();
                        let mut stmt = match conn.prepare("SELECT id, kind FROM game_sources") {
                            Ok(s) => s,
                            Err(_) => return,
                        };
                        let rows = match stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))) {
                            Ok(r) => r,
                            Err(_) => return,
                        };
                        rows.filter_map(|r| r.ok()).collect()
                    };

                    for (sid, kind) in sources {
                        if kind == "remote" {
                            let _ = crate::commands::remote::scan_remote_source_inner(
                                &state,
                                sid,
                                Some(app.clone()),
                                cancel.clone(),
                                None,
                                None,
                            )
                            .await;
                        } else {
                            let state2 = state.clone();
                            let app2 = app.clone();
                            let sid2 = sid.clone();
                            let cancel2 = cancel.clone();
                            let _ = tauri::async_runtime::spawn_blocking(move || {
                                crate::scanner::scan_source(
                                    &state2,
                                    &sid2,
                                    Some(app2),
                                    cancel2,
                                    None,
                                    None,
                                )
                            })
                            .await;
                        }
                    }
                });
            }
            "hash" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let state = app.state::<crate::state::AppState>().inner().clone();
                    let _ = crate::commands::backup::compute_hashes_inner(&state, Some(app), None)
                        .await;
                });
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(&app_handle);

    if let Err(e) = tray {
        eprintln!("tray spawn error: {}", e);
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let toggle = MenuItem::with_id(
        app,
        "toggle",
        "Afficher/Masquer RETROVERSE",
        true,
        None::<&str>,
    )?;
    let scan = MenuItem::with_id(app, "scan", "Tout scanner", true, None::<&str>)?;
    let hash = MenuItem::with_id(app, "hash", "Calculer les empreintes", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quitter", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    menu.append(&toggle)?;
    menu.append(&sep1)?;
    menu.append(&scan)?;
    menu.append(&hash)?;
    menu.append(&sep2)?;
    menu.append(&quit)?;
    Ok(menu)
}
