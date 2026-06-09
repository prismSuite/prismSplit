use egui::{Color32, RichText, Ui};
use crate::app::PrismSplitApp;
use crate::widgets::fieldset;
use crate::state::AppState;

pub fn show(app: &mut PrismSplitApp, ui: &mut Ui) {
    ui.columns(2, |columns| {
        fieldset(&mut columns[0], "I/O TARGETS", |ui| {
            ui.label("INPUT FILE");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.state.input_file);
                if ui.button("BROWSE").clicked() {
                    app.browse_input_file();
                }
            });

            ui.label("OUTPUT DIRECTORY");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.state.output_dir);
                if ui.button("BROWSE").clicked() {
                    app.browse_output_dir();
                }
            });

            if !app.state.input_file.is_empty() {
                ui.label(
                    RichText::new("Tip: you can also drag a file onto the window.")
                        .italics()
                        .small(),
                );
            }
        });

        fieldset(&mut columns[1], "ENGINE_PARAMS", |ui| {
            ui.label("MODEL");
            egui::ComboBox::from_id_source("model_selector")
                .selected_text(selected_model_label(&app.state))
                .show_ui(ui, |ui| {
                    for entry in &app.state.catalog {
                        ui.selectable_value(
                            &mut app.state.selected_model,
                            entry.id.clone(),
                            format!("{} [{}]", entry.name, entry.backend),
                        );
                    }
                });

            ui.label("EXPORT FORMAT");
            egui::ComboBox::from_id_source("export_format")
                .selected_text(app.state.export_format.as_str())
                .show_ui(ui, |ui| {
                    for option in ["WAV", "FLAC", "MP3"] {
                        ui.selectable_value(
                            &mut app.state.export_format,
                            option.to_string(),
                            option,
                        );
                    }
                });

            ui.label("QUALITY");
            egui::ComboBox::from_id_source("quality")
                .selected_text(app.state.quality.as_str())
                .show_ui(ui, |ui| {
                    for option in crate::state::QUALITY_PRESETS {
                        ui.selectable_value(&mut app.state.quality, option.to_string(), *option);
                    }
                });

            ui.add_space(8.0);
            ui.checkbox(&mut app.state.enable_preview, "PREVISUALIZAR STEMS EN VIVO");
        });
    });

    ui.add_space(12.0);
    fieldset(ui, "EXECUTION_NODE", |ui| {
        if app.state.is_processing {
            // Animate VU meter
            let time = ui.input(|i| i.time);
            let val_l = (time * 8.0).sin().abs() as f32 * 0.7 + (time * 19.0).cos().abs() as f32 * 0.25;
            let val_l = val_l.clamp(0.1, 0.98);
            let peak_l = val_l + 0.02;
            ui.label(RichText::new("L-CH LEVEL").monospace().small());
            crate::widgets::vu_meter(ui, val_l, peak_l);
            ui.add_space(4.0);
            
            let val_r = (time * 7.1).cos().abs() as f32 * 0.65 + (time * 15.3).sin().abs() as f32 * 0.3;
            let val_r = val_r.clamp(0.1, 0.98);
            let peak_r = val_r + 0.02;
            ui.label(RichText::new("R-CH LEVEL").monospace().small());
            crate::widgets::vu_meter(ui, val_r, peak_r);
            ui.add_space(8.0);

            ui.add(
                egui::ProgressBar::new(app.state.process_progress / 100.0)
                    .text(format!("{:.0}% active", app.state.process_progress)),
            );
            ui.add_space(8.0);
            if crate::widgets::custom_button(
                ui,
                "CANCEL SEPARATION",
                true,
                Color32::from_rgb(239, 68, 68),
            ) {
                let backend = std::sync::Arc::clone(&app.backend);
                let rt = std::sync::Arc::clone(&app.runtime);
                rt.spawn(async move {
                    backend.kill_process("job-local").await;
                });
                app.state.is_processing = false;
                app.state.process_progress = 0.0;
                app.state.push_log("USER: Separation cancelled by user.");
            }
        } else {
            // Show noise floor (signal present indicator)
            ui.label(RichText::new("L-CH LEVEL (STANDBY)").monospace().small());
            crate::widgets::vu_meter(ui, 0.02, 0.05);
            ui.add_space(4.0);
            ui.label(RichText::new("R-CH LEVEL (STANDBY)").monospace().small());
            crate::widgets::vu_meter(ui, 0.015, 0.04);
            ui.add_space(8.0);
        }

        let can_start = !app.state.input_file.trim().is_empty()
            && !app.state.selected_model.trim().is_empty()
            && !app.state.is_processing;

        if crate::widgets::custom_button(
            ui,
            "START SEPARATION ENGINE",
            can_start,
            Color32::from_rgb(34, 211, 238),
        ) {
            app.state.is_processing = true;
            app.process_audio();
        }
    });
}

fn selected_model_label(state: &AppState) -> String {
    state
        .catalog
        .iter()
        .find(|entry| entry.id == state.selected_model)
        .map(|entry| format!("{} [{}]", entry.name, entry.backend))
        .unwrap_or_else(|| "Select a model".into())
}
