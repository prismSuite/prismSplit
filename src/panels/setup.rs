use egui::{Color32, RichText, Ui};
use crate::app::PrismSplitApp;
use crate::widgets::fieldset;

pub fn show(app: &mut PrismSplitApp, ui: &mut Ui) {
    fieldset(ui, "CORE_SETUP", |ui| {
        if let Some(health) = &app.state.health {
            ui.label(format!("Runtime ready: {}", health.runtime_ready));
            ui.label(format!("Dependencies ready: {}", health.dependencies_ready));
            ui.label(format!("Catalog ready: {}", health.model_catalog_ready));
            ui.label(format!("Installed models: {}", health.installed_model_count));
        } else {
            ui.label("No engine health data yet.");
        }

        if let Some(status) = &app.state.setup_status {
            ui.separator();
            ui.label(RichText::new("Last prepare status").strong());
            ui.label(format!("Ready: {}", status.ready));
            if !status.completed_stages.is_empty() {
                ui.label(format!(
                    "Stages: {}",
                    status.completed_stages.join(", ")
                ));
            }
            if let Some(error) = &status.last_error {
                ui.colored_label(Color32::from_rgb(242, 139, 130), error);
            }
        }

        ui.add_space(10.0);
        ui.columns(3, |columns| {
            if crate::widgets::custom_button(&mut columns[0], "PREPARE ENGINE", true, Color32::from_rgb(34, 211, 238)) {
                app.prepare_engine();
            }
            if crate::widgets::custom_button(&mut columns[1], "SMART REPAIR", true, Color32::from_rgb(245, 158, 11)) {
                app.repair_engine();
            }
            if crate::widgets::custom_button(&mut columns[2], "REFRESH", true, Color32::from_rgb(180, 187, 193)) {
                app.refresh_health();
            }
        });
    });
}
