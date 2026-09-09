use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct TrayManager {
    pub tray_icon: TrayIcon,
    pub open_item_id: muda::MenuId,
    pub toggle_item_id: muda::MenuId,
    pub quit_item_id: muda::MenuId,
}

impl TrayManager {
    pub fn new() -> Self {
        let menu = Menu::new();
        let open_item = MenuItem::new("Open Settings...", true, None);
        let toggle_item = MenuItem::new("Toggle Effects", true, None);
        let quit_item = MenuItem::new("Quit FXCursor", true, None);

        let open_item_id = open_item.id().clone();
        let toggle_item_id = toggle_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        let _ = menu.append(&open_item);
        let _ = menu.append(&toggle_item);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&quit_item);

        // Generate simple 32x32 RGBA icon buffer
        let mut icon_rgba = vec![0u8; 32 * 32 * 4];
        for y in 0..32 {
            for x in 0..32 {
                let idx = (y * 32 + x) * 4;
                let dx = x as f32 - 15.5;
                let dy = y as f32 - 15.5;
                if (dx * dx + dy * dy).sqrt() <= 12.0 {
                    icon_rgba[idx] = 0;
                    icon_rgba[idx + 1] = 242;
                    icon_rgba[idx + 2] = 254;
                    icon_rgba[idx + 3] = 255;
                }
            }
        }

        let icon = Icon::from_rgba(icon_rgba, 32, 32).expect("Failed to create tray icon");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("FXCursor V4")
            .with_icon(icon)
            .build()
            .expect("Failed to build system tray icon");

        Self {
            tray_icon,
            open_item_id,
            toggle_item_id,
            quit_item_id,
        }
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}
