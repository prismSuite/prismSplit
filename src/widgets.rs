use egui::{Align, Color32, Frame, Layout, RichText, Stroke, Ui};

pub fn nav_button(ui: &mut Ui, active: bool, label: &str) -> bool {
    let text = if active {
        // Monolith primary accent (Electric Cyan)
        RichText::new(label).strong().color(Color32::from_rgb(34, 211, 238))
    } else {
        RichText::new(label)
    };

    ui.selectable_label(active, text).clicked()
}

pub fn fieldset<R>(ui: &mut Ui, legend: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let frame = Frame::none()
        .inner_margin(12.0)
        .outer_margin(6.0);

    let response = frame.show(ui, |ui| {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.label(RichText::new(legend).strong().monospace().color(Color32::from_rgb(34, 211, 238)));
            ui.separator();
            add_contents(ui)
        })
        .inner
    });

    let rect = response.response.rect;
    let painter = ui.painter();

    // Retro Industrial 3D bevel colors (outset/inset simulation)
    // Dark shadow for top and left (recessed look) (Surface-0)
    let shadow = Color32::from_rgb(5, 5, 5); 
    // Light highlights for bottom and right (Highlight-6)
    let highlight = Color32::from_rgb(50, 50, 50); 

    // Draw the 3D frame border lines
    painter.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.5, shadow));
    painter.line_segment([rect.left_top(), rect.left_bottom()], Stroke::new(1.5, shadow));
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.5, highlight));
    painter.line_segment([rect.right_top(), rect.right_bottom()], Stroke::new(1.5, highlight));

    response.inner
}

pub fn status_chip(ui: &mut Ui, label: &str, value: &str, accent: bool) {
    let color = if accent {
        // Monolith primary accent (Electric Cyan)
        Color32::from_rgb(34, 211, 238)
    } else {
        Color32::from_rgb(180, 187, 193)
    };
    ui.label(
        RichText::new(format!("{} {}", label, value))
            .monospace()
            .color(color),
    );
}
