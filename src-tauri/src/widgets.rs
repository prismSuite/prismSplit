use egui::{Align, Color32, Frame, Layout, RichText, Stroke, Ui};

pub fn nav_button(ui: &mut Ui, active: bool, label: &str) -> bool {
    let text = if active {
        // Monolith primary accent
        RichText::new(label).strong().color(Color32::from_rgb(0, 210, 255))
    } else {
        RichText::new(label)
    };

    ui.selectable_label(active, text).clicked()
}

pub fn fieldset<R>(ui: &mut Ui, legend: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let frame = Frame::group(ui.style())
        // Monolith surface-6 / subtle border
        .stroke(Stroke::new(1.0, Color32::from_rgb(58, 64, 85)))
        .inner_margin(12.0);

    frame
        .show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.label(RichText::new(legend).strong().monospace());
                ui.separator();
                add_contents(ui)
            })
            .inner
        })
        .inner
}

pub fn status_chip(ui: &mut Ui, label: &str, value: &str, accent: bool) {
    let color = if accent {
        // Monolith primary accent
        Color32::from_rgb(0, 210, 255)
    } else {
        Color32::from_rgb(180, 187, 193)
    };
    ui.label(
        RichText::new(format!("{} {}", label, value))
            .monospace()
            .color(color),
    );
}
