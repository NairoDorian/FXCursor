use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub enabled: bool,
    pub effect_type: u32,       // 0: Particle Trail, 1: Ripple, 2: Glow
    pub trail_color: [f32; 4],  // RGBA [0.0 - 1.0]
    pub trail_length: u32,
    pub trail_width: f32,
    pub speed: f32,
    pub friction: f32,
    pub gravity: f32,
    pub ripple_radius: f32,
    pub click_response: bool,   // Ripple on click
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            effect_type: 0,
            trail_color: [0.0, 0.8, 1.0, 1.0], // Neon cyan
            trail_length: 64,
            trail_width: 8.0,
            speed: 1.0,
            friction: 0.08,
            gravity: 0.0,
            ripple_radius: 100.0,
            click_response: true,
        }
    }
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        let path = Path::new("config.ron");
        if path.exists() {
            if let Ok(mut file) = File::open(path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(config) = ron::from_str(&contents) {
                        return config;
                    }
                }
            }
        }
        
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new("config.ron");
        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}
