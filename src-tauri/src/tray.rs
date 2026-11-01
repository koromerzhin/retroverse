use ksni::menu::StandardItem;
use ksni::{Icon, MenuItem, ToolTip, Tray, TrayMethods};

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

    fn icon_pixmap(&self) -> Vec<Icon> {
        // 16x16 purple "R" placeholder (ARGB32)
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
        vec![
            MenuItem::Standard(StandardItem {
                label: "Afficher RETROVERSE".into(),
                enabled: true,
                activate: Box::new(|tray: &mut Self| {
                    if let Some(w) = tray.app.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
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