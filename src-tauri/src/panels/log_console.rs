use egui::{Color32, RichText, Ui};
use std::collections::VecDeque;

pub fn show(ui: &mut Ui, logs: &VecDeque<String>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("SYSTEM LOG").strong().monospace());
        ui.separator();
        ui.label(RichText::new(format!("{} lines", logs.len())).monospace());
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for line in logs {
                let upper = line.to_ascii_uppercase();
                let color = if upper.contains("ERROR") || upper.contains("ERR:") {
                    Color32::from_rgb(242, 139, 130) // --ui-danger
                } else if upper.contains("WARN") {
                    Color32::from_rgb(251, 191, 36) // --ui-warning
                } else if upper.contains("SUCCESS") {
                    Color32::from_rgb(52, 168, 83) // --ui-success
                } else {
                    Color32::from_rgb(230, 230, 235) // standard text
                };
                ui.label(RichText::new(line).monospace().color(color));
            }
        });
}
