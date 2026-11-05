use ksni::menu::StandardItem;
use ksni::{MenuItem, ToolTip, Tray, TrayMethods};

use tauri::AppHandle;
use tauri::Manager;

pub struct RetroverseTray {
    app: AppHandle,
}

impl RetroverseTray {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Tray for RetroverseTray {
    fn id(&self) -> String {
        "retroverse".into()
    }

    fn title(&self) -> String {
        "RETROVERSE".into()
    }

    fn icon_name(&self) -> String {
        "applications-games".into()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        // Load the application logo as the tray icon pixmap.
        let bytes = include_bytes!("../icons/icon.png");
        if let Ok(img) = image::load_from_memory(bytes) {
            let img = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            return vec![ksni::Icon {
                width: w as i32,
                height: h as i32,
                data: rgba.into_raw(),
            }];
        }
        Vec::new()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "RETROVERSE".into(),
            description: "Universal Video Game Collection Companion".into(),
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let visible = self
            .app
            .get_webview_window("main")
            .and_then(|w| w.is_visible().ok())
            .unwrap_or(false);
        let toggle_label = if visible {
            "Masquer RETROVERSE"
        } else {
            "Afficher RETROVERSE"
        };
        vec![
            MenuItem::Standard(StandardItem {
                label: toggle_label.into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    if let Some(w) = tray.app.get_webview_window("main") {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Tout scanner".into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let app = tray.app.clone();
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
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Calculer les empreintes".into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    let app = tray.app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app.state::<crate::state::AppState>().inner().clone();
                        let _ =
                            crate::commands::backup::compute_hashes_inner(&state, Some(app), None)
                                .await;
                    });
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Quitter".into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    tray.app.exit(0);
                }),
                ..Default::default()
            }),
        ]
    }
}

pub fn spawn_tray(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let tray = RetroverseTray::new(app);
        if let Err(e) = tray.spawn().await {
            eprintln!("tray spawn error: {}", e);
        }
    });
}
