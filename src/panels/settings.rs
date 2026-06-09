use egui::{Color32, Ui};
use crate::app::PrismSplitApp;
use crate::widgets::fieldset;

pub fn show(app: &mut PrismSplitApp, ui: &mut Ui) {
    ui.columns(2, |columns| {
        fieldset(&mut columns[0], "SYSTEM_PATHS", |ui| {
            ui.label("MODELS DIRECTORY");
            ui.horizontal(|ui| {
                let value = app.state.config.models_dir.get_or_insert_with(String::new);
                ui.text_edit_singleline(value);
                if ui.button("BROWSE").clicked() {
                    app.browse_models_dir();
                }
            });

            ui.label("CACHE DIRECTORY");
            ui.horizontal(|ui| {
                let value = app.state.config.cache_dir.get_or_insert_with(String::new);
                ui.text_edit_singleline(value);
                if ui.button("BROWSE").clicked() {
                    app.browse_cache_dir();
                }
            });
        });

        fieldset(&mut columns[1], "ENGINE_MAINTENANCE", |ui| {
            ui.label("HARDWARE ACCELERATION");
            if let Some(health) = &app.state.health {
                if health.gpu_devices.is_empty() {
                    ui.label("No GPU devices detected.");
                } else {
                    for gpu in &health.gpu_devices {
                        ui.label(gpu);
                    }
                }
            } else {
                ui.label("Health data unavailable.");
            }

            ui.add_space(6.0);
            if crate::widgets::custom_button(ui, "APPLY SYSTEM CONFIGURATION", true, Color32::from_rgb(16, 185, 129)) {
                app.apply_settings();
            }

            ui.add_space(10.0);
            ui.separator();
            ui.label("SMART DIAGNOSTICS & SYSTEM SAFETY");
            ui.add_space(6.0);
            ui.columns(2, |columns| {
                if crate::widgets::custom_button(&mut columns[0], "SMART REPAIR", true, Color32::from_rgb(245, 158, 11)) {
                    app.repair_engine();
                }
                if crate::widgets::custom_button(&mut columns[1], "FORCE RE-PREPARE", true, Color32::from_rgb(239, 68, 68)) {
                    app.prepare_engine();
                }
            });
        });
    });
}
