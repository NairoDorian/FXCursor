use egui::{Color32, Slider};
use crate::config::AppConfig;

fn color_picker(ui: &mut egui::Ui, label: &str, color: &mut [f32; 4], config_changed: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.color_edit_button_rgba_unmultiplied(color).changed() {
            *config_changed = true;
        }
    });
}

pub fn show_settings_panel(ctx: &egui::Context, config: &mut AppConfig, config_changed: &mut bool) {
    // Apply a premium, modern dark theme
    let mut visuals = egui::Visuals::dark();
    visuals.window_rounding = 8.0.into();
    visuals.widgets.active.bg_fill = Color32::from_rgb(0, 204, 255);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(0, 153, 204);
    ctx.set_visuals(visuals);

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("⚡ CURSOR FX ULTIMATE");
            ui.label("WebGPU Multi-Layer FX Engine & Physics Settings");
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(5.0);

            // --- 1. General Setup ---
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let mut enabled = config.enabled;
                    if ui.checkbox(&mut enabled, "Enable Cursor FX").changed() {
                        config.enabled = enabled;
                        *config_changed = true;
                    }

                    ui.add_space(10.0);

                    let mut click_response = config.click_response;
                    if ui.checkbox(&mut click_response, "🌊 Click Ripple").changed() {
                        config.click_response = click_response;
                        *config_changed = true;
                    }
                });

                ui.add_space(5.0);

                let prev_type = config.effect_type;
                ui.horizontal(|ui| {
                    ui.label("FX Mode:");
                    ui.selectable_value(&mut config.effect_type, 0, "☄️ Ribbon Trail");
                    ui.selectable_value(&mut config.effect_type, 1, "🌊 SDF Ripple Only");
                    ui.selectable_value(&mut config.effect_type, 2, "✨ Glow Aura");
                });
                if config.effect_type != prev_type {
                    *config_changed = true;
                }
            });

            ui.add_space(10.0);

            // --- 2. Trail Physics ---
            ui.collapsing("☄️ Trail Physics Settings", |ui| {
                ui.group(|ui| {
                    let prev_length = config.trail_length;
                    let prev_width = config.trail_width;
                    let prev_hs = config.head_spring;
                    let prev_hf = config.head_friction;
                    let prev_bs = config.body_spring;
                    let prev_bf = config.body_friction;
                    let prev_skip = config.position_skip;
                    let prev_steps = config.interpolation_steps;
                    let prev_fade = config.fade_mode;
                    let prev_gradient = config.enable_gradient;
                    let prev_min_w = config.min_trail_width;
                    let prev_vel_w = config.velocity_width_multiplier;
                    let prev_vel_a = config.velocity_alpha_multiplier;

                    ui.horizontal(|ui| {
                        ui.label("Trail Length:");
                        ui.add(Slider::new(&mut config.trail_length, 5..=100).text("nodes"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Base Size:");
                        ui.add(Slider::new(&mut config.trail_width, 5.0..=150.0).text("px"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Min Tail Size:");
                        ui.add(Slider::new(&mut config.min_trail_width, 1.0..=20.0).text("px"));
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Head Spring:");
                        ui.add(Slider::new(&mut config.head_spring, 1.0..=500.0).text("strength"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Head Friction:");
                        ui.add(Slider::new(&mut config.head_friction, 0.0..=99.0).text("%"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Body Spring:");
                        ui.add(Slider::new(&mut config.body_spring, 1.0..=500.0).text("strength"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Body Friction:");
                        ui.add(Slider::new(&mut config.body_friction, 0.0..=99.0).text("%"));
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Position Skip:");
                        ui.add(Slider::new(&mut config.position_skip, 0..=10).text("updates"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Spline Smoothness:");
                        ui.add(Slider::new(&mut config.interpolation_steps, 1..=10).text("steps"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Fade Curve:");
                        ui.selectable_value(&mut config.fade_mode, 0, "Linear");
                        ui.selectable_value(&mut config.fade_mode, 1, "Ease-Out");
                        ui.selectable_value(&mut config.fade_mode, 2, "Expo");
                        ui.selectable_value(&mut config.fade_mode, 3, "Sigmoid");
                    });

                    ui.horizontal(|ui| {
                        let mut rgb_rainbow = config.rainbow_mode;
                        if ui.checkbox(&mut rgb_rainbow, "🌈 Rainbow Hue Cycle").changed() {
                            config.rainbow_mode = rgb_rainbow;
                            *config_changed = true;
                        }
                        if config.rainbow_mode {
                            ui.add(Slider::new(&mut config.rainbow_speed, 0.5..=10.0).text("speed"));
                        }
                    });

                    ui.horizontal(|ui| {
                        let mut adapt = config.adaptive_quality;
                        if ui.checkbox(&mut adapt, "⚡ Adaptive Quality (Drop samples on speed)").changed() {
                            config.adaptive_quality = adapt;
                            *config_changed = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        let mut grad = config.enable_gradient;
                        if ui.checkbox(&mut grad, "🎨 Enable Gradient (Start to End Color)").changed() {
                            config.enable_gradient = grad;
                            *config_changed = true;
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Velocity Width Boost:");
                        ui.add(Slider::new(&mut config.velocity_width_multiplier, 0.0..=5.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Velocity Opacity Boost:");
                        ui.add(Slider::new(&mut config.velocity_alpha_multiplier, 0.0..=5.0));
                    });

                    if config.trail_length != prev_length
                        || (config.trail_width - prev_width).abs() > 0.001
                        || (config.head_spring - prev_hs).abs() > 0.001
                        || (config.head_friction - prev_hf).abs() > 0.001
                        || (config.body_spring - prev_bs).abs() > 0.001
                        || (config.body_friction - prev_bf).abs() > 0.001
                        || config.position_skip != prev_skip
                        || config.interpolation_steps != prev_steps
                        || config.fade_mode != prev_fade
                        || config.enable_gradient != prev_gradient
                        || (config.min_trail_width - prev_min_w).abs() > 0.001
                        || (config.velocity_width_multiplier - prev_vel_w).abs() > 0.001
                        || (config.velocity_alpha_multiplier - prev_vel_a).abs() > 0.001
                    {
                        *config_changed = true;
                    }
                });
            });

            ui.add_space(5.0);

            // --- 3. Trail Layers ---
            ui.collapsing("🎨 Ribbon Trail Layers", |ui| {
                for i in 0..4 {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let mut layer_enabled = config.layers[i].enabled;
                            if ui.checkbox(&mut layer_enabled, format!("Layer {} Active", i + 1)).changed() {
                                config.layers[i].enabled = layer_enabled;
                                *config_changed = true;
                            }
                        });

                        if config.layers[i].enabled {
                            color_picker(ui, "  Start Color:", &mut config.layers[i].start_color, config_changed);
                            if config.enable_gradient {
                                color_picker(ui, "  End Color:", &mut config.layers[i].end_color, config_changed);
                            }

                            let prev_wf = config.layers[i].width_factor;
                            let prev_af = config.layers[i].alpha_factor;
                            let prev_sb = config.layers[i].start_blur;
                            let prev_eb = config.layers[i].end_blur;

                            ui.horizontal(|ui| {
                                ui.label("  Width Factor:");
                                ui.add(Slider::new(&mut config.layers[i].width_factor, 0.05..=3.0).text("x"));
                            });

                            ui.horizontal(|ui| {
                                ui.label("  Opacity Factor:");
                                ui.add(Slider::new(&mut config.layers[i].alpha_factor, 0.0..=1.0));
                            });

                            ui.horizontal(|ui| {
                                ui.label("  Start Blur:");
                                ui.add(Slider::new(&mut config.layers[i].start_blur, 0.0..=1.0));
                            });

                            ui.horizontal(|ui| {
                                ui.label("  End Blur:");
                                ui.add(Slider::new(&mut config.layers[i].end_blur, 0.0..=1.0));
                            });

                            if (config.layers[i].width_factor - prev_wf).abs() > 0.001
                                || (config.layers[i].alpha_factor - prev_af).abs() > 0.001
                                || (config.layers[i].start_blur - prev_sb).abs() > 0.001
                                || (config.layers[i].end_blur - prev_eb).abs() > 0.001
                            {
                                *config_changed = true;
                            }
                        }
                    });
                }
            });

            ui.add_space(5.0);

            // --- 4. Squishy Head ---
            ui.collapsing("🟢 Squishy Cursor Head", |ui| {
                ui.group(|ui| {
                    let mut head_enabled = config.head_enabled;
                    if ui.checkbox(&mut head_enabled, "Enable Cursor Head").changed() {
                        config.head_enabled = head_enabled;
                        *config_changed = true;
                    }

                    if config.head_enabled {
                        let mut head_filled = config.head_filled;
                        if ui.checkbox(&mut head_filled, "Filled Cursor (vs Outline)").changed() {
                            config.head_filled = head_filled;
                            *config_changed = true;
                        }

                        color_picker(ui, "Head Color:", &mut config.head_color, config_changed);

                        let prev_size = config.head_size;
                        let prev_outline = config.head_outline_width;
                        let prev_intensity = config.head_squish_intensity;
                        let prev_smoothing = config.head_squish_smoothing;

                        ui.horizontal(|ui| {
                            ui.label("Base Size:");
                            ui.add(Slider::new(&mut config.head_size, 5.0..=100.0).text("px"));
                        });

                        if !config.head_filled {
                            ui.horizontal(|ui| {
                                ui.label("Outline Width:");
                                ui.add(Slider::new(&mut config.head_outline_width, 1.0..=20.0).text("px"));
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label("Squish Intensity:");
                            ui.add(Slider::new(&mut config.head_squish_intensity, 0.0..=10.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Squish Smoothing:");
                            ui.add(Slider::new(&mut config.head_squish_smoothing, 10.0..=100.0).text("%"));
                        });

                        if (config.head_size - prev_size).abs() > 0.001
                            || (config.head_outline_width - prev_outline).abs() > 0.001
                            || (config.head_squish_intensity - prev_intensity).abs() > 0.001
                            || (config.head_squish_smoothing - prev_smoothing).abs() > 0.001
                        {
                            *config_changed = true;
                        }
                    }
                });
            });

            ui.add_space(5.0);

            // --- 5. Ripples & Particles ---
            ui.collapsing("🌊 Click Ripples & Particles", |ui| {
                ui.group(|ui| {
                    ui.label("🌊 SDF Click Ripples:");
                    color_picker(ui, "Left Click Color:", &mut config.ripple_left_color, config_changed);
                    color_picker(ui, "Right Click Color:", &mut config.ripple_right_color, config_changed);
                    color_picker(ui, "Middle Click Color:", &mut config.ripple_middle_color, config_changed);

                    let prev_duration = config.ripple_duration;
                    let prev_radius = config.ripple_radius;
                    let prev_start_width = config.ripple_start_width;

                    ui.horizontal(|ui| {
                        ui.label("Ripple Expansion Size:");
                        ui.add(Slider::new(&mut config.ripple_radius, 20.0..=400.0).text("px"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Ripple Duration:");
                        ui.add(Slider::new(&mut config.ripple_duration, 0.1..=3.0).text("seconds"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Ripple Start Width:");
                        ui.add(Slider::new(&mut config.ripple_start_width, 0.5..=50.0).text("px"));
                    });

                    if (config.ripple_duration - prev_duration).abs() > 0.001
                        || (config.ripple_radius - prev_radius).abs() > 0.001
                        || (config.ripple_start_width - prev_start_width).abs() > 0.001
                    {
                        *config_changed = true;
                    }

                    ui.separator();

                    ui.label("✨ Click Particle Bursts:");
                    let mut part_enabled = config.particle_enabled;
                    if ui.checkbox(&mut part_enabled, "Enable Particles").changed() {
                        config.particle_enabled = part_enabled;
                        *config_changed = true;
                    }

                    if config.particle_enabled {
                        let prev_cnt = config.particle_count;
                        let prev_spd = config.particle_speed;
                        let prev_life = config.particle_lifetime;
                        let prev_sz = config.particle_size;
                        let prev_fric = config.particle_friction;
                        let prev_grav = config.particle_gravity;

                        ui.horizontal(|ui| {
                            ui.label("Count:");
                            ui.add(Slider::new(&mut config.particle_count, 4..=64));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Ejection Speed:");
                            ui.add(Slider::new(&mut config.particle_speed, 50.0..=1000.0));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Lifetime:");
                            ui.add(Slider::new(&mut config.particle_lifetime, 0.1..=3.0).text("seconds"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Size:");
                            ui.add(Slider::new(&mut config.particle_size, 1.0..=20.0).text("px"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Friction (Air Drag):");
                            ui.add(Slider::new(&mut config.particle_friction, 50.0..=99.0).text("%"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Gravity:");
                            ui.add(Slider::new(&mut config.particle_gravity, -500.0..=1000.0));
                        });

                        if config.particle_count != prev_cnt
                            || (config.particle_speed - prev_spd).abs() > 0.001
                            || (config.particle_lifetime - prev_life).abs() > 0.001
                            || (config.particle_size - prev_sz).abs() > 0.001
                            || (config.particle_friction - prev_fric).abs() > 0.001
                            || (config.particle_gravity - prev_grav).abs() > 0.001
                        {
                            *config_changed = true;
                        }
                    }
                });
            });

            ui.add_space(5.0);

            // --- 6. Satellites ---
            ui.collapsing("🪐 Mirrored Orbiting Satellites", |ui| {
                ui.group(|ui| {
                    let mut sat_enabled = config.satellite_enabled;
                    if ui.checkbox(&mut sat_enabled, "Enable Satellites").changed() {
                        config.satellite_enabled = sat_enabled;
                        *config_changed = true;
                    }

                    if config.satellite_enabled {
                        let mut sat_filled = config.satellite_filled;
                        if ui.checkbox(&mut sat_filled, "Filled Satellites (vs Outline)").changed() {
                            config.satellite_filled = sat_filled;
                            *config_changed = true;
                        }

                        color_picker(ui, "Satellite Color:", &mut config.satellite_color, config_changed);

                        let prev_cnt = config.satellite_count;
                        let prev_orbit = config.satellite_orbit_diameter;
                        let prev_size = config.satellite_size;
                        let prev_out_w = config.satellite_outline_width;
                        let prev_spd = config.satellite_speed;

                        ui.horizontal(|ui| {
                            ui.label("Count:");
                            ui.add(Slider::new(&mut config.satellite_count, 1..=12));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Orbit Diameter:");
                            ui.add(Slider::new(&mut config.satellite_orbit_diameter, 10.0..=400.0).text("px"));
                        });

                        ui.horizontal(|ui| {
                            ui.label("Satellite Size:");
                            ui.add(Slider::new(&mut config.satellite_size, 2.0..=50.0).text("px"));
                        });

                        if !config.satellite_filled {
                            ui.horizontal(|ui| {
                                ui.label("Outline Width:");
                                ui.add(Slider::new(&mut config.satellite_outline_width, 1.0..=10.0).text("px"));
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label("Orbit Speed:");
                            ui.add(Slider::new(&mut config.satellite_speed, -20.0..=20.0).text("deg/frame"));
                        });

                        ui.separator();

                        let mut dual = config.satellite_enable_dual_ring;
                        if ui.checkbox(&mut dual, "Mirrored Dual Ring").changed() {
                            config.satellite_enable_dual_ring = dual;
                            *config_changed = true;
                        }

                        if config.satellite_enable_dual_ring {
                            let prev_dspd = config.satellite_dual_speed;
                            ui.horizontal(|ui| {
                                ui.label("Mirrored Speed:");
                                ui.add(Slider::new(&mut config.satellite_dual_speed, -20.0..=20.0).text("deg/frame"));
                            });
                            if (config.satellite_dual_speed - prev_dspd).abs() > 0.001 {
                                *config_changed = true;
                            }
                        }

                        ui.separator();

                        let mut show_ring = config.satellite_show_orbit_ring;
                        if ui.checkbox(&mut show_ring, "Show Background Orbit Ring").changed() {
                            config.satellite_show_orbit_ring = show_ring;
                            *config_changed = true;
                        }

                        if config.satellite_show_orbit_ring {
                            color_picker(ui, "Ring Color:", &mut config.satellite_ring_color, config_changed);

                            let prev_rw = config.satellite_ring_width;
                            ui.horizontal(|ui| {
                                ui.label("Ring Width:");
                                ui.add(Slider::new(&mut config.satellite_ring_width, 1.0..=10.0).text("px"));
                            });
                            if (config.satellite_ring_width - prev_rw).abs() > 0.001 {
                                *config_changed = true;
                            }
                        }

                        if config.satellite_count != prev_cnt
                            || (config.satellite_orbit_diameter - prev_orbit).abs() > 0.001
                            || (config.satellite_size - prev_size).abs() > 0.001
                            || (config.satellite_outline_width - prev_out_w).abs() > 0.001
                            || (config.satellite_speed - prev_spd).abs() > 0.001
                        {
                            *config_changed = true;
                        }
                    }
                });
            });

            ui.add_space(15.0);

            // --- 7. Save & Restores ---
            ui.horizontal(|ui| {
                if ui.button("💾 Save Settings").clicked() {
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
