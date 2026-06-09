use egui::{Align, Color32, Frame, Layout, RichText, Stroke, Ui, Align2};

pub fn nav_button(ui: &mut Ui, active: bool, label: &str) -> bool {
    let text = if active {
        // Monolith primary accent (Electric Cyan)
        RichText::new(label).strong().monospace().color(Color32::from_rgb(34, 211, 238))
    } else {
        RichText::new(label).monospace().color(Color32::from_rgb(140, 150, 160))
    };

    let response = ui.selectable_label(active, text);
    
    // Draw a small neon indicator line below active tabs
    if active {
        let rect = response.rect;
        let line_y = rect.bottom() + 2.0;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), line_y), egui::pos2(rect.right(), line_y)],
            Stroke::new(2.0, Color32::from_rgb(34, 211, 238)),
        );
    }
    
    response.clicked()
}

pub fn draw_screw(ui: &mut Ui, center: egui::Pos2) {
    let painter = ui.painter();
    let radius = 5.0;
    
    // 1. Dark outer shadow
    painter.circle_filled(center, radius + 0.5, Color32::from_rgb(5, 5, 5));
    // 2. Main steel body
    painter.circle_filled(center, radius, Color32::from_rgb(90, 95, 100));
    // 3. Highlight edge
    painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_rgb(160, 165, 170)));
    // 4. Center dark well
    painter.circle_filled(center, 2.0, Color32::from_rgb(30, 30, 30));
    
    // 5. Screw slit (flathead at 45deg)
    painter.line_segment(
        [egui::pos2(center.x - 2.5, center.y - 2.5), egui::pos2(center.x + 2.5, center.y + 2.5)],
        Stroke::new(1.2, Color32::from_rgb(20, 20, 20)),
    );
}

pub fn fieldset<R>(ui: &mut Ui, legend: &str, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let frame = Frame::none()
        .inner_margin(14.0)
        .outer_margin(6.0);

    let response = frame.show(ui, |ui| {
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.horizontal(|ui| {
                // LED status light for the panel
                ui.painter().circle_filled(
                    egui::pos2(ui.cursor().min.x + 4.0, ui.cursor().min.y + 7.0),
                    3.0,
                    Color32::from_rgb(34, 211, 238),
                );
                ui.add_space(12.0);
                ui.label(
                    RichText::new(legend)
                        .strong()
                        .monospace()
                        .color(Color32::from_rgb(230, 235, 240)),
                );
            });
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
            add_contents(ui)
        })
        .inner
    });

    let rect = response.response.rect;
    let painter = ui.painter();

    // 3D double bevel border (recessed well style)
    let shadow_dark = Color32::from_rgb(5, 5, 5); 
    let shadow_med = Color32::from_rgb(20, 20, 20); 
    let highlight_light = Color32::from_rgb(55, 55, 55); 
    let highlight_med = Color32::from_rgb(35, 35, 35); 

    // Outer border
    painter.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, shadow_dark));
    painter.line_segment([rect.left_top(), rect.left_bottom()], Stroke::new(1.0, shadow_dark));
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, highlight_light));
    painter.line_segment([rect.right_top(), rect.right_bottom()], Stroke::new(1.0, highlight_light));

    // Inner bevel offset by 1.0px
    let inner_rect = rect.shrink(1.0);
    painter.line_segment([inner_rect.left_top(), inner_rect.right_top()], Stroke::new(1.0, shadow_med));
    painter.line_segment([inner_rect.left_top(), inner_rect.left_bottom()], Stroke::new(1.0, shadow_med));
    painter.line_segment([inner_rect.left_bottom(), inner_rect.right_bottom()], Stroke::new(1.0, highlight_med));
    painter.line_segment([inner_rect.right_top(), inner_rect.right_bottom()], Stroke::new(1.0, highlight_med));

    // Draw decorative chassis screws in the corners of the panel
    draw_screw(ui, egui::pos2(rect.left() + 6.0, rect.top() + 6.0));
    draw_screw(ui, egui::pos2(rect.right() - 6.0, rect.top() + 6.0));

    response.inner
}

pub fn status_chip(ui: &mut Ui, label: &str, value: &str, accent: bool) {
    let color = if accent {
        // Monolith primary accent (Electric Cyan)
        Color32::from_rgb(34, 211, 238)
    } else {
        Color32::from_rgb(140, 150, 160)
    };
    
    // Draw status chip inside a small sunken bevel well
    Frame::none()
        .fill(Color32::from_rgb(5, 5, 5))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(format!("{} {}", label, value))
                    .monospace()
                    .color(color),
            );
        });
}

