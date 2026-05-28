use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent},
    Icon, TrayIcon, TrayIconBuilder,
};

pub enum TrayAction {
    Quit,
    ToggleConfig,
}

pub struct SystemTray {
    _tray_icon: TrayIcon,
    quit_item: MenuItem,
    toggle_config_item: MenuItem,
}

impl SystemTray {
    pub fn new() -> Self {
        let menu = Menu::new();
        let toggle_config_item = MenuItem::new("Show/Hide Settings", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append(&toggle_config_item).unwrap();
        menu.append(&quit_item).unwrap();

        // Generate a 32x32 RGBA icon (a nice neon cyan circle for Cursor FX)
        let width = 32;
        let height = 32;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let dx = (x as f32) - 15.5;
                let dy = (y as f32) - 15.5;
                let dist_sq = dx * dx + dy * dy;
                let idx = ((y * width + x) * 4) as usize;
                
                // Draw a simple glowing circle
                if dist_sq < 144.0 { // Radius ~12
                    rgba[idx] = 0;       // R
                    rgba[idx + 1] = 204; // G
                    rgba[idx + 2] = 255; // B
                    rgba[idx + 3] = 255; // A (opaque)
                } else if dist_sq < 196.0 { // Glow border
                    rgba[idx] = 0;
                    rgba[idx + 1] = 204;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = 120; // translucent glow
                } else {
                    rgba[idx] = 0;
                    rgba[idx + 1] = 0;
                    rgba[idx + 2] = 0;
                    rgba[idx + 3] = 0;   // transparent background
                }
            }
        }

        let icon = Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon");

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Cursor FX")
            .with_icon(icon)
            .build()
            .expect("Failed to build tray icon");

        Self {
            _tray_icon: tray_icon,
            quit_item,
            toggle_config_item,
        }
    }

    /// Check if a menu event corresponds to our tray items and map it to a TrayAction.
    pub fn handle_menu_event(&self, event: &MenuEvent) -> Option<TrayAction> {
        if event.id == self.quit_item.id() {
            Some(TrayAction::Quit)
        } else if event.id == self.toggle_config_item.id() {
            Some(TrayAction::ToggleConfig)
        } else {
            None
        }
    }
}
