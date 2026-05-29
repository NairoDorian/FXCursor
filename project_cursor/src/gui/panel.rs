use egui::{Color32, Slider};
use crate::config::AppConfig;


pub fn show_settings_panel(ctx: &egui::Context, config: &mut AppConfig, config_changed: &mut bool) {
    // Apply a premium, modern dark theme
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = 8.0.into();
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 204, 255);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 153, 204);
    ctx.set_visuals(visuals);

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("⚡ CURSOR FX SETTINGS");
            ui.label("Hardware-Accelerated Mouse Effects");
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);

            // General Toggle
            ui.horizontal(|ui| {
                let mut enabled = config.enabled;
                if ui.checkbox(&mut enabled, "Enable Cursor Effects").changed() {
                    config.enabled = enabled;
                    *config_changed = true;
                }

                ui.add_space(20.0);

                let mut click_response = config.click_response;
                if ui.checkbox(&mut click_response, "🌊 Click Ripple on Click").changed() {
                    config.click_response = click_response;
                    *config_changed = true;
                }
            });

            ui.add_space(10.0);

            // Effect Selector
            ui.group(|ui| {
                ui.label("Effect Type:");
                let prev_type = config.effect_type;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut config.effect_type, 0, "☄️ Particle Trail");
                    ui.selectable_value(&mut config.effect_type, 1, "🌊 Click Ripple");
                    ui.selectable_value(&mut config.effect_type, 2, "✨ Glow Aura");
                });
                if config.effect_type != prev_type {
                    *config_changed = true;
                }
            });

            ui.add_space(15.0);

            // Color Customization
            ui.group(|ui| {
                ui.label("Visual Aesthetics:");
                ui.horizontal(|ui| {
                    ui.label("FX Color:");
                    let mut color = Color32::from_rgba_unmultiplied(
                        (config.trail_color[0] * 255.0) as u8,
                        (config.trail_color[1] * 255.0) as u8,
                        (config.trail_color[2] * 255.0) as u8,
                        (config.trail_color[3] * 255.0) as u8,
                    );
                    if ui.color_edit_button_srgba(&mut color).changed() {
                        let rgba = color.to_array();
                        config.trail_color = [
                            rgba[0] as f32 / 255.0,
                            rgba[1] as f32 / 255.0,
                            rgba[2] as f32 / 255.0,
                            rgba[3] as f32 / 255.0,
                        ];
                        *config_changed = true;
                    }
                });

                ui.add_space(5.0);

                // Config Sliders based on selected effect
                let prev_len = config.trail_length;
                let prev_width = config.trail_width;

                ui.horizontal(|ui| {
                    ui.label("Trail Length:");
                    ui.add(Slider::new(&mut config.trail_length, 16..=256).text("particles"));
                });

                ui.horizontal(|ui| {
                    ui.label("FX Width / Size:");
                    ui.add(Slider::new(&mut config.trail_width, 1.0..=50.0).text("px"));
                });

                if config.trail_length != prev_len || (config.trail_width - prev_width).abs() > 0.001 {
                    *config_changed = true;
                }
            });

            ui.add_space(15.0);

            // Physics Customization
            ui.group(|ui| {
                ui.label("Physics Dynamics:");
                
                let prev_speed = config.speed;
                let prev_friction = config.friction;
                let prev_gravity = config.gravity;
                let prev_radius = config.ripple_radius;

                ui.horizontal(|ui| {
                    ui.label("Motion Speed:");
                    ui.add(Slider::new(&mut config.speed, 0.1..=3.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Friction (Damping):");
                    ui.add(Slider::new(&mut config.friction, 0.001..=0.3));
                });

                ui.horizontal(|ui| {
                    ui.label("Gravity / Buoyancy:");
                    ui.add(Slider::new(&mut config.gravity, -5.0..=5.0));
                });

                if config.effect_type == 1 {
                    ui.horizontal(|ui| {
                        ui.label("Max Ripple Radius:");
                        ui.add(Slider::new(&mut config.ripple_radius, 20.0..=400.0).text("px"));
                    });
                }

                if (config.speed - prev_speed).abs() > 0.001
                    || (config.friction - prev_friction).abs() > 0.001
                    || (config.gravity - prev_gravity).abs() > 0.001
                    || (config.ripple_radius - prev_radius).abs() > 0.001
                {
                    *config_changed = true;
                }
            });

            ui.add_space(20.0);

            // Action Buttons
            ui.horizontal(|ui| {
                if ui.button("💾 Save Config").clicked() {
                    if let Err(e) = config.save() {
                        log::error!("Failed to save config: {}", e);
                    }
                }
                
                if ui.button("🔄 Reset Defaults").clicked() {
                    *config = AppConfig::default();
                    *config_changed = true;
                    let _ = config.save();
                }
            });
        });
    });
}