pub fn custom_button(ui: &mut Ui, label: &str, enabled: bool, color: Color32) -> bool {
    let btn_width = ui.available_width();
    let btn_height = 36.0;
    
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(btn_width, btn_height),
        if enabled { egui::Sense::click() } else { egui::Sense::hover() },
    );
    
    let painter = ui.painter();
    
    // Draw button background
    let bg_color = if !enabled {
        Color32::from_rgb(20, 20, 20)
    } else if response.is_pointer_button_down_on() {
        // Pressed (Sunken)
        let r = (color.r() as f32 * 0.75) as u8;
        let g = (color.g() as f32 * 0.75) as u8;
        let b = (color.b() as f32 * 0.75) as u8;
        Color32::from_rgb(r, g, b)
    } else if response.hovered() {
        // Highlighted
        let r = (color.r() as f32 * 1.15).min(255.0) as u8;
        let g = (color.g() as f32 * 1.15).min(255.0) as u8;
        let b = (color.b() as f32 * 1.15).min(255.0) as u8;
        Color32::from_rgb(r, g, b)
    } else {
        color
    };
    
    painter.rect_filled(rect, 0.0, bg_color);
    
    // Draw 3D border
    let is_pressed = enabled && response.is_pointer_button_down_on();
    let shadow = Color32::from_rgb(5, 5, 5);
    let highlight = Color32::from_rgb(255, 255, 255);
    
    let top_left = Stroke::new(2.0, if is_pressed { shadow } else { highlight });
    let bottom_right = Stroke::new(2.0, if is_pressed { highlight } else { shadow });
    
    painter.line_segment([rect.left_top(), rect.right_top()], top_left);
    painter.line_segment([rect.left_top(), rect.left_bottom()], top_left);
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], bottom_right);
    painter.line_segment([rect.right_top(), rect.right_bottom()], bottom_right);
    
    // Label text
    let text_pos = if is_pressed {
        egui::pos2(rect.center().x + 1.0, rect.center().y + 1.0)
    } else {
        rect.center()
    };
    
    let text_color = if enabled { Color32::from_rgb(5, 5, 5) } else { Color32::from_rgb(70, 70, 70) };
    let text_style = RichText::new(label).strong().monospace().color(text_color);
    
    painter.text(
        text_pos,
        Align2::CENTER_CENTER,
        text_style.text(),
        egui::FontId::monospace(12.0),
        text_color,
    );
    
    enabled && response.clicked()
}

pub fn vu_meter(ui: &mut Ui, value: f32, peak: f32) {
    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 16.0),
        egui::Sense::hover(),
    );
    let painter = ui.painter();
    
    // Background slot
    painter.rect_filled(rect, 0.0, Color32::from_rgb(5, 5, 5));
    
    // Inset borders
    let shadow = Color32::from_rgb(17, 17, 17);
    let highlight = Color32::from_rgb(45, 45, 45);
    painter.line_segment([rect.left_top(), rect.right_top()], Stroke::new(1.0, shadow));
    painter.line_segment([rect.left_top(), rect.left_bottom()], Stroke::new(1.0, shadow));
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, highlight));
    painter.line_segment([rect.right_top(), rect.right_bottom()], Stroke::new(1.0, highlight));

    let num_segments = 28;
    let seg_spacing = 2.0;
    let total_spacing = seg_spacing * (num_segments - 1) as f32;
    let seg_width = (rect.width() - 8.0 - total_spacing) / num_segments as f32;

    for i in 0..num_segments {
        let t = i as f32 / num_segments as f32;
        let seg_left = rect.left() + 4.0 + i as f32 * (seg_width + seg_spacing);
        let seg_rect = egui::Rect::from_min_max(
            egui::pos2(seg_left, rect.top() + 3.0),
            egui::pos2(seg_left + seg_width, rect.bottom() - 3.0),
        );

        let base_color = if t < 0.70 {
            Color32::from_rgb(16, 185, 129) // green
        } else if t < 0.88 {
            Color32::from_rgb(245, 158, 11) // amber
        } else {
            Color32::from_rgb(239, 68, 68) // red
        };

        let is_lit = t <= value;
        let is_peak = (t - peak).abs() < (1.0 / num_segments as f32);

        let final_color = if is_lit {
            base_color
        } else if is_peak {
            Color32::from_rgb(239, 68, 68)
        } else {
            let r = (base_color.r() as f32 * 0.12) as u8;
            let g = (base_color.g() as f32 * 0.12) as u8;
            let b = (base_color.b() as f32 * 0.12) as u8;
            Color32::from_rgb(r, g, b)
        };

        painter.rect_filled(seg_rect, 0.0, final_color);
    }
}
