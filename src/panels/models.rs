use egui::{Color32, Ui};
use crate::app::PrismSplitApp;
use crate::widgets::fieldset;

pub fn show(app: &mut PrismSplitApp, ui: &mut Ui) {
    ui.columns(3, |columns| {
        if crate::widgets::custom_button(&mut columns[0], "SYNC UVR CATALOG", true, Color32::from_rgb(34, 211, 238)) {
            app.sync_catalog();
        }
        if crate::widgets::custom_button(&mut columns[1], "SCAN LOCAL MODELS", true, Color32::from_rgb(180, 187, 193)) {
            app.browse_local_scan();
        }
        if crate::widgets::custom_button(&mut columns[2], "REFRESH CATALOG", true, Color32::from_rgb(180, 187, 193)) {
            app.load_catalog();
        }
    });
    ui.add_space(8.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for entry in app.state.catalog.clone() {
            fieldset(ui, &entry.name, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Backend: {}", entry.backend));
                    ui.separator();
                    ui.label(format!("Version: {}", entry.version));
                    ui.separator();
                    ui.label(format!("Installed: {}", entry.is_installed));
                });
                ui.label(format!("File: {}", entry.filename));
                if entry.size_bytes > 0 {
                    ui.label(format!(
                        "Size: {:.2} MB",
                        entry.size_bytes as f64 / (1024.0 * 1024.0)
                    ));
                }

                if app.state.downloading_id.as_deref() == Some(entry.id.as_str()) {
                    ui.add(
                        egui::ProgressBar::new(app.state.download_progress / 100.0)
                            .text(format!("{:.0}%", app.state.download_progress)),
                    );
                }

                let label = if entry.is_installed {
                    "INSTALLED"
                } else {
                    "DOWNLOAD MODEL"
                };
                let btn_color = if entry.is_installed {
                    Color32::from_rgb(16, 185, 129) // Green
                } else {
                    Color32::from_rgb(34, 211, 238) // Cyan
                };
                if crate::widgets::custom_button(
                    ui,
                    label,
                    !entry.is_installed && app.state.downloading_id.is_none(),
                    btn_color,
                ) {
                    app.download_model(entry.id.clone());
                }
            });
            ui.add_space(6.0);
        }
    });
}

pub(crate) fn has_trusted_checksum(sha256: &str) -> bool {
    let value = sha256.trim();
    !value.is_empty() && !value.eq_ignore_ascii_case("replace-with-real-sha256")
}

