use egui::{Align, Color32, Frame, Layout, RichText, Stroke, Ui};

pub fn nav_button(ui: &mut Ui, active: bool, label: &str) -> bool {
    let text = if active {
        RichText::new(label).strong().color(Color32::from_rgb(0, 255, 102))
    } else {
        RichText::new(label)
    };

    ui.selectable_label(active, text).clicked()
}

pub fn fieldset<R>(ui: &mut Ui, legend: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let frame = Frame::group(ui.style())
        .stroke(Stroke::new(1.0, Color32::from_rgb(70, 70, 70)))
        .inner_margin(10.0);

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
        Color32::from_rgb(0, 255, 102)
    } else {
        Color32::from_rgb(180, 187, 193)
    };
    ui.label(
        RichText::new(format!("{} {}", label, value))
            .monospace()
            .color(color),
    );
}
