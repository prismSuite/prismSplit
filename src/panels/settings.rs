use egui::{Color32, Ui};
use crate::app::PrismSplitApp;
use crate::widgets::fieldset;

pub fn show(app: &mut PrismSplitApp, ui: &mut Ui) {
    ui.columns(2, |columns| {
        let col_0 = &mut columns[0];
        fieldset(col_0, "SYSTEM_PATHS", |ui| {
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

        col_0.add_space(8.0);

        fieldset(col_0, "INFERENCE_CONFIGURATION", |ui| {
            ui.label("INFERENCE DEVICE");
            let mut device = app.state.config.inference_device.clone().unwrap_or_else(|| "Auto".to_string());
            egui::ComboBox::from_id_source("inference_device_selector")
                .selected_text(&device)
                .show_ui(ui, |ui| {
                    for option in &["Auto", "CPU", "CUDA", "MPS"] {
                        ui.selectable_value(&mut device, option.to_string(), *option);
                    }
                });
            app.state.config.inference_device = Some(device);

            ui.add_space(6.0);
            ui.label("MDX OVERLAP (Voz/Música)");
            let mut overlap = app.state.config.mdx_overlap.unwrap_or(0.25);
            ui.add(egui::Slider::new(&mut overlap, 0.0..=0.95).text("Overlap"));
            app.state.config.mdx_overlap = Some(overlap);

            ui.add_space(6.0);
            ui.label("CPU THREADS");
            let mut threads = app.state.config.cpu_threads.unwrap_or(0);
            ui.add(egui::Slider::new(&mut threads, 0..=16).text("Threads (0 = Auto)"));
            app.state.config.cpu_threads = Some(threads);
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
