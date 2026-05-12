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
                    Color32::from_rgb(255, 99, 99)
                } else if upper.contains("WARN") {
                    Color32::from_rgb(255, 212, 92)
                } else if upper.contains("SUCCESS") {
                    Color32::from_rgb(0, 255, 102)
                } else {
                    Color32::from_rgb(196, 204, 210)
                };
                ui.label(RichText::new(line).monospace().color(color));
            }
        });
}
